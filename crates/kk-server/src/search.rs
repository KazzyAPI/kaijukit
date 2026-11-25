//! v1 search: CQL is parsed in kk-core and translated here to SQL over live content or users.
//! There is no separate index: text is pulled out of the current ADF body at query time.

use std::sync::Arc;

use axum::{
    extract::{RawQuery, State},
    response::Response,
    Extension,
};
use chrono::Utc;
use kk_core::{
    cql::{self, CqlError, Expr, Field, Op, Value as CqlValue},
    ApiError, UserId,
};
use serde_json::{json, Value};

use crate::{
    authz::Access,
    error::{db_err, R},
    http::{ts, Q},
    v1, AppState,
};

#[derive(Clone, Copy, PartialEq)]
enum Scope {
    Content,
    User,
}

/// Bind values (all text, cast in SQL) shared by every clause of one query.
struct Ctx {
    binds: Vec<String>,
    me: i64,
    scope: Scope,
}

impl Ctx {
    fn bind(&mut self, v: String, cast: &str) -> String {
        self.binds.push(v);
        format!("${}{cast}", self.binds.len() + FIXED)
    }
}

/// `$1` readable spaces and `$2` hidden ids come first.
const FIXED: usize = 2;

fn bad(e: CqlError) -> ApiError {
    ApiError::BadRequest(e.to_string())
}

fn unsupported(what: &str) -> ApiError {
    ApiError::BadRequest(format!("{what} is not supported"))
}

fn plain(v: &CqlValue, c: &Ctx) -> Result<String, ApiError> {
    match v {
        CqlValue::Str(s) => Ok(s.clone()),
        CqlValue::Func(f, _) if f.eq_ignore_ascii_case("currentUser") => Ok(c.me.to_string()),
        CqlValue::Func(f, _) => Err(unsupported(&format!("function {f}"))),
    }
}

fn like_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn clause(field: Field, op: Op, values: &[CqlValue], c: &mut Ctx) -> Result<String, ApiError> {
    // in / not in expand to comparisons
    if matches!(op, Op::In | Op::NotIn) {
        let parts = values
            .iter()
            .map(|v| clause(field, Op::Eq, std::slice::from_ref(v), c))
            .collect::<Result<Vec<_>, _>>()?;
        let any = format!("({})", parts.join(" OR "));
        return Ok(if op == Op::In {
            any
        } else {
            format!("NOT {any}")
        });
    }
    let v = values
        .first()
        .ok_or_else(|| ApiError::BadRequest("missing value".into()))?;
    let cmp = match op {
        Op::Eq => "=",
        Op::Ne => "<>",
        Op::Gt => ">",
        Op::Ge => ">=",
        Op::Lt => "<",
        Op::Le => "<=",
        _ => "",
    };
    let neg = matches!(op, Op::NotContains);
    let wrap = |s: String| if neg { format!("NOT ({s})") } else { s };
    let sql = match (c.scope, field) {
        (Scope::Content, Field::Type) => {
            let t = plain(v, c)?.to_lowercase();
            if !["page", "comment", "attachment", "blogpost"].contains(&t.as_str()) {
                return Err(unsupported(&format!("type {t}")));
            }
            format!("c.type {cmp} {}", c.bind(t, ""))
        }
        (Scope::Content, Field::Space) => {
            format!("upper(sp.key) {cmp} upper({})", c.bind(plain(v, c)?, ""))
        }
        (Scope::Content, Field::Status) => {
            format!("c.status {cmp} {}", c.bind(plain(v, c)?.to_lowercase(), ""))
        }
        (Scope::Content, Field::Title) => match op {
            Op::Contains | Op::NotContains => wrap(format!(
                "c.title ILIKE '%' || {} || '%'",
                c.bind(like_escape(&plain(v, c)?), "")
            )),
            _ => format!("lower(c.title) {cmp} lower({})", c.bind(plain(v, c)?, "")),
        },
        (Scope::Content, Field::Text) => wrap(format!(
            "d.tsv @@ plainto_tsquery('english', {})",
            c.bind(plain(v, c)?, "")
        )),
        (Scope::Content, Field::Label) => {
            let p = c.bind(plain(v, c)?, "");
            let has = format!(
                "EXISTS (SELECT 1 FROM content_labels cl JOIN labels l ON l.id = cl.label_id \
                 WHERE cl.content_id = c.id AND l.name = {p})"
            );
            if op == Op::Ne {
                format!("NOT {has}")
            } else {
                has
            }
        }
        (Scope::Content, Field::Creator) => {
            format!("c.author_id::text {cmp} {}", c.bind(plain(v, c)?, ""))
        }
        (Scope::Content, Field::Created | Field::LastModified) => {
            let at = cql::date_value(v, Utc::now().naive_utc(), 0).map_err(bad)?;
            let col = if field == Field::Created {
                "c.created_at"
            } else {
                "d.modified"
            };
            format!(
                "{col} AT TIME ZONE 'UTC' {cmp} {}",
                c.bind(at.to_string(), "::timestamp")
            )
        }
        (Scope::Content, Field::Id) => {
            format!("c.id {cmp} {}", c.bind(plain(v, c)?, "::bigint"))
        }
        (Scope::Content, Field::Parent) => {
            format!("c.parent_id {cmp} {}", c.bind(plain(v, c)?, "::bigint"))
        }
        (Scope::Content, Field::Ancestor) => format!(
            "EXISTS (WITH RECURSIVE a AS (SELECT parent_id FROM content WHERE id = c.id \
             UNION SELECT x.parent_id FROM content x JOIN a ON x.id = a.parent_id) \
             SELECT 1 FROM a WHERE a.parent_id = {})",
            c.bind(plain(v, c)?, "::bigint")
        ),
        (Scope::User, Field::UserFullname) => match op {
            Op::Contains | Op::NotContains => wrap(format!(
                "u.display_name ILIKE '%' || {} || '%'",
                c.bind(like_escape(&plain(v, c)?), "")
            )),
            _ => format!(
                "lower(u.display_name) {cmp} lower({})",
                c.bind(plain(v, c)?, "")
            ),
        },
        (Scope::User, Field::UserAccountId) => {
            format!("u.id::text {cmp} {}", c.bind(plain(v, c)?, ""))
        }
        (_, f) => return Err(unsupported(&format!("field {} in this search", f.name()))),
    };
    Ok(sql)
}

