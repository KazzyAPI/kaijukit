//! Loads the facts the pure rules in `kk_core::perm` need and applies them: a guard
//! in front of every by-id route, and helpers for list filtering.

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{MatchedPath, RawPathParams, Request, State},
    http::Method,
    middleware::Next,
    response::{IntoResponse, Response},
};
use kk_core::{
    perm::{self, Grants, Principal, Restriction, Subject},
    ApiError, UserId,
};
use sqlx::PgPool;

use crate::{
    error::{db_err, E},
    http::{parse_attachment_id, parse_id},
    AppState,
};

type Res<T> = Result<T, ApiError>;

pub fn subject(kind: &str, id: i64) -> Subject {
    if kind == "user" {
        Subject::User(id)
    } else {
        Subject::Group(id)
    }
}

/// What a content id resolves to for permission purposes.
pub struct Target {
    pub space_id: i64,
    pub ty: String,
    pub author_id: i64,
    /// The page whose restrictions govern this content (itself for pages).
    pub page_id: i64,
}

pub struct Access {
    pub p: Principal,
    grants: HashMap<i64, Grants>,
}

impl Access {
    pub async fn load(db: &PgPool, user: UserId) -> Res<Access> {
        let admin: bool = sqlx::query_scalar("SELECT is_admin FROM users WHERE id = $1")
            .bind(user.0)
            .fetch_one(db)
            .await
            .map_err(|e| db_err(e, ""))?;
        let groups: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM groups WHERE everyone UNION SELECT group_id FROM group_members WHERE user_id = $1",
        )
        .bind(user.0)
        .fetch_all(db)
        .await
        .map_err(|e| db_err(e, ""))?;
        let rows: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT space_id, operation, target FROM space_permissions WHERE \
             (subject_type = 'user' AND subject_id = $1) OR (subject_type = 'group' AND subject_id = ANY($2))",
        )
        .bind(user.0)
        .bind(&groups)
        .fetch_all(db)
        .await
        .map_err(|e| db_err(e, ""))?;
        let mut grants: HashMap<i64, Grants> = HashMap::new();
        for (space, op, target) in rows {
            grants.entry(space).or_default().held.insert((op, target));
        }
        let p = Principal {
            id: user.0,
            admin,
            groups: groups.into_iter().collect(),
        };
        Ok(Access { p, grants })
    }

    /// Grants a group holds on its own in one space.
    pub async fn grants_for_group(db: &PgPool, group: i64, space: i64) -> Res<Grants> {
        let held: Vec<(String, String)> = sqlx::query_as(
            "SELECT operation, target FROM space_permissions \
             WHERE space_id = $1 AND subject_type = 'group' AND subject_id = $2",
        )
        .bind(space)
        .bind(group)
        .fetch_all(db)
        .await
        .map_err(|e| db_err(e, ""))?;
        Ok(Grants {
            held: held.into_iter().collect(),
            all: false,
        })
    }

    pub fn grants(&self, space: i64) -> Grants {
        if self.p.admin {
            return Grants {
                all: true,
                ..Default::default()
            };
        }
        self.grants.get(&space).cloned().unwrap_or_default()
    }

    /// Spaces the caller may read; `None` means all of them (site administrator).
    pub fn readable_spaces(&self) -> Option<Vec<i64>> {
        (!self.p.admin).then(|| {
            self.grants
                .iter()
                .filter(|(_, g)| g.has("read", "space"))
                .map(|(s, _)| *s)
                .collect()
        })
    }

    /// Content ids the caller cannot see because of read restrictions (whole subtrees).
    pub async fn hidden(&self, db: &PgPool) -> Res<Vec<i64>> {
        if self.p.admin {
            return Ok(vec![]);
        }
        let rows: Vec<(i64, String, String, i64)> = sqlx::query_as(
            "SELECT content_id, operation, subject_type, subject_id FROM content_restrictions",
        )
        .fetch_all(db)
        .await
        .map_err(|e| db_err(e, ""))?;
        let mut by: HashMap<i64, Restriction> = HashMap::new();
        for (id, op, kind, sid) in rows {
            let r = by.entry(id).or_default();
            let s = subject(&kind, sid);
            if op == "read" {
                r.read.push(s)
            } else {
                r.update.push(s)
            }
        }
        let roots: Vec<i64> = by
            .into_iter()
            .filter(|(_, r)| {
                !perm::can_view(
                    &self.p,
                    &Grants {
                        all: true,
                        ..Default::default()
                    },
                    std::slice::from_ref(r),
                )
            })
            .map(|(id, _)| id)
            .collect();
        if roots.is_empty() {
            return Ok(roots);
        }
        sqlx::query_scalar(
            "WITH RECURSIVE d AS (SELECT id FROM content WHERE id = ANY($1) \
             UNION SELECT c.id FROM d JOIN content c ON c.parent_id = d.id \
             OR EXISTS (SELECT 1 FROM attachments a WHERE a.content_id = c.id AND a.container_id = d.id) \
             OR EXISTS (SELECT 1 FROM comments k WHERE k.content_id = c.id AND k.container_id = d.id)) \
             SELECT id FROM d",
        )
        .bind(roots)
        .fetch_all(db)
        .await
        .map_err(|e| db_err(e, ""))
    }

    /// Restrictions of the ancestors and the page itself, root first.
    pub async fn chain(&self, db: &PgPool, page_id: i64) -> Res<Vec<Restriction>> {
        chain(db, page_id).await
    }

    /// 404 unless the caller may see the content; the target is returned for further checks.
    pub async fn visible(&self, db: &PgPool, id: i64) -> Res<(Target, Vec<Restriction>)> {
        let t = target(db, id).await?;
        let chain = self.chain(db, t.page_id).await?;
        if !perm::can_view(&self.p, &self.grants(t.space_id), &chain) {
            return Err(ApiError::NotFound("content not found".into()));
        }
        Ok((t, chain))
    }

    pub fn require_space(&self, space: i64, key: &str, target: &str) -> Res<()> {
        let g = self.grants(space);
        if !g.has("read", "space") {
            return Err(ApiError::NotFound("space not found".into()));
        }
        if !g.has(key, target) {
            return Err(ApiError::Forbidden(format!(
                "requires the {key} {target} permission"
            )));
        }
        Ok(())
    }
}

