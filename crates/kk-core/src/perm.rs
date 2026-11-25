//! Permission rules. The server loads facts (grants, restriction rows, group
//! memberships) and these functions decide; nothing here touches IO.

use std::collections::HashSet;

/// Valid `(operation key, target)` pairs of a space permission.
pub const SPACE_PERMS: &[(&str, &str)] = &[
    ("read", "space"),
    ("create", "page"),
    ("create", "blogpost"),
    ("create", "comment"),
    ("create", "attachment"),
    ("delete", "page"),
    ("delete", "blogpost"),
    ("delete", "comment"),
    ("delete", "attachment"),
    ("delete", "space"),
    ("export", "space"),
    ("administer", "space"),
    ("archive", "page"),
    ("restrict_content", "space"),
];

/// Grants of a new space for the everyone group.
pub const DEFAULT_GRANTS: &[(&str, &str)] = &[
    ("read", "space"),
    ("create", "page"),
    ("create", "blogpost"),
    ("create", "comment"),
    ("create", "attachment"),
    ("delete", "comment"),
    ("delete", "attachment"),
];

/// Operations listed on a space response (a subset of the grantable pairs).
const SPACE_OPS: &[(&str, &str)] = &[
    ("read", "space"),
    ("create", "page"),
    ("create", "blogpost"),
    ("create", "comment"),
    ("create", "attachment"),
    ("delete", "space"),
    ("export", "space"),
    ("administer", "space"),
    ("archive", "page"),
    ("restrict_content", "space"),
];

const PAGE_OPS: &[(&str, &str)] = &[
    ("read", "page"),
    ("update", "page"),
    ("delete", "page"),
    ("copy", "page"),
    ("move", "page"),
    ("archive", "page"),
    ("restrict_content", "page"),
    ("create", "comment"),
    ("create", "attachment"),
];
const COMMENT_OPS: &[(&str, &str)] = &[
    ("read", "comment"),
    ("update", "comment"),
    ("delete", "comment"),
];
const ATTACHMENT_OPS: &[(&str, &str)] = &[
    ("read", "attachment"),
    ("update", "attachment"),
    ("delete", "attachment"),
];

