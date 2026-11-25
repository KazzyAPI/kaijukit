//! End-to-end HTTP tests against a real Postgres (`KK_TEST_DATABASE_URL`, a superuser URL).
//! Each test gets its own freshly migrated database.

use std::sync::atomic::{AtomicU32, Ordering};

use axum::{
    body::Body,
    http::{header, HeaderMap, Method, Request, StatusCode},
    Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use http_body_util::BodyExt;
use kk_server::{app, bootstrap_admin, migrate, AppState};
use serde_json::{json, Value};
use sqlx::{postgres::PgConnectOptions, Connection, PgConnection, PgPool};
use tower::ServiceExt;

const API: &str = "/wiki/api/v2";
static N: AtomicU32 = AtomicU32::new(0);

async fn start() -> Router {
    let url = std::env::var("KK_TEST_DATABASE_URL")
        .expect("set KK_TEST_DATABASE_URL (see docs/arch/backend-dev.md)");
    let name = format!(
        "kk_test_{}_{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    );
    let mut admin = PgConnection::connect(&url).await.unwrap();
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let opts: PgConnectOptions = url.parse::<PgConnectOptions>().unwrap().database(&name);
    let db = PgPool::connect_with(opts).await.unwrap();
    migrate(&db).await.unwrap();
    bootstrap_admin(&db, "admin@kaijukit.local", "kk-dev-token")
        .await
        .unwrap();
    app(AppState {
        db,
        base_url: "http://localhost:8080".into(),
    })
}

struct Res {
    status: StatusCode,
    headers: HeaderMap,
    json: Value,
}

async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    auth: Option<&str>,
) -> Res {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(a) = auth {
        req = req.header(
            header::AUTHORIZATION,
            format!("Basic {}", STANDARD.encode(a)),
        );
    }
    let body = body.map_or_else(Body::empty, |b| Body::from(b.to_string()));
    let res = app.clone().oneshot(req.body(body).unwrap()).await.unwrap();
    let (parts, body) = res.into_parts();
    let bytes = body.collect().await.unwrap().to_bytes();
    Res {
        status: parts.status,
        headers: parts.headers,
        json: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

const ADMIN: &str = "admin@kaijukit.local:kk-dev-token";

async fn call(app: &Router, method: Method, path: &str, body: Option<Value>) -> Res {
    send(app, method, &format!("{API}{path}"), body, Some(ADMIN)).await
}

async fn get(app: &Router, path: &str) -> Res {
    call(app, Method::GET, path, None).await
}

async fn space(app: &Router, key: &str) -> Value {
    let r = call(
        app,
        Method::POST,
        "/spaces",
        Some(json!({"name": format!("Space {key}"), "key": key})),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.json);
    r.json
}

async fn page(app: &Router, space_id: &Value, title: &str, parent: Option<&Value>) -> Value {
    let mut b = json!({
        "spaceId": space_id, "title": title,
        "body": {"representation": "storage", "value": format!("<p>{title}</p>")},
    });
    if let Some(p) = parent {
        b["parentId"] = p.clone();
    }
    let r = call(app, Method::POST, "/pages", Some(b)).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.json);
    r.json
}

async fn put_page(app: &Router, id: &str, title: &str, number: i64, text: &str) -> Res {
    let body = json!({"id": id, "status": "current", "title": title,
        "body": {"representation": "storage", "value": text}, "version": {"number": number, "message": "edit"}});
    call(app, Method::PUT, &format!("/pages/{id}"), Some(body)).await
}

fn titles(r: &Res) -> Vec<&str> {
    r.json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["title"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn auth_and_probes() {
    let app = start().await;
    let uri = format!("{API}/spaces");
    for creds in [
        None,
        Some("admin@kaijukit.local:wrong"),
        Some("nobody@x.io:kk-dev-token"),
        Some("no-colon"),
    ] {
        let r = send(&app, Method::GET, &uri, None, creds).await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{creds:?}");
        assert_eq!(
            r.headers[header::WWW_AUTHENTICATE],
            "Basic realm=\"kaijukit\""
        );
        assert_eq!(
            r.json,
            json!({"errors": [{"status": 401, "code": "UNAUTHORIZED", "title": "Unauthorized", "detail": null}]})
        );
    }
    let ok = send(
        &app,
        Method::GET,
        &uri,
        None,
        Some("ADMIN@kaijukit.local:kk-dev-token"),
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK);
    assert_eq!(
        ok.json,
        json!({"results": [], "_links": {"base": "http://localhost:8080/wiki"}})
    );
    for p in ["/healthz", "/readyz"] {
        assert_eq!(
            send(&app, Method::GET, p, None, None).await.status,
            StatusCode::OK
        );
    }
}

#[tokio::test]
async fn create_space_with_homepage_and_page_bodies() {
    let app = start().await;
    let sp = space(&app, "ENG").await;
    assert_eq!(sp["key"], "ENG");
    assert_eq!(sp["name"], "Space ENG");
    assert_eq!(
        (sp["type"].as_str(), sp["status"].as_str()),
        (Some("global"), Some("current"))
    );
    assert_eq!(sp["_links"]["webui"], "/spaces/ENG");
    let home = sp["homepageId"].as_str().unwrap();

    let dup = call(
        &app,
        Method::POST,
        "/spaces",
        Some(json!({"name": "Again", "key": "ENG"})),
    )
    .await;
    assert_eq!(dup.status, StatusCode::BAD_REQUEST);

    let got = get(&app, &format!("/spaces/{}", sp["id"].as_str().unwrap())).await;
    assert_eq!(got.json["homepageId"], home);
    assert_eq!(
        get(&app, "/spaces/999999999").await.status,
        StatusCode::NOT_FOUND
    );

    let p = page(&app, &sp["id"], "Intro", None).await;
    assert_eq!(p["parentId"], home);
    assert_eq!(p["parentType"], "page");
    assert_eq!(p["version"]["number"], 1);
    assert_eq!(p["body"]["storage"]["value"], "<p>Intro</p>");
    let id = p["id"].as_str().unwrap();

    let storage = get(&app, &format!("/pages/{id}?body-format=storage")).await;
    assert_eq!(
        storage.json["body"]["storage"],
        json!({"representation": "storage", "value": "<p>Intro</p>"})
    );
    let adf = get(&app, &format!("/pages/{id}?body-format=atlas_doc_format")).await;
    let value: Value = serde_json::from_str(
        adf.json["body"]["atlas_doc_format"]["value"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        value,
        json!({"version": 1, "type": "doc", "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Intro"}]}]})
    );
    assert_eq!(
        get(&app, &format!("/pages/{id}")).await.json["body"],
        json!({})
    );

    let same_title = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"], "title": "intro", "body": {"representation": "storage", "value": ""}})),
    )
    .await;
    assert_eq!(same_title.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        same_title.json["errors"][0]["detail"],
        "A page with this title already exists in the space"
    );

    let bad = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"], "title": "Bad", "body": {"representation": "storage", "value": "<p>"}})),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn drafts_root_level_and_title_rules() {
    let app = start().await;
    let sp = space(&app, "DRF").await;
    let draft = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"], "status": "draft"})),
    )
    .await;
    assert_eq!(
        (
            draft.status,
            draft.json["status"].as_str(),
            draft.json["title"].as_str()
        ),
        (StatusCode::OK, Some("draft"), Some(""))
    );
    let untitled = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"]})),
    )
    .await;
    assert_eq!(untitled.status, StatusCode::BAD_REQUEST);

    let root = call(
        &app,
        Method::POST,
        "/pages?root-level=true",
        Some(json!({"spaceId": sp["id"], "title": "Root"})),
    )
    .await;
    assert_eq!(root.json["parentId"], Value::Null);
    let both = call(
        &app,
        Method::POST,
        "/pages?root-level=true",
        Some(json!({"spaceId": sp["id"], "title": "X", "parentId": sp["homepageId"]})),
    )
    .await;
    assert_eq!(both.status, StatusCode::BAD_REQUEST);

    let id = draft.json["id"].as_str().unwrap();
    let publish = call(
        &app,
        Method::PUT,
        &format!("/pages/{id}"),
        Some(json!({"id": id, "status": "current", "title": "Published", "body": {"representation": "storage", "value": "<p>x</p>"}, "version": {"number": 2}})),
    )
    .await;
    assert_eq!(
        (publish.status, publish.json["status"].as_str()),
        (StatusCode::OK, Some("current"))
    );
}

