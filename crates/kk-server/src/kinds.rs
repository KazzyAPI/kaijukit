//! The content types that share the generic read endpoints. Handlers take a `const K: u8` index
//! into this table.

pub struct Kind {
    /// `content.type` value.
    pub ty: &'static str,
    /// Path segment under `/wiki/api/v2`.
    pub seg: &'static str,
}

pub const PAGE: u8 = 0;
pub const BLOG: u8 = 4;
pub const EMBED: u8 = 8;
pub const CUSTOM: u8 = 9;

const KINDS: [Kind; 10] = [
    Kind {
        ty: "page",
        seg: "pages",
    },
    Kind {
        ty: "attachment",
        seg: "attachments",
    },
    Kind {
        ty: "comment",
        seg: "comments",
    },
    Kind {
        ty: "space",
        seg: "spaces",
    },
    Kind {
        ty: "blogpost",
        seg: "blogposts",
    },
    Kind {
        ty: "folder",
        seg: "folders",
    },
    Kind {
        ty: "whiteboard",
        seg: "whiteboards",
    },
    Kind {
        ty: "database",
        seg: "databases",
    },
    Kind {
        ty: "embed",
        seg: "embeds",
    },
    Kind {
        ty: "custom",
        seg: "custom-content",
    },
];

pub fn of(k: u8) -> &'static Kind {
    &KINDS[k as usize]
}

/// Types that sit in the content tree (children, ancestors, descendants).
pub const TREE_TYPES: [&str; 5] = ["page", "folder", "whiteboard", "database", "embed"];

/// `targetType` reported for a content type's own operations.
pub fn target_label(ty: &str) -> &'static str {
    match ty {
        "blogpost" => "blogpost",
        "folder" => "folder",
        "whiteboard" => "whiteboard",
        "database" => "database",
        "embed" => "embed",
        "custom" => "custom_content",
        _ => "page",
    }
}