pub async fn chain<'e>(db: impl sqlx::PgExecutor<'e>, page_id: i64) -> Res<Vec<Restriction>> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(i64, i32, Option<String>, Option<String>, Option<i64>)> = sqlx::query_as(
        "WITH RECURSIVE a AS (SELECT id, parent_id, 0 AS d FROM content WHERE id = $1 \
         UNION ALL SELECT c.id, c.parent_id, a.d + 1 FROM content c JOIN a ON c.id = a.parent_id) \
         SELECT a.id, a.d, r.operation, r.subject_type, r.subject_id FROM a \
         LEFT JOIN content_restrictions r ON r.content_id = a.id ORDER BY a.d DESC, a.id",
    )
    .bind(page_id)
    .fetch_all(db)
    .await
    .map_err(|e| db_err(e, ""))?;
    let mut out: Vec<(i64, Restriction)> = Vec::new();
    for (id, _, op, kind, sid) in rows {
        if out.last().map(|(i, _)| *i) != Some(id) {
            out.push((id, Restriction::default()));
        }
        if let (Some(op), Some(kind), Some(sid)) = (op, kind, sid) {
            let r = &mut out.last_mut().unwrap().1;
            let s = subject(&kind, sid);
            if op == "read" {
                r.read.push(s)
            } else {
                r.update.push(s)
            }
        }
    }
    Ok(out.into_iter().map(|(_, r)| r).collect())
}

/// Space, type, author and governing page of any content row (comments and attachments follow their container).
pub async fn target(db: &PgPool, id: i64) -> Res<Target> {
    let row = |id: i64| async move {
        sqlx::query_as::<_, (i64, String, i64, Option<i64>)>(
            "SELECT c.space_id, c.type, c.author_id, COALESCE(cm.container_id, at.container_id) FROM content c \
             LEFT JOIN comments cm ON cm.content_id = c.id LEFT JOIN attachments at ON at.content_id = c.id \
             WHERE c.id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("content not found".into()))
    };
    let (space_id, ty, author_id, mut up) = row(id).await?;
    let mut page_id = id;
    // comment on an attachment on a page: at most two hops
    for _ in 0..3 {
        let Some(parent) = up else { break };
        page_id = parent;
        up = row(parent).await?.3;
    }
    Ok(Target {
        space_id,
        ty,
        author_id,
        page_id,
    })
}

pub async fn space_by_key(db: &PgPool, key: &str) -> Res<i64> {
    sqlx::query_scalar("SELECT id FROM spaces WHERE upper(key) = upper($1)")
        .bind(key)
        .fetch_optional(db)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("space not found".into()))
}