#[tokio::test]
async fn update_conflict_versions_and_title() {
    let app = start().await;
    let sp = space(&app, "UPD").await;
    let p = page(&app, &sp["id"], "Doc", None).await;
    let id = p["id"].as_str().unwrap();
    let put = |title: &'static str, number: i64, text: &'static str| {
        put_page(&app, id, title, number, text)
    };

    let stale = put("Doc", 1, "<p>stale</p>").await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
    assert_eq!(stale.json["errors"][0]["code"], "CONFLICT");
    assert_eq!(stale.json["errors"][0]["status"], 409);
    let skipped = put("Doc", 3, "<p>skip</p>").await;
    assert_eq!(skipped.status, StatusCode::CONFLICT);

    let ok = put("Doc v2", 2, "<h1>Two</h1>").await;
    assert_eq!(ok.status, StatusCode::OK);
    assert_eq!(ok.json["version"]["number"], 2);
    assert_eq!(ok.json["version"]["message"], "edit");
    assert_eq!(ok.json["title"], "Doc v2");
    assert_eq!(ok.json["body"]["storage"]["value"], "<h1>Two</h1>");
    assert_eq!(
        put("Doc v2", 2, "<p>again</p>").await.status,
        StatusCode::CONFLICT
    );

    let t = call(
        &app,
        Method::PUT,
        &format!("/pages/{id}/title"),
        Some(json!({"status": "current", "title": "Renamed"})),
    )
    .await;
    assert_eq!(
        (t.status, t.json["version"]["number"].clone()),
        (StatusCode::OK, json!(3))
    );
    assert_eq!(t.json["body"], json!({}));

    let versions = get(&app, &format!("/pages/{id}/versions?body-format=storage")).await;
    let nums: Vec<i64> = versions.json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["number"].as_i64().unwrap())
        .collect();
    assert_eq!(nums, vec![1, 2, 3]);
    assert_eq!(
        versions.json["results"][0]["page"]["body"]["storage"]["value"],
        "<p>Doc</p>"
    );
    assert_eq!(versions.json["results"][2]["page"]["title"], "Renamed");
    let newest = get(
        &app,
        &format!("/pages/{id}/versions?sort=-modified-date&limit=1"),
    )
    .await;
    assert_eq!(newest.json["results"][0]["number"], 3);

    let v2 = get(&app, &format!("/pages/{id}/versions/2")).await;
    assert_eq!(
        (
            v2.json["number"].clone(),
            v2.json["prevVersion"].clone(),
            v2.json["nextVersion"].clone(),
            v2.json["message"].clone()
        ),
        (json!(2), json!(1), json!(3), json!("edit"))
    );
    assert_eq!(
        get(&app, &format!("/pages/{id}/versions/9")).await.status,
        StatusCode::NOT_FOUND
    );
    let old = get(&app, &format!("/pages/{id}?version=1&body-format=storage")).await;
    assert_eq!(
        (
            old.json["title"].as_str(),
            old.json["body"]["storage"]["value"].as_str()
        ),
        (Some("Doc"), Some("<p>Doc</p>"))
    );
}