pub fn valid_space_perm(key: &str, target: &str) -> bool {
    SPACE_PERMS.contains(&(key, target))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Subject {
    User(i64),
    Group(i64),
}

/// Who is asking. `groups` includes the everyone group.
#[derive(Clone, Debug, Default)]
pub struct Principal {
    pub id: i64,
    pub admin: bool,
    pub groups: HashSet<i64>,
}

impl Principal {
    pub fn is_in(&self, set: &[Subject]) -> bool {
        set.iter().any(|s| match *s {
            Subject::User(u) => u == self.id,
            Subject::Group(g) => self.groups.contains(&g),
        })
    }
}

/// Space permissions the principal holds in one space.
#[derive(Clone, Debug, Default)]
pub struct Grants {
    pub held: HashSet<(String, String)>,
    /// Site administrators hold everything.
    pub all: bool,
}

impl Grants {
    pub fn has(&self, key: &str, target: &str) -> bool {
        self.all
            || self.held.contains(&(key.into(), target.into()))
            || self.held.contains(&("administer".into(), "space".into()))
    }
}

/// Restriction sets of one content row; an empty set is no restriction.
#[derive(Clone, Debug, Default)]
pub struct Restriction {
    pub read: Vec<Subject>,
    pub update: Vec<Subject>,
}

impl Restriction {
    fn view_ok(&self, p: &Principal) -> bool {
        self.read.is_empty() || p.is_in(&self.read) || p.is_in(&self.update)
    }

    fn update_ok(&self, p: &Principal) -> bool {
        self.update.is_empty() || p.is_in(&self.update)
    }
}

/// `chain` is the restriction of every ancestor and the content itself, root first.
pub fn can_view(p: &Principal, g: &Grants, chain: &[Restriction]) -> bool {
    p.admin || (g.has("read", "space") && chain.iter().all(|r| r.view_ok(p)))
}

/// Only the content's own update restriction counts.
pub fn can_update(p: &Principal, g: &Grants, chain: &[Restriction]) -> bool {
    p.admin
        || (can_view(p, g, chain)
            && g.has("create", "page")
            && chain.last().is_none_or(|r| r.update_ok(p)))
}

/// Whether the principal may do `(op, target)` on a page.
pub fn page_can(op: &str, p: &Principal, g: &Grants, chain: &[Restriction]) -> bool {
    content_can("page", op, p, g, chain)
}

/// Like [`page_can`] for a content type that has its own create and delete permissions
/// (`page` or `blogpost`).
pub fn content_can(ty: &str, op: &str, p: &Principal, g: &Grants, chain: &[Restriction]) -> bool {
    if !can_view(p, g, chain) {
        return false;
    }
    let edit = chain.last().is_none_or(|r| r.update_ok(p)) || p.admin;
    match op {
        "read" => true,
        "update" | "move" => g.has("create", ty) && edit,
        "copy" => g.has("create", ty),
        "delete" => g.has("delete", ty) && edit,
        "archive" => g.has("archive", "page") && edit,
        "restrict_content" => g.has("restrict_content", "space") || can_update(p, g, chain),
        "create_comment" => g.has("create", "comment"),
        "create_attachment" => g.has("create", "attachment"),
        _ => false,
    }
}

pub fn page_ops(
    p: &Principal,
    g: &Grants,
    chain: &[Restriction],
) -> Vec<(&'static str, &'static str)> {
    content_ops("page", p, g, chain)
}

/// Operations on a `page` or `blogpost`; the page-targeted entries report `ty` as their target.
pub fn content_ops(
    ty: &'static str,
    p: &Principal,
    g: &Grants,
    chain: &[Restriction],
) -> Vec<(&'static str, &'static str)> {
    PAGE_OPS
        .iter()
        .filter(|(op, target)| {
            let key = if *op == "create" {
                format!("create_{target}")
            } else {
                (*op).into()
            };
            content_can(ty, &key, p, g, chain)
        })
        .map(|(op, target)| (*op, if *target == "page" { ty } else { *target }))
        .collect()
}

pub fn space_ops(g: &Grants) -> Vec<(&'static str, &'static str)> {
    SPACE_OPS
        .iter()
        .filter(|(k, t)| g.has(k, t))
        .copied()
        .collect()
}

pub fn comment_can(
    op: &str,
    p: &Principal,
    g: &Grants,
    chain: &[Restriction],
    author: bool,
) -> bool {
    can_view(p, g, chain)
        && match op {
            "read" => true,
            "update" => author && g.has("create", "comment"),
            "delete" => (author && g.has("delete", "comment")) || g.has("administer", "space"),
            _ => false,
        }
}

pub fn comment_ops(
    p: &Principal,
    g: &Grants,
    chain: &[Restriction],
    author: bool,
) -> Vec<(&'static str, &'static str)> {
    COMMENT_OPS
        .iter()
        .filter(|(op, _)| comment_can(op, p, g, chain, author))
        .copied()
        .collect()
}

pub fn attachment_can(op: &str, p: &Principal, g: &Grants, chain: &[Restriction]) -> bool {
    can_view(p, g, chain)
        && match op {
            "read" => true,
            "update" => g.has("create", "attachment"),
            "delete" => g.has("delete", "attachment"),
            _ => false,
        }
}

pub fn attachment_ops(
    p: &Principal,
    g: &Grants,
    chain: &[Restriction],
) -> Vec<(&'static str, &'static str)> {
    ATTACHMENT_OPS
        .iter()
        .filter(|(op, _)| attachment_can(op, p, g, chain))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: i64, groups: &[i64]) -> Principal {
        Principal {
            id,
            admin: false,
            groups: groups.iter().copied().collect(),
        }
    }

    fn grants(pairs: &[(&str, &str)]) -> Grants {
        Grants {
            held: pairs
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            all: false,
        }
    }

    fn restr(read: &[Subject], update: &[Subject]) -> Restriction {
        Restriction {
            read: read.to_vec(),
            update: update.to_vec(),
        }
    }

    #[test]
    fn no_read_grant_hides_everything() {
        let g = grants(&[("create", "page")]);
        assert!(!can_view(&user(1, &[]), &g, &[]));
        assert!(page_ops(&user(1, &[]), &g, &[]).is_empty());
    }

    #[test]
    fn read_restriction_is_cumulative_and_group_aware() {
        let g = grants(DEFAULT_GRANTS);
        let chain = [
            restr(&[Subject::Group(7)], &[]),
            restr(&[Subject::User(2)], &[]),
        ];
        assert!(!can_view(&user(1, &[7]), &g, &chain), "in parent set only");
        assert!(!can_view(&user(2, &[]), &g, &chain), "in child set only");
        assert!(can_view(&user(2, &[7]), &g, &chain));
        let admin = Principal {
            admin: true,
            ..user(9, &[])
        };
        assert!(can_view(&admin, &grants(&[]), &chain));
    }

    #[test]
    fn update_restriction_blocks_edit_but_not_view_and_is_not_inherited() {
        let g = grants(DEFAULT_GRANTS);
        let locked = [restr(&[], &[Subject::User(2)])];
        assert!(can_view(&user(1, &[]), &g, &locked));
        assert!(!page_can("update", &user(1, &[]), &g, &locked));
        assert!(!page_can("delete", &user(1, &[]), &g, &locked));
        assert!(page_can("update", &user(2, &[]), &g, &locked));
        let child = [restr(&[], &[Subject::User(2)]), Restriction::default()];
        assert!(page_can("update", &user(1, &[]), &g, &child));
    }

    #[test]
    fn update_subject_can_view_a_read_restricted_page() {
        let g = grants(DEFAULT_GRANTS);
        let chain = [restr(&[Subject::User(5)], &[Subject::User(2)])];
        assert!(can_view(&user(2, &[]), &g, &chain));
        assert!(!can_view(&user(3, &[]), &g, &chain));
    }

    #[test]
    fn administer_implies_everything_and_ops_follow_grants() {
        let admin_space = grants(&[("read", "space"), ("administer", "space")]);
        assert_eq!(space_ops(&admin_space).len(), SPACE_OPS.len());
        let g = grants(DEFAULT_GRANTS);
        let ops = page_ops(&user(1, &[]), &g, &[]);
        assert!(ops.contains(&("update", "page")) && !ops.contains(&("archive", "page")));
        assert!(ops.contains(&("create", "comment")));
        assert!(
            ops.contains(&("restrict_content", "page")),
            "editors may restrict"
        );
    }

    #[test]
    fn comments_edit_by_author_only() {
        let g = grants(DEFAULT_GRANTS);
        let p = user(1, &[]);
        assert!(comment_can("update", &p, &g, &[], true));
        assert!(!comment_can("update", &p, &g, &[], false));
        assert!(!comment_can("delete", &p, &g, &[], false));
        assert!(comment_can(
            "delete",
            &p,
            &grants(&[("read", "space"), ("administer", "space")]),
            &[],
            false
        ));
    }

    #[test]
    fn space_perm_pairs_are_validated() {
        assert!(valid_space_perm("restrict_content", "space"));
        assert!(!valid_space_perm("read", "page"));
    }
}