fn expr(e: &Expr, c: &mut Ctx) -> Result<String, ApiError> {
    Ok(match e {
        Expr::And(xs) | Expr::Or(xs) => {
            let join = if matches!(e, Expr::And(_)) {
                " AND "
            } else {
                " OR "
            };
            let parts = xs
                .iter()
                .map(|x| expr(x, c))
                .collect::<Result<Vec<_>, _>>()?;
            format!("({})", parts.join(join))
        }
        Expr::Not(x) => format!("NOT {}", expr(x, c)?),
        Expr::Clause { field, op, values } => {
            // `id != 0` stands in for an ORDER BY only query
            clause(*field, *op, values, c)?
        }
    })
}

fn order_by(q: &cql::Query) -> Result<String, ApiError> {
    let mut parts = Vec::new();
    for (f, desc) in &q.order {
        let col = match f {
            Field::Title => "lower(c.title)",
            Field::Created => "c.created_at",
            Field::LastModified => "d.modified",
            Field::Id => "c.id",
            f => return Err(unsupported(&format!("ordering by {}", f.name()))),
        };
        parts.push(format!("{col} {}", if *desc { "DESC" } else { "ASC" }));
    }
    parts.push("c.id".into());
    Ok(parts.join(", "))
}

pub const HIT_COLS: &str =
    "c.id, c.type, c.status, c.title, c.space_id, sp.key, sp.name, d.modified, \
    left(d.txt, 300), c.version_number::bigint";

pub const DERIVED: &str = "FROM content c JOIN spaces sp ON sp.id = c.space_id \
    JOIN content_search d ON d.content_id = c.id \
    WHERE c.status IN ('current', 'archived') AND c.type IN ('page', 'comment', 'attachment', 'blogpost') \
    AND ($1::bigint[] IS NULL OR c.space_id = ANY($1)) AND NOT (c.id = ANY($2))";

pub type Hit = (
    i64,
    String,
    String,
    String,
    i64,
    String,
    String,
    chrono::DateTime<chrono::Utc>,
    String,
    i64,
);

pub fn content_json(h: &Hit, site: &str) -> Value {
    let (id, kind, status, title, space_id, key, space_name, when, _, version) = h;
    json!({
        "id": id.to_string(), "type": kind, "status": status, "title": title,
        "space": {
            "id": space_id, "key": key, "name": space_name, "type": "global", "status": "current",
            "_expandable": {}, "_links": {"webui": format!("/spaces/{key}")},
        },
        "version": {"number": version, "minorEdit": false, "when": ts(*when)},
        "_expandable": {},
        "_links": {"webui": format!("/spaces/{key}/pages/{id}"), "self": format!("{site}/rest/api/content/{id}")},
    })
}