#[tokio::test]
async fn trash_restore_and_purge() {
    let app = start().await;
    let sp = space(&app, "TRS").await;
    let parent = page(&app, &sp["id"], "Parent", None).await;
    let child = page(&app, &sp["id"], "Child", Some(&parent["id"])).await;
    let (pid, cid) = (
        parent["id"].as_str().unwrap(),
        child["id"].as_str().unwrap(),
    );

    let purge_early = call(
        &app,
        Method::DELETE,
        &format!("/pages/{pid}?purge=true"),
        None,
    )
    .await;
    assert_eq!(purge_early.status, StatusCode::BAD_REQUEST);
    let home = call(
        &app,
        Method::DELETE,
        &format!("/pages/{}", sp["homepageId"].as_str().unwrap()),
        None,
    )
    .await;
    assert_eq!(home.status, StatusCode::BAD_REQUEST);

    assert_eq!(
        call(&app, Method::DELETE, &format!("/pages/{pid}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/pages/{pid}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, &format!("/pages/{cid}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, &format!("/pages/{pid}?status=trashed"))
            .await
            .json["status"],
        "trashed"
    );
    assert_eq!(
        get(
            &app,
            &format!("/spaces/{}/pages", sp["id"].as_str().unwrap())
        )
        .await
        .json["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let restore = call(
        &app,
        Method::PUT,
        &format!("/pages/{pid}"),
        Some(json!({"id": pid, "status": "current", "title": "Parent", "body": {"representation": "storage", "value": "<p>back</p>"}, "version": {"number": 2}})),
    )
    .await;
    assert_eq!(
        (restore.status, restore.json["status"].as_str()),
        (StatusCode::OK, Some("current"))
    );
    assert_eq!(
        get(&app, &format!("/pages/{cid}")).await.json["status"],
        "current"
    );

    call(&app, Method::DELETE, &format!("/pages/{pid}"), None).await;
    assert_eq!(
        call(
            &app,
            Method::DELETE,
            &format!("/pages/{pid}?purge=true"),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    for id in [pid, cid] {
        assert_eq!(
            get(&app, &format!("/pages/{id}?status=trashed"))
                .await
                .status,
            StatusCode::NOT_FOUND
        );
    }
    let again = page(&app, &sp["id"], "Parent", None).await;
    assert_eq!(again["title"], "Parent");
}

#[tokio::test]
async fn children_ancestors_descendants() {
    let app = start().await;
    let sp = space(&app, "TREE").await;
    let top = page(&app, &sp["id"], "Top", None).await;
    let tid = top["id"].as_str().unwrap();
    let mut kids = vec![];
    for t in ["Zeta", "Alpha", "Mid"] {
        kids.push(page(&app, &sp["id"], t, Some(&top["id"])).await);
    }
    let grand = page(&app, &sp["id"], "Grand", Some(&kids[1]["id"])).await;
    let great = page(&app, &sp["id"], "Great", Some(&grand["id"])).await;

    let c = get(&app, &format!("/pages/{tid}/children")).await;
    assert_eq!(titles(&c), ["Zeta", "Alpha", "Mid"]);
    let pos: Vec<i64> = c.json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["childPosition"].as_i64().unwrap())
        .collect();
    assert_eq!(pos, [0, 1, 2]);
    assert_eq!(c.json["results"][0]["status"], "current");
    assert_eq!(c.json["results"][0]["spaceId"], sp["id"]);
    let rev = get(&app, &format!("/pages/{tid}/children?sort=-child-position")).await;
    assert_eq!(titles(&rev), ["Mid", "Alpha", "Zeta"]);
    let direct = get(&app, &format!("/pages/{tid}/direct-children")).await;
    assert_eq!(direct.json["results"][0]["type"], "page");

    let d = get(&app, &format!("/pages/{tid}/descendants")).await;
    assert_eq!(titles(&d), ["Zeta", "Alpha", "Mid", "Grand"]);
    assert_eq!(d.json["results"][3]["depth"], 2);
    assert_eq!(d.json["results"][3]["parentId"], kids[1]["id"]);
    let deep = get(&app, &format!("/pages/{tid}/descendants?depth=3")).await;
    assert_eq!(titles(&deep), ["Zeta", "Alpha", "Mid", "Grand", "Great"]);
    assert_eq!(
        get(&app, &format!("/pages/{tid}/descendants?depth=11"))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );

    let a = get(
        &app,
        &format!("/pages/{}/ancestors", great["id"].as_str().unwrap()),
    )
    .await;
    assert_eq!(
        a.json["results"],
        json!([
            {"id": sp["homepageId"], "type": "page"},
            {"id": top["id"], "type": "page"},
            {"id": kids[1]["id"], "type": "page"},
            {"id": grand["id"], "type": "page"},
        ])
    );
    assert_eq!(
        get(&app, "/pages/1/children").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn cursor_pagination() {
    let app = start().await;
    let sp = space(&app, "PAG").await;
    let sid = sp["id"].as_str().unwrap();
    for t in ["B", "C", "D"] {
        page(&app, &sp["id"], t, None).await;
    }
    let path = format!("/spaces/{sid}/pages?limit=3&sort=title");
    let first = get(&app, &path).await;
    assert_eq!(titles(&first), ["B", "C", "D"]);
    let next = first.json["_links"]["next"].as_str().unwrap();
    assert!(
        next.starts_with(&format!(
            "{API}/spaces/{sid}/pages?limit=3&sort=title&cursor="
        )),
        "{next}"
    );
    assert_eq!(
        first.headers[header::LINK].to_str().unwrap(),
        format!("<{next}>; rel=\"next\"")
    );
    assert_eq!(first.json["_links"]["base"], "http://localhost:8080/wiki");

    let second = send(&app, Method::GET, next, None, Some(ADMIN)).await;
    assert_eq!(titles(&second), ["Space PAG"]);
    assert_eq!(
        second.json["_links"],
        json!({"base": "http://localhost:8080/wiki"})
    );
    assert!(second.headers.get(header::LINK).is_none());

    let spaces = get(&app, "/spaces?limit=1").await;
    assert_eq!(spaces.json["results"].as_array().unwrap().len(), 1);
    assert!(spaces.json["_links"].get("next").is_none());

    for bad in ["cursor=garbage", "limit=0", "limit=251", "limit=x"] {
        assert_eq!(
            get(&app, &format!("/pages?{bad}")).await.status,
            StatusCode::BAD_REQUEST,
            "{bad}"
        );
    }
}

#[tokio::test]
async fn request_validation() {
    let app = start().await;
    let sp = space(&app, "VAL").await;
    let r = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"title": "no space"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.json["errors"][0]["code"], "INVALID_REQUEST_PARAMETER");
    let r = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": "424242", "title": "t"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    assert_eq!(
        get(&app, "/pages/abc").await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        get(&app, "/pages?body-format=wiki").await.status,
        StatusCode::BAD_REQUEST
    );
    let wiki = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"], "title": "w", "body": {"representation": "wiki", "value": "h1. x"}})),
    )
    .await;
    assert_eq!(wiki.status, StatusCode::BAD_REQUEST);
    let big = "x".repeat(5 * 1024 * 1024 + 1);
    let r = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"], "title": big})),
    )
    .await;
    assert_eq!(
        (r.status, r.json["errors"][0]["status"].clone()),
        (StatusCode::PAYLOAD_TOO_LARGE, json!(413))
    );
}

fn draft_put(id: &str, title: &str, text: &str) -> Value {
    json!({"id": id, "status": "draft", "title": title,
        "body": {"representation": "storage", "value": text}, "version": {"number": 99}})
}

#[tokio::test]
async fn drafts_of_published_pages() {
    let app = start().await;
    let sp = space(&app, "PDR").await;
    let p = page(&app, &sp["id"], "Live", None).await;
    let id = p["id"].as_str().unwrap();
    let path = format!("/pages/{id}");
    let body = |r: &Res| {
        r.json["body"]["storage"]["value"]
            .as_str()
            .unwrap()
            .to_string()
    };

    let saved = call(
        &app,
        Method::PUT,
        &path,
        Some(draft_put(id, "Live v2", "<p>wip</p>")),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.json);
    assert_eq!(
        (
            saved.json["status"].as_str(),
            saved.json["title"].as_str(),
            saved.json["version"]["number"].as_i64()
        ),
        (Some("draft"), Some("Live v2"), Some(1))
    );
    call(
        &app,
        Method::PUT,
        &path,
        Some(draft_put(id, "Live v2", "<p>wip2</p>")),
    )
    .await;

    let cur = get(&app, &format!("{path}?body-format=storage")).await;
    assert_eq!(
        (
            cur.json["status"].as_str(),
            cur.json["title"].as_str(),
            body(&cur).as_str()
        ),
        (Some("current"), Some("Live"), "<p>Live</p>")
    );
    let d = get(&app, &format!("{path}?get-draft=true&body-format=storage")).await;
    assert_eq!(
        (
            d.json["status"].as_str(),
            d.json["title"].as_str(),
            body(&d).as_str()
        ),
        (Some("draft"), Some("Live v2"), "<p>wip2</p>")
    );
    let vs = get(&app, &format!("{path}/versions")).await;
    assert_eq!(vs.json["results"].as_array().unwrap().len(), 1);
    assert_eq!(
        titles(&get(&app, "/pages").await)
            .iter()
            .filter(|t| **t == "Live")
            .count(),
        1
    );

    let gone = call(&app, Method::DELETE, &format!("{path}?draft=true"), None).await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT);
    let after = get(&app, &format!("{path}?get-draft=true&body-format=storage")).await;
    assert_eq!(
        (after.json["status"].as_str(), body(&after).as_str()),
        (Some("current"), "<p>Live</p>")
    );
    let again = call(&app, Method::DELETE, &format!("{path}?draft=true"), None).await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);

    call(
        &app,
        Method::PUT,
        &path,
        Some(draft_put(id, "Live v2", "<p>wip3</p>")),
    )
    .await;
    let published = put_page(&app, id, "Live v2", 2, "<p>final</p>").await;
    assert_eq!(
        (
            published.json["status"].as_str(),
            published.json["version"]["number"].as_i64()
        ),
        (Some("current"), Some(2))
    );
    let after = get(&app, &format!("{path}?get-draft=true&body-format=storage")).await;
    assert_eq!(
        (
            after.json["status"].as_str(),
            after.json["title"].as_str(),
            body(&after).as_str()
        ),
        (Some("current"), Some("Live v2"), "<p>final</p>")
    );
    let stale = put_page(&app, id, "Live v2", 2, "x").await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn users_bulk_and_access_check() {
    let app = start().await;
    let sp = space(&app, "USR").await;
    let author = sp["authorId"].as_str().unwrap().to_string();
    let r = call(
        &app,
        Method::POST,
        "/users-bulk",
        Some(json!({"accountIds": [author, "999999", "abc"]})),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.json);
    assert_eq!(
        r.json["results"],
        json!([{"accountId": author, "displayName": "Admin", "publicName": "Admin", "email": "admin@kaijukit.local",
            "accountType": "atlassian", "accountStatus": "active", "isExternalCollaborator": false}])
    );
    let empty = call(
        &app,
        Method::POST,
        "/users-bulk",
        Some(json!({"accountIds": []})),
    )
    .await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);
    let c = call(
        &app,
        Method::POST,
        "/user/access/check-access-by-email",
        Some(json!({"emails": ["ADMIN@kaijukit.local", "nobody@x.io", "bad"]})),
    )
    .await;
    assert_eq!(
        c.json,
        json!({"emailsWithoutAccess": ["nobody@x.io"], "invalidEmails": ["bad"]})
    );
}