/// Operation a request needs on the content it names, by method and route shape.
fn needed(ty: &str, method: &Method, route: &str) -> &'static str {
    let read = *method == Method::GET || *method == Method::HEAD;
    if read
        || route.contains("/like")
        || route.contains("/restriction")
        || route.contains("/permission")
        || route.contains("/watch")
        || route.contains("/notification")
    {
        return "read";
    }
    let exact = route.ends_with("/{id}");
    match ty {
        "comment" | "attachment" => {
            if *method == Method::DELETE {
                "delete"
            } else {
                "update"
            }
        }
        _ if route.contains("/child/attachment") => "create_attachment",
        _ if exact && *method == Method::DELETE => "delete",
        _ => "update",
    }
}

fn denied(op: &str) -> ApiError {
    ApiError::Forbidden(format!("you do not have permission to {op} this content"))
}

async fn check(
    s: &AppState,
    access: &Access,
    route: &str,
    method: &Method,
    params: &[(String, String)],
) -> Res<()> {
    let raw = |k: &str| params.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    let is_space_route = route.starts_with("/spaces/") || route.starts_with("/space/");
    if is_space_route {
        let space = if let Some(key) = raw("key") {
            space_by_key(&s.db, key).await?
        } else {
            parse_id(raw("id").unwrap_or_default())?
        };
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM spaces WHERE id = $1")
            .bind(space)
            .fetch_optional(&s.db)
            .await
            .map_err(|e| db_err(e, ""))?;
        if exists.is_none() {
            return Ok(()); // the handler answers 404 itself
        }
        let write = !(*method == Method::GET || *method == Method::HEAD);
        return if write {
            access.require_space(space, "administer", "space")
        } else {
            access.require_space(space, "read", "space")
        };
    }
    let Some(id) = raw("id")
        .or_else(|| raw("container"))
        .or_else(|| raw("contentId"))
    else {
        return Ok(());
    };
    let Ok(mut id) = parse_attachment_id(id) else {
        return Ok(());
    };
    if route.starts_with("/tasks/") {
        // tasks are addressed by their own id; the page governs them
        let page: Option<i64> = sqlx::query_scalar("SELECT page_id FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(&s.db)
            .await
            .map_err(|e| db_err(e, ""))?;
        let Some(page) = page else { return Ok(()) };
        id = page;
    }
    let (t, chain) = match access.visible(&s.db, id).await {
        Ok(v) => v,
        Err(ApiError::NotFound(_)) => {
            // unknown ids fall through to the handler's own 404; hidden ones are indistinguishable
            return if target(&s.db, id).await.is_err() {
                Ok(())
            } else {
                Err(ApiError::NotFound("content not found".into()))
            };
        }
        Err(e) => return Err(e),
    };
    let op = needed(&t.ty, method, route);
    let g = access.grants(t.space_id);
    let ok = match t.ty.as_str() {
        // body edits are limited to the author by the handler; resolving needs only the right to comment
        "comment" if op == "update" => g.has("create", "comment"),
        "comment" => perm::comment_can(op, &access.p, &g, &chain, t.author_id == access.p.id),
        "attachment" => perm::attachment_can(op, &access.p, &g, &chain),
        "blogpost" => perm::content_can("blogpost", op, &access.p, &g, &chain),
        _ => perm::page_can(op, &access.p, &g, &chain),
    };
    if ok {
        Ok(())
    } else {
        Err(denied(op))
    }
}

pub async fn guard(State(s): State<AppState>, mut req: Request, next: Next) -> Response {
    let Some(user) = req.extensions().get::<UserId>().copied() else {
        return next.run(req).await;
    };
    let access = match Access::load(&s.db, user).await {
        Ok(a) => Arc::new(a),
        Err(e) => return E(e).into_response(),
    };
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let route = ["/wiki/api/v2", "/wiki/rest/api", "/wiki/download"]
        .iter()
        .find_map(|p| route.strip_prefix(p))
        .unwrap_or(&route)
        .to_string();
    let params: Vec<(String, String)> = {
        let (mut parts, body) = req.into_parts();
        let ps = <RawPathParams as axum::extract::FromRequestParts<()>>::from_request_parts(
            &mut parts,
            &(),
        )
        .await
        .map(|p| {
            p.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default();
        req = Request::from_parts(parts, body);
        ps
    };
    if let Err(e) = check(&s, &access, &route, req.method(), &params).await {
        return E(e).into_response();
    }
    req.extensions_mut().insert(access);
    next.run(req).await
}