async fn run(
    s: &AppState,
    a: &Access,
    user: UserId,
    raw: Option<String>,
    path: &str,
    results: bool,
) -> R<Response> {
    let q = Q::parse(raw);
    let src = q
        .get("cql")
        .ok_or_else(|| ApiError::BadRequest("cql is required".into()))?;
    let query = cql::parse(src).map_err(bad)?;
    let (start, limit) = v1::window(&q, 25)?;
    let mut c = Ctx {
        binds: vec![],
        me: user.0,
        scope: Scope::Content,
    };
    let cond = expr(&query.expr, &mut c)?;
    let order = order_by(&query)?;
    let started = std::time::Instant::now();
    let sql = format!(
        "SELECT {HIT_COLS} {DERIVED} AND ({cond}) ORDER BY {order} \
         LIMIT {} OFFSET {start}",
        limit + 1
    );
    let mut qy = sqlx::query_as::<_, Hit>(&sql)
        .bind(a.readable_spaces())
        .bind(a.hidden(&s.db).await?);
    for b in &c.binds {
        qy = qy.bind(b);
    }
    let rows = qy.fetch_all(&s.db).await.map_err(|e| db_err(e, ""))?;
    let site = s.site();
    let items = rows
        .iter()
        .map(|h| {
            if !results {
                return content_json(h, &site);
            }
            let excerpt = h.8.split_whitespace().collect::<Vec<_>>().join(" ");
            json!({
                "content": content_json(h, &site),
                "title": h.3, "excerpt": excerpt,
                "url": format!("/spaces/{}/pages/{}", h.5, h.0),
                "resultGlobalContainer": {"title": h.6, "displayUrl": format!("/spaces/{}", h.5)},
                "breadcrumbs": [], "entityType": "content",
                "iconCssClass": format!("aui-icon content-type-{}", h.1),
                "lastModified": ts(h.7), "friendlyLastModified": "",
            })
        })
        .collect();
    let mut page = v1::page(&q, path, &site, items, start, limit);
    if results {
        page["cqlQuery"] = json!(src);
        page["searchDuration"] = json!(started.elapsed().as_millis() as i64);
        page["totalSize"] = json!(start + page["size"].as_i64().unwrap_or(0));
    }
    Ok(v1::ok(page))
}

pub async fn search(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Extension(u): Extension<UserId>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    run(&s, &a, u, raw, "/wiki/rest/api/search", true).await
}

pub async fn content(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Extension(u): Extension<UserId>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    run(&s, &a, u, raw, "/wiki/rest/api/content/search", false).await
}

pub async fn user(
    State(s): State<AppState>,
    Extension(u): Extension<UserId>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let src = q
        .get("cql")
        .ok_or_else(|| ApiError::BadRequest("cql is required".into()))?;
    let query = cql::parse(src).map_err(bad)?;
    let (start, limit) = v1::window(&q, 25)?;
    let mut c = Ctx {
        binds: vec![],
        me: u.0,
        scope: Scope::User,
    };
    let cond = expr(&query.expr, &mut c)?;
    let sql = format!(
        "SELECT u.id FROM users u WHERE $1::bigint[] IS NULL AND $2::bigint[] IS NOT NULL AND ({cond}) \
         ORDER BY lower(u.display_name), u.id LIMIT {} OFFSET {start}",
        limit + 1
    );
    let mut qy = sqlx::query_scalar::<_, i64>(&sql)
        .bind(None::<Vec<i64>>)
        .bind(Vec::<i64>::new());
    for b in &c.binds {
        qy = qy.bind(b);
    }
    let ids = qy.fetch_all(&s.db).await.map_err(|e| db_err(e, ""))?;
    let users = v1::users(&s.db, &ids).await?;
    let now = ts(Utc::now());
    let items = users
        .into_iter()
        .map(|u| {
            let name = u["displayName"].clone();
            json!({
                "user": u, "title": name, "excerpt": "", "url": "", "breadcrumbs": [],
                "entityType": "user", "iconCssClass": "aui-icon content-type-profile",
                "lastModified": now, "friendlyLastModified": "",
            })
        })
        .collect();
    let mut page = v1::page(
        &q,
        "/wiki/rest/api/search/user",
        &s.site(),
        items,
        start,
        limit,
    );
    page["cqlQuery"] = json!(src);
    page["searchDuration"] = json!(0);
    page["totalSize"] = json!(start + page["size"].as_i64().unwrap_or(0));
    Ok(v1::ok(page))
}