#[tokio::test]
async fn list_filters_and_sorts() {
    let app = start().await;
    let sp = space(&app, "FLT").await;
    let a = page(&app, &sp["id"], "Alpha", None).await;
    let b = page(&app, &sp["id"], "Beta", None).await;
    put_page(&app, a["id"].as_str().unwrap(), "Alpha", 2, "<p>x</p>").await;
    let q = |extra: &str| format!("/pages?space-id={}&{extra}", sp["id"].as_str().unwrap());
    assert_eq!(titles(&get(&app, &q("title=Beta")).await), ["Beta"]);
    assert_eq!(
        titles(&get(&app, &q("sort=-modified-date&title=Alpha")).await),
        ["Alpha"]
    );
    let all = get(&app, &q("sort=-modified-date")).await;
    assert_eq!(titles(&all)[0], "Alpha");
    let ids = q(&format!("id={}", b["id"].as_str().unwrap()));
    assert_eq!(titles(&get(&app, &ids).await), ["Beta"]);
    let trashed = get(&app, &q("status=trashed")).await;
    assert_eq!(titles(&trashed).len(), 0);
    assert_eq!(
        get(&app, &q("sort=bogus")).await.status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn rich_nodes_round_trip_through_storage() {
    let app = start().await;
    let sp = space(&app, "RICH").await;
    let xml = "<ac:structured-macro ac:name=\"info\" ac:schema-version=\"1\"><ac:rich-text-body><p>note</p></ac:rich-text-body></ac:structured-macro>\
               <p>due <time datetime=\"2023-11-14\" /> <span style=\"color: #ff0000;\">red</span></p>";
    let r = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"], "title": "Rich", "body": {"representation": "storage", "value": xml}})),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.json);
    assert_eq!(r.json["body"]["storage"]["value"], xml);
    let id = r.json["id"].as_str().unwrap();
    let adf = get(&app, &format!("/pages/{id}?body-format=atlas_doc_format")).await;
    let value: Value = serde_json::from_str(
        adf.json["body"]["atlas_doc_format"]["value"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        value["content"],
        json!([
            {"type": "panel", "attrs": {"panelType": "info"}, "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "note"}]}]},
            {"type": "paragraph", "content": [
                {"type": "text", "text": "due "},
                {"type": "date", "attrs": {"timestamp": "1699920000000"}},
                {"type": "text", "text": " "},
                {"type": "text", "text": "red", "marks": [{"type": "textColor", "attrs": {"color": "#ff0000"}}]},
            ]},
        ])
    );
}

fn storage(v: &str) -> Value {
    json!({"representation": "storage", "value": v})
}

async fn post_comment(app: &Router, path: &str, body: Value) -> Res {
    call(app, Method::POST, path, Some(body)).await
}

#[tokio::test]
async fn footer_comments_threads_and_versions() {
    let app = start().await;
    let sp = space(&app, "FC").await;
    let pg = page(&app, &sp["id"], "Notes", None).await;
    let pid = pg["id"].as_str().unwrap();

    let c = post_comment(
        &app,
        "/footer-comments",
        json!({"pageId": pid, "body": storage("<p>first</p>")}),
    )
    .await;
    assert_eq!(c.status, StatusCode::CREATED, "{}", c.json);
    let cid = c.json["id"].as_str().unwrap().to_string();
    let admin = pg["authorId"].as_str().unwrap();
    assert_eq!(c.json["status"], "current");
    assert_eq!(c.json["title"], "Re: Notes");
    assert_eq!(c.json["pageId"], pid);
    assert_eq!(c.json["version"]["number"], 1);
    assert_eq!(c.json["version"]["authorId"], admin);
    assert_eq!(
        c.json["body"]["storage"],
        json!({"representation": "storage", "value": "<p>first</p>"})
    );
    assert_eq!(c.json["_links"]["base"], "http://localhost:8080/wiki");
    assert_eq!(
        c.json["_links"]["webui"],
        format!("/spaces/FC/pages/{pid}/Notes?focusedCommentId={cid}")
    );

    let reply = post_comment(
        &app,
        "/footer-comments",
        json!({"parentCommentId": cid, "body": storage("<p>re</p>")}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
    assert_eq!(reply.json["parentCommentId"], cid);
    assert_eq!(reply.json["pageId"], pid);
    let rid = reply.json["id"].as_str().unwrap().to_string();

    let roots = get(
        &app,
        &format!("/pages/{pid}/footer-comments?body-format=storage"),
    )
    .await;
    assert_eq!(roots.json["results"].as_array().unwrap().len(), 1);
    assert_eq!(
        roots.json["results"][0]["body"]["storage"]["value"],
        "<p>first</p>"
    );
    let kids = get(&app, &format!("/footer-comments/{cid}/children")).await;
    assert_eq!(kids.json["results"][0]["id"], rid);
    assert_eq!(
        get(&app, "/footer-comments").await.json["results"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        get(&app, "/inline-comments").await.json["results"],
        json!([])
    );
    assert_eq!(
        get(&app, &format!("/inline-comments/{cid}")).await.status,
        StatusCode::NOT_FOUND
    );

    let edit = |n: i64| json!({"version": {"number": n, "message": "typo"}, "body": storage("<p>edited</p>")});
    let stale = call(
        &app,
        Method::PUT,
        &format!("/footer-comments/{cid}"),
        Some(edit(3)),
    )
    .await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
    let up = call(
        &app,
        Method::PUT,
        &format!("/footer-comments/{cid}"),
        Some(edit(2)),
    )
    .await;
    assert_eq!(up.status, StatusCode::OK, "{}", up.json);
    assert_eq!(up.json["version"]["number"], 2);
    assert_eq!(up.json["version"]["message"], "typo");
    assert_eq!(up.json["body"]["storage"]["value"], "<p>edited</p>");

    let vs = get(
        &app,
        &format!("/footer-comments/{cid}/versions?body-format=storage"),
    )
    .await;
    let numbers: Vec<_> = vs.json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["number"].clone())
        .collect();
    assert_eq!(numbers, [json!(1), json!(2)]);
    assert_eq!(
        vs.json["results"][0]["comment"]["body"]["storage"]["value"],
        "<p>first</p>"
    );
    let one = get(&app, &format!("/footer-comments/{cid}/versions/1")).await;
    assert_eq!(one.json["nextVersion"], 2);
    assert_eq!(one.json["number"], 1);
    let with = get(
        &app,
        &format!("/footer-comments/{cid}?include-versions=true&version=1&body-format=storage"),
    )
    .await;
    assert_eq!(with.json["body"]["storage"]["value"], "<p>first</p>");
    assert_eq!(
        with.json["versions"]["results"].as_array().unwrap().len(),
        2
    );

    assert_eq!(
        call(
            &app,
            Method::DELETE,
            &format!("/footer-comments/{cid}"),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/footer-comments/{rid}")).await.status,
        StatusCode::NOT_FOUND
    );
    let bad = post_comment(
        &app,
        "/footer-comments",
        json!({"pageId": pid, "parentCommentId": rid, "body": storage("<p>x</p>")}),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let missing = post_comment(
        &app,
        "/footer-comments",
        json!({"pageId": "999999", "body": storage("<p>x</p>")}),
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn inline_comments_anchor_resolve_and_dangle() {
    let app = start().await;
    let sp = space(&app, "IC").await;
    let pg = call(
        &app,
        Method::POST,
        "/pages",
        Some(json!({"spaceId": sp["id"], "title": "Doc",
        "body": storage("<p>alpha beta alpha</p>")})),
    )
    .await
    .json;
    let pid = pg["id"].as_str().unwrap();
    let props = |sel: &str, count: i64, index: i64| {
        json!({"pageId": pid, "body": storage("<p>q</p>"),
        "inlineCommentProperties": {"textSelection": sel, "textSelectionMatchCount": count, "textSelectionMatchIndex": index}})
    };

    let missing = post_comment(&app, "/inline-comments", props("zzz", 1, 0)).await;
    assert_eq!(missing.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        missing.json["errors"][0]["detail"],
        "textSelection was not found in the page body"
    );
    let count = post_comment(&app, "/inline-comments", props("alpha", 1, 0)).await;
    assert_eq!(
        count.json["errors"][0]["detail"],
        "textSelectionMatchCount is 1 but the selection occurs 2 times"
    );
    assert_eq!(
        post_comment(
            &app,
            "/inline-comments",
            json!({"pageId": pid, "body": storage("<p>q</p>")})
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );

    let c = post_comment(&app, "/inline-comments", props("alpha", 2, 1)).await;
    assert_eq!(c.status, StatusCode::CREATED, "{}", c.json);
    let cid = c.json["id"].as_str().unwrap();
    let marker = c.json["properties"]["inlineMarkerRef"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(marker.len(), 36);
    assert_eq!(c.json["properties"]["inlineOriginalSelection"], "alpha");
    assert_eq!(c.json["resolutionStatus"], "open");

    let body = get(&app, &format!("/pages/{pid}?body-format=storage")).await;
    assert_eq!(
        body.json["body"]["storage"]["value"],
        format!("<p>alpha beta <ac:inline-comment-marker ac:ref=\"{marker}\">alpha</ac:inline-comment-marker></p>")
    );
    assert_eq!(body.json["version"]["number"], 1);

    let open = get(
        &app,
        &format!("/pages/{pid}/inline-comments?resolution-status=open"),
    )
    .await;
    assert_eq!(open.json["results"][0]["resolutionStatus"], "open");
    assert_eq!(
        open.json["results"][0]["properties"]["inlineMarkerRef"],
        marker
    );

    let resolve = |r: bool| json!({"resolved": r});
    let done = call(
        &app,
        Method::PUT,
        &format!("/inline-comments/{cid}"),
        Some(resolve(true)),
    )
    .await;
    assert_eq!(done.json["resolutionStatus"], "resolved");
    assert_eq!(done.json["resolutionLastModifierId"], pg["authorId"]);
    let filtered = get(
        &app,
        &format!("/pages/{pid}/inline-comments?resolution-status=open,reopened"),
    )
    .await;
    assert_eq!(filtered.json["results"], json!([]));
    let again = call(
        &app,
        Method::PUT,
        &format!("/inline-comments/{cid}"),
        Some(resolve(false)),
    )
    .await;
    assert_eq!(again.json["resolutionStatus"], "reopened");

    let second = post_comment(&app, "/inline-comments", props("beta", 1, 0)).await;
    let sid = second.json["id"].as_str().unwrap();
    assert_eq!(
        call(
            &app,
            Method::DELETE,
            &format!("/inline-comments/{sid}"),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    let after = get(&app, &format!("/pages/{pid}?body-format=storage")).await;
    assert_eq!(
        after.json["body"]["storage"]["value"],
        format!("<p>alpha beta <ac:inline-comment-marker ac:ref=\"{marker}\">alpha</ac:inline-comment-marker></p>")
    );

    let edited = put_page(&app, pid, "Doc", 2, "<p>gamma</p>").await;
    assert_eq!(edited.status, StatusCode::OK, "{}", edited.json);
    let dangling = get(&app, &format!("/inline-comments/{cid}")).await;
    assert_eq!(dangling.json["resolutionStatus"], "dangling");
    let reopen = call(
        &app,
        Method::PUT,
        &format!("/inline-comments/{cid}"),
        Some(resolve(true)),
    )
    .await;
    assert_eq!(reopen.status, StatusCode::BAD_REQUEST);
}

async fn v1(app: &Router, method: Method, path: &str, body: Option<Value>) -> Res {
    send(
        app,
        method,
        &format!("/wiki/rest/api{path}"),
        body,
        Some(ADMIN),
    )
    .await
}

fn names(r: &Res) -> Vec<&str> {
    r.json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn labels_on_pages_spaces_and_content() {
    let app = start().await;
    let sp = space(&app, "LB").await;
    let pg = page(&app, &sp["id"], "Tagged", None).await;
    let pid = pg["id"].as_str().unwrap();

    let add = v1(
        &app,
        Method::POST,
        &format!("/content/{pid}/label"),
        Some(json!([{"prefix": "global", "name": "Alpha"}, {"prefix": "my", "name": "beta"}])),
    )
    .await;
    assert_eq!(add.status, StatusCode::OK, "{}", add.json);
    assert_eq!(add.json["size"], 2);
    assert_eq!(add.json["results"][0]["name"], "alpha");
    assert_eq!(add.json["results"][0]["label"], "alpha");
    assert_eq!(add.json["results"][0]["prefix"], "global");
    let again = v1(
        &app,
        Method::POST,
        &format!("/content/{pid}/label"),
        Some(json!({"name": "alpha"})),
    )
    .await;
    assert_eq!(again.json["size"], 2);
    let bad = v1(
        &app,
        Method::POST,
        &format!("/content/{pid}/label"),
        Some(json!({"name": "has space"})),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);

    let list = get(&app, &format!("/pages/{pid}/labels")).await;
    assert_eq!(names(&list), ["alpha", "beta"]);
    assert_eq!(list.json["results"][1]["prefix"], "my");
    assert_eq!(
        names(&get(&app, &format!("/pages/{pid}/labels?prefix=my")).await),
        ["beta"]
    );
    assert_eq!(
        names(&get(&app, &format!("/pages/{pid}/labels?sort=-name")).await),
        ["beta", "alpha"]
    );
    assert_eq!(
        get(&app, &format!("/pages/{pid}/labels?prefix=bogus"))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );

    let sl = v1(
        &app,
        Method::POST,
        "/space/LB/label",
        Some(json!({"name": "team-x"})),
    )
    .await;
    assert_eq!(sl.json["results"][0]["prefix"], "team");
    let sid = sp["id"].as_str().unwrap();
    assert_eq!(
        names(&get(&app, &format!("/spaces/{sid}/labels")).await),
        ["team-x"]
    );
    assert_eq!(
        names(&get(&app, &format!("/spaces/{sid}/content/labels")).await),
        ["alpha", "beta"]
    );
    assert_eq!(
        get(&app, "/labels").await.json["results"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    let del = v1(
        &app,
        Method::DELETE,
        &format!("/content/{pid}/label/alpha"),
        None,
    )
    .await;
    assert_eq!(del.status, StatusCode::NO_CONTENT);
    let q = v1(
        &app,
        Method::DELETE,
        &format!("/content/{pid}/label?name=beta"),
        None,
    )
    .await;
    assert_eq!(q.status, StatusCode::NO_CONTENT);
    assert_eq!(
        v1(
            &app,
            Method::DELETE,
            &format!("/content/{pid}/label/beta"),
            None
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, &format!("/pages/{pid}/labels")).await.json["results"],
        json!([])
    );
    assert_eq!(
        get(&app, "/pages/999999/labels").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn properties_versions_and_app_properties() {
    let app = start().await;
    let sp = space(&app, "PR").await;
    let pg = page(&app, &sp["id"], "Props", None).await;
    let pid = pg["id"].as_str().unwrap();
    let admin = pg["authorId"].as_str().unwrap();

    let c = call(
        &app,
        Method::POST,
        &format!("/pages/{pid}/properties"),
        Some(json!({"key": "color", "value": {"a": [1, 2]}})),
    )
    .await;
    assert_eq!(c.status, StatusCode::OK, "{}", c.json);
    assert_eq!(c.json["key"], "color");
    assert_eq!(c.json["value"], json!({"a": [1, 2]}));
    assert_eq!(c.json["version"]["number"], 1);
    assert_eq!(c.json["version"]["authorId"], admin);
    let id = c.json["id"].as_str().unwrap().to_string();
    let dup = call(
        &app,
        Method::POST,
        &format!("/pages/{pid}/properties"),
        Some(json!({"key": "color", "value": 1})),
    )
    .await;
    assert_eq!(dup.status, StatusCode::CONFLICT);

    let up = |n: i64, key: &str| json!({"key": key, "value": "blue", "version": {"number": n, "message": "m"}});
    let stale = call(
        &app,
        Method::PUT,
        &format!("/pages/{pid}/properties/{id}"),
        Some(up(1, "color")),
    )
    .await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
    let rename = call(
        &app,
        Method::PUT,
        &format!("/pages/{pid}/properties/{id}"),
        Some(up(2, "other")),
    )
    .await;
    assert_eq!(rename.status, StatusCode::BAD_REQUEST);
    let ok = call(
        &app,
        Method::PUT,
        &format!("/pages/{pid}/properties/{id}"),
        Some(up(2, "color")),
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.json);
    assert_eq!(ok.json["value"], "blue");
    assert_eq!(ok.json["version"]["number"], 2);
    assert_eq!(ok.json["version"]["message"], "m");

    call(
        &app,
        Method::POST,
        &format!("/pages/{pid}/properties"),
        Some(json!({"key": "a", "value": true})),
    )
    .await;
    let keys = |r: &Res| {
        r.json["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["key"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        keys(&get(&app, &format!("/pages/{pid}/properties?sort=key")).await),
        ["a", "color"]
    );
    assert_eq!(
        keys(&get(&app, &format!("/pages/{pid}/properties?key=color")).await),
        ["color"]
    );
    let big = "x".repeat(40_000);
    let too = call(
        &app,
        Method::POST,
        &format!("/pages/{pid}/properties"),
        Some(json!({"key": "big", "value": big})),
    )
    .await;
    assert_eq!(too.status, StatusCode::PAYLOAD_TOO_LARGE);

    assert_eq!(
        call(
            &app,
            Method::DELETE,
            &format!("/pages/{pid}/properties/{id}"),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/pages/{pid}/properties/{id}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let cm = call(
        &app,
        Method::POST,
        "/footer-comments",
        Some(json!({"pageId": pid, "body": storage("<p>c</p>")})),
    )
    .await;
    let cid = cm.json["id"].as_str().unwrap();
    let cp = call(
        &app,
        Method::POST,
        &format!("/comments/{cid}/properties"),
        Some(json!({"key": "k", "value": 1})),
    )
    .await;
    assert_eq!(cp.status, StatusCode::OK);
    assert_eq!(
        get(&app, &format!("/comments/{cid}/properties")).await.json["results"][0]["value"],
        1
    );

    let sid = sp["id"].as_str().unwrap();
    let sc = call(
        &app,
        Method::POST,
        &format!("/spaces/{sid}/properties"),
        Some(json!({"key": "theme", "value": {"d": 1}})),
    )
    .await;
    assert_eq!(sc.status, StatusCode::CREATED, "{}", sc.json);
    assert_eq!(sc.json["createdBy"], admin);
    assert_eq!(sc.json["version"]["number"], 1);
    let spid = sc.json["id"].as_str().unwrap();
    let su = call(
        &app,
        Method::PUT,
        &format!("/spaces/{sid}/properties/{spid}"),
        Some(up(2, "theme")),
    )
    .await;
    assert_eq!(su.json["version"]["number"], 2);
    assert_eq!(
        get(&app, &format!("/spaces/{sid}/properties/{spid}"))
            .await
            .json["value"],
        "blue"
    );
    assert_eq!(
        call(
            &app,
            Method::DELETE,
            &format!("/spaces/{sid}/properties/{spid}"),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );

    let put = call(
        &app,
        Method::PUT,
        "/app/properties/flag",
        Some(json!({"on": true})),
    )
    .await;
    assert_eq!(put.status, StatusCode::CREATED);
    let put = call(
        &app,
        Method::PUT,
        "/app/properties/flag",
        Some(json!({"on": false})),
    )
    .await;
    assert_eq!(put.status, StatusCode::OK);
    assert_eq!(
        get(&app, "/app/properties/flag").await.json,
        json!({"key": "flag", "value": {"on": false}})
    );
    assert_eq!(
        get(&app, "/app/properties").await.json["results"],
        json!([{"key": "flag", "value": {"on": false}}])
    );
    assert_eq!(
        call(&app, Method::DELETE, "/app/properties/flag", None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, "/app/properties/flag").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn likes_and_operations() {
    let app = start().await;
    let sp = space(&app, "LK").await;
    let pg = page(&app, &sp["id"], "Liked", None).await;
    let pid = pg["id"].as_str().unwrap();
    assert_eq!(
        get(&app, &format!("/pages/{pid}/likes/count")).await.json,
        json!({"count": 0})
    );
    assert_eq!(
        v1(&app, Method::PUT, &format!("/content/{pid}/like"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        v1(&app, Method::PUT, &format!("/content/{pid}/like"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/pages/{pid}/likes/count")).await.json,
        json!({"count": 1})
    );
    assert_eq!(
        get(&app, &format!("/pages/{pid}/likes/users")).await.json["results"],
        json!([{"accountId": pg["authorId"]}])
    );

    let cm = call(
        &app,
        Method::POST,
        "/footer-comments",
        Some(json!({"pageId": pid, "body": storage("<p>c</p>")})),
    )
    .await;
    let cid = cm.json["id"].as_str().unwrap();
    v1(&app, Method::PUT, &format!("/content/{cid}/like"), None).await;
    assert_eq!(
        get(&app, &format!("/footer-comments/{cid}/likes/count"))
            .await
            .json["count"],
        1
    );
    assert_eq!(
        get(&app, &format!("/inline-comments/{cid}/likes/count"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    v1(&app, Method::DELETE, &format!("/content/{cid}/like"), None).await;
    assert_eq!(
        get(&app, &format!("/footer-comments/{cid}/likes/count"))
            .await
            .json["count"],
        0
    );

    let ops = get(&app, &format!("/pages/{pid}/operations")).await;
    assert_eq!(
        ops.json["operations"][0],
        json!({"operation": "read", "targetType": "page"})
    );
    assert_eq!(ops.json["operations"].as_array().unwrap().len(), 9);
    let sid = sp["id"].as_str().unwrap();
    assert_eq!(
        get(&app, &format!("/spaces/{sid}/operations")).await.json["operations"][7],
        json!({"operation": "administer", "targetType": "space"})
    );
    assert_eq!(
        get(&app, &format!("/footer-comments/{cid}/operations"))
            .await
            .json["operations"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        get(&app, "/pages/999999/operations").await.status,
        StatusCode::NOT_FOUND
    );
}
