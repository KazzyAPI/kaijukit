//! Storage constructs beyond plain text blocks: macros (panels, expand, status, code, generic),
//! tasks, decisions, layouts, media, smart links, dates, mentions, emoji and styled marks.
//!
//! Every converter claims a construct only when it can reproduce it exactly; anything with
//! attributes we cannot map is left to the caller, which keeps it verbatim.

use std::cell::Cell;

use serde_json::{json, Map, Value};

use crate::{
    storage::{block_list, blocks, children, inline_list, inlines, raw_ext},
    xml::{esc, esc_attr, El, X},
    Mark, Node,
};

pub(crate) const MACRO_TYPE: &str = "kk.macro";
const PANELS: [(&str, &str); 4] = [
    ("info", "info"),
    ("note", "note"),
    ("warning", "warning"),
    ("tip", "success"),
];
const PANEL_TYPES: [&str; 5] = ["info", "note", "warning", "success", "error"];
const COLOURS: [(&str, &str); 6] = [
    ("Grey", "neutral"),
    ("Red", "red"),
    ("Yellow", "yellow"),
    ("Green", "green"),
    ("Blue", "blue"),
    ("Purple", "purple"),
];
const LAYOUTS: [(&str, &[f64]); 6] = [
    ("single", &[100.0]),
    ("two_equal", &[50.0, 50.0]),
    ("two_left_sidebar", &[66.66, 33.33]),
    ("two_right_sidebar", &[33.33, 66.66]),
    ("three_equal", &[33.33, 33.33, 33.33]),
    ("three_with_sidebars", &[25.0, 50.0, 25.0]),
];
const DAY_MS: i64 = 86_400_000;
const INDENT_PX: f64 = 30.0;

thread_local! {
    static TASK_ID: Cell<u32> = const { Cell::new(0) };
}

pub(crate) fn reset_task_ids() {
    TASK_ID.with(|c| c.set(0));
}

fn els(e: &El) -> impl Iterator<Item = &El> {
    e.kids.iter().filter_map(|k| match k {
        X::El(c) => Some(c),
        _ => None,
    })
}

fn only_attrs(e: &El, allowed: &[&str]) -> bool {
    e.attrs.iter().all(|(k, _)| allowed.contains(&k.as_str()))
}

fn blank(k: &X) -> bool {
    matches!(k, X::Text(t) if t.trim().is_empty())
}

fn keys_in(n: &Node, allowed: &[&str]) -> bool {
    n.attrs.as_ref().is_none_or(|a| {
        a.as_object()
            .is_some_and(|o| o.keys().all(|k| allowed.contains(&k.as_str())))
    })
}

fn attr<'a>(n: &'a Node, k: &str) -> Option<&'a str> {
    n.attrs.as_ref()?.get(k)?.as_str()
}

fn macro_id(n: &Node) -> String {
    attr(n, "localId")
        .map(|i| format!(" ac:macro-id=\"{}\"", esc_attr(i)))
        .unwrap_or_default()
}

fn with_id(node: Node, attrs: Map<String, Value>, id: Option<&str>) -> Node {
    let mut attrs = attrs;
    if let Some(i) = id {
        attrs.insert("localId".into(), i.into());
    }
    if attrs.is_empty() {
        node
    } else {
        node.attrs(Value::Object(attrs))
    }
}

fn plain_attrs(pairs: &[(&str, Option<String>)]) -> String {
    pairs
        .iter()
        .filter_map(|(k, v)| Some(format!(" {k}=\"{}\"", esc_attr(v.as_deref()?))))
        .collect()
}

fn cdata(s: &str) -> String {
    format!("<![CDATA[{}]]>", s.replace("]]>", "]]]]><![CDATA[>"))
}

struct Macro<'a> {
    name: &'a str,
    id: Option<&'a str>,
    schema: Option<&'a str>,
    params: Vec<(&'a str, String)>,
    rich: Option<&'a [X]>,
    plain: Option<String>,
}

impl Macro<'_> {
    fn param(&self, k: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(n, _)| *n == k)
            .map(|(_, v)| v.as_str())
    }

    fn only_params(&self, allowed: &[&str]) -> bool {
        self.params.iter().all(|(n, _)| allowed.contains(n))
    }
}

fn parse_macro(e: &El) -> Option<Macro<'_>> {
    if !only_attrs(e, &["ac:name", "ac:schema-version", "ac:macro-id"]) {
        return None;
    }
    let mut m = Macro {
        name: e.attr("ac:name")?,
        id: e.attr("ac:macro-id"),
        schema: e.attr("ac:schema-version"),
        params: vec![],
        rich: None,
        plain: None,
    };
    for k in &e.kids {
        match k {
            X::El(c) if c.name == "ac:parameter" && only_attrs(c, &["ac:name"]) => {
                if c.kids.iter().any(|x| matches!(x, X::El(_))) {
                    return None;
                }
                m.params.push((c.attr("ac:name")?, c.text()));
            }
            X::El(c) if c.name == "ac:rich-text-body" && m.rich.is_none() && m.plain.is_none() => {
                m.rich = Some(&c.kids);
            }
            X::El(c) if c.name == "ac:plain-text-body" && m.rich.is_none() && m.plain.is_none() => {
                m.plain = Some(c.text());
            }
            k if blank(k) => {}
            _ => return None,
        }
    }
    let mut names: Vec<_> = m.params.iter().map(|(n, _)| *n).collect();
    names.sort_unstable();
    names.dedup();
    (names.len() == m.params.len()).then_some(m)
}

fn macro_node(e: &El, inline: bool) -> Result<Node, String> {
    let ext = if inline {
        "inlineExtension"
    } else {
        "extension"
    };
    if !inline {
        if let Some(g) = media_group(e) {
            return Ok(g);
        }
    }
    let Some(m) = parse_macro(e) else {
        return Ok(raw_ext(e, ext));
    };
    let known = if inline { status(&m) } else { known_block(&m)? };
    if let Some(n) = known {
        return Ok(n);
    }
    if inline && (m.rich.is_some() || m.plain.is_some()) {
        return Ok(raw_ext(e, ext));
    }
    let mut params = Map::new();
    if !m.params.is_empty() {
        let p: Map<_, _> = m
            .params
            .iter()
            .map(|(k, v)| (k.to_string(), json!({"value": v})))
            .collect();
        params.insert("macroParams".into(), Value::Object(p));
    }
    if let Some(s) = m.schema {
        params.insert(
            "macroMetadata".into(),
            json!({"schemaVersion": {"value": s}}),
        );
    }
    let mut attrs = Map::new();
    attrs.insert("extensionType".into(), MACRO_TYPE.into());
    attrs.insert("extensionKey".into(), m.name.into());
    attrs.insert("parameters".into(), Value::Object(params));
    let mut node = Node::new(ext);
    if let Some(t) = &m.plain {
        attrs.insert("text".into(), t.as_str().into());
    }
    if let Some(r) = m.rich {
        node = Node::new("bodiedExtension").content(blocks(r)?);
    }
    Ok(with_id(node, attrs, m.id))
}

fn known_block(m: &Macro) -> Result<Option<Node>, String> {
    let name = m.name;
    if name == "code" && m.only_params(&["language"]) && m.rich.is_none() {
        let Some(body) = &m.plain else {
            return Ok(None);
        };
        let mut attrs = Map::new();
        if let Some(l) = m.param("language") {
            attrs.insert("language".into(), l.into());
        }
        if let Some(i) = m.id {
            attrs.insert("uniqueId".into(), i.into());
        }
        let mut node = Node::new("codeBlock");
        if !attrs.is_empty() {
            node = node.attrs(Value::Object(attrs));
        }
        let text = (!body.is_empty()).then(|| Node::text(body, &[]));
        return Ok(Some(node.content(text.into_iter().collect())));
    }
    let Some(rich) = m.rich.filter(|_| m.plain.is_none()) else {
        return Ok(None);
    };
    let panel = match name {
        "panel" if m.only_params(&["panelType"]) => m
            .param("panelType")
            .filter(|t| PANEL_TYPES.contains(t))
            .map(String::from),
        _ if m.params.is_empty() => PANELS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, t)| t.to_string()),
        _ => None,
    };
    if let Some(t) = panel {
        let mut a = Map::new();
        a.insert("panelType".into(), t.into());
        return Ok(Some(with_id(
            Node::new("panel").content(blocks(rich)?),
            a,
            m.id,
        )));
    }
    if name == "expand" && m.only_params(&["title"]) {
        let mut a = Map::new();
        if let Some(t) = m.param("title") {
            a.insert("title".into(), t.into());
        }
        return Ok(Some(with_id(
            Node::new("expand").content(blocks(rich)?),
            a,
            m.id,
        )));
    }
    Ok(None)
}

fn status(m: &Macro) -> Option<Node> {
    if m.name != "status" || !m.only_params(&["title", "colour", "subtle"]) {
        return None;
    }
    if m.rich.is_some() || m.plain.is_some() {
        return None;
    }
    let colour = m.param("colour")?;
    let (_, color) = COLOURS
        .iter()
        .find(|(c, _)| c.eq_ignore_ascii_case(colour))?;
    let mut a = Map::new();
    a.insert("text".into(), m.param("title").unwrap_or("").into());
    a.insert("color".into(), (*color).into());
    match m.param("subtle") {
        None | Some("false") => {}
        Some("true") => {
            a.insert("style".into(), "subtle".into());
        }
        Some(_) => return None,
    }
    Some(with_id(Node::new("status"), a, m.id))
}

fn media_ref(r: &El) -> Option<Node> {
    if !r.kids.is_empty() {
        return None;
    }
    let mut a = Map::new();
    match r.name.as_str() {
        "ri:attachment" if only_attrs(r, &["ri:filename", "kk:media-id", "kk:collection"]) => {
            a.insert("type".into(), "file".into());
            if let Some(i) = r.attr("kk:media-id") {
                a.insert("id".into(), i.into());
            }
            if let Some(c) = r.attr("kk:collection") {
                a.insert("collection".into(), c.into());
            }
            a.insert("__fileName".into(), r.attr("ri:filename")?.into());
        }
        "ri:url" if only_attrs(r, &["ri:value"]) => {
            a.insert("type".into(), "external".into());
            a.insert("url".into(), r.attr("ri:value")?.into());
        }
        _ => return None,
    }
    Some(Node::new("media").attrs(Value::Object(a)))
}

fn media_group(e: &El) -> Option<Node> {
    if e.attr("ac:name") != Some("view-file") || !only_attrs(e, &["ac:name", "ac:schema-version"]) {
        return None;
    }
    let mut files = vec![];
    for k in &e.kids {
        match k {
            X::El(p) if p.name == "ac:parameter" && p.attr("ac:name") == Some("name") => {
                let mut inner = els(p);
                let (r, none) = (inner.next()?, inner.next());
                let media =
                    media_ref(r).filter(|m| none.is_none() && attr(m, "type") == Some("file"))?;
                files.push(media);
            }
            k if blank(k) => {}
            _ => return None,
        }
    }
    (!files.is_empty()).then(|| Node::new("mediaGroup").content(files))
}

fn num(s: &str) -> Option<Value> {
    s.parse::<u64>().ok().map(Value::from)
}

fn media_single(img: &El) -> Option<Node> {
    let allowed = [
        "ac:layout",
        "ac:width",
        "ac:alt",
        "ac:original-width",
        "ac:original-height",
    ];
    if !only_attrs(img, &allowed) {
        return None;
    }
    let (mut media, mut caption) = (None, None);
    for k in &img.kids {
        match k {
            X::El(c) if c.name == "ac:caption" && caption.is_none() && only_attrs(c, &[]) => {
                let text = inlines(&c.kids, &[]).ok()?;
                caption = Some(Node::new("caption").content(text));
            }
            X::El(c) if media.is_none() => media = Some(media_ref(c)?),
            k if blank(k) => {}
            _ => return None,
        }
    }
    let mut media = media?;
    let mut a = media.attrs.take()?;
    if let Some(v) = img.attr("ac:alt") {
        a["alt"] = v.into();
    }
    for (from, to) in [
        ("ac:original-width", "width"),
        ("ac:original-height", "height"),
    ] {
        if let Some(v) = img.attr(from) {
            a[to] = num(v)?;
        }
    }
    media.attrs = Some(a);
    let mut s = Map::new();
    if let Some(l) = img.attr("ac:layout") {
        s.insert("layout".into(), l.into());
    }
    if let Some(w) = img.attr("ac:width") {
        s.insert("width".into(), num(w)?);
    }
    let node = Node::new("mediaSingle").content(std::iter::once(media).chain(caption).collect());
    Some(with_id(node, s, None))
}

/// `<p>` holding only an image or a block-style smart link is a media single / block card.
pub(crate) fn sole(p: &El) -> Option<Node> {
    if !only_attrs(p, &[]) {
        return None;
    }
    let mut it = p.kids.iter().filter(|k| !blank(k));
    let X::El(only) = it.next()? else { return None };
    if it.next().is_some() {
        return None;
    }
    match only.name.as_str() {
        "ac:image" => media_single(only),
        "a" if card(only) == Some("block") => {
            Some(Node::new("blockCard").attrs(json!({"url": only.attr("href")?})))
        }
        _ => None,
    }
}

fn card(a: &El) -> Option<&str> {
    only_attrs(a, &["href", "data-card-appearance"])
        .then(|| a.attr("data-card-appearance"))
        .flatten()
}

fn layout(e: &El) -> Option<Vec<Node>> {
    if !only_attrs(e, &[]) {
        return None;
    }
    let mut out = vec![];
    for k in &e.kids {
        match k {
            X::El(s) if s.name == "ac:layout-section" && only_attrs(s, &["ac:type"]) => {
                let (_, widths) = LAYOUTS
                    .iter()
                    .find(|(t, _)| Some(*t) == s.attr("ac:type"))?;
                let cells: Vec<_> = els(s).collect();
                let ok = cells.len() == widths.len()
                    && cells
                        .iter()
                        .all(|c| c.name == "ac:layout-cell" && only_attrs(c, &[]));
                if !ok {
                    return None;
                }
                let cols = cells
                    .iter()
                    .zip(*widths)
                    .map(|(c, w)| {
                        let body = blocks(&c.kids).ok()?;
                        Some(
                            Node::new("layoutColumn")
                                .attrs(json!({"width": w}))
                                .content(body),
                        )
                    })
                    .collect::<Option<Vec<_>>>()?;
                out.push(Node::new("layoutSection").content(cols));
            }
            k if blank(k) => {}
            _ => return None,
        }
    }
    Some(out)
}

fn text_of(e: &El) -> Option<String> {
    (!e.kids.iter().any(|k| matches!(k, X::El(_)))).then(|| e.text())
}

fn task_list(e: &El) -> Option<Node> {
    if !only_attrs(e, &["kk:local-id"]) {
        return None;
    }
    let mut items = vec![];
    for k in &e.kids {
        match k {
            X::El(t) if t.name == "ac:task" && only_attrs(t, &[]) => {
                let (mut uuid, mut state, mut body) = (None, None, None);
                for c in &t.kids {
                    match c {
                        X::El(c) if c.name == "ac:task-id" => {}
                        X::El(c) if c.name == "ac:task-uuid" => uuid = text_of(c),
                        X::El(c) if c.name == "ac:task-status" => {
                            state = match text_of(c)?.trim() {
                                "incomplete" => Some("TODO"),
                                "complete" => Some("DONE"),
                                _ => return None,
                            };
                        }
                        X::El(c) if c.name == "ac:task-body" && only_attrs(c, &[]) => {
                            body = Some(inlines(&c.kids, &[]).ok()?);
                        }
                        c if blank(c) => {}
                        _ => return None,
                    }
                }
                let mut a = Map::new();
                if let Some(u) = uuid {
                    a.insert("localId".into(), u.into());
                }
                a.insert("state".into(), state?.into());
                items.push(Node::new("taskItem").attrs(Value::Object(a)).content(body?));
            }
            k if blank(k) => {}
            _ => return None,
        }
    }
    let mut a = Map::new();
    if let Some(i) = e.attr("kk:local-id") {
        a.insert("localId".into(), i.into());
    }
    Some(with_id(Node::new("taskList").content(items), a, None))
}

fn adf_attr(e: &El, key: &str) -> Option<String> {
    els(e)
        .find(|c| c.name == "ac:adf-attribute" && c.attr("key") == Some(key))
        .and_then(text_of)
}

fn decision_list(e: &El) -> Option<Node> {
    let mut nodes = els(e).filter(|c| c.name != "ac:adf-fallback");
    let list = nodes
        .next()
        .filter(|l| l.name == "ac:adf-node" && l.attr("type") == Some("decision-list"))?;
    if nodes.next().is_some() {
        return None;
    }
    let mut items = vec![];
    for c in els(list) {
        match c.name.as_str() {
            "ac:adf-attribute" if c.attr("key") == Some("local-id") => {}
            "ac:adf-node" if c.attr("type") == Some("decision-item") => {
                let content = els(c).find(|x| x.name == "ac:adf-content")?;
                let mut a = Map::new();
                if let Some(i) = adf_attr(c, "local-id") {
                    a.insert("localId".into(), i.into());
                }
                a.insert("state".into(), adf_attr(c, "state")?.into());
                let body = inlines(&content.kids, &[]).ok()?;
                items.push(
                    Node::new("decisionItem")
                        .attrs(Value::Object(a))
                        .content(body),
                );
            }
            _ => return None,
        }
    }
    let mut a = Map::new();
    if let Some(i) = adf_attr(list, "local-id") {
        a.insert("localId".into(), i.into());
    }
    Some(with_id(Node::new("decisionList").content(items), a, None))
}

/// Block-level storage elements this module owns; `None` leaves the element to the caller.
pub(crate) fn block(e: &El) -> Result<Option<Vec<Node>>, String> {
    Ok(match e.name.as_str() {
        "ac:structured-macro" => Some(vec![macro_node(e, false)?]),
        "ac:layout" => layout(e),
        "ac:task-list" => task_list(e).map(|n| vec![n]),
        "ac:adf-extension" => decision_list(e).map(|n| vec![n]),
        _ => None,
    })
}

fn civil_to_days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

fn days_to_civil(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn parse_day(s: &str) -> Option<i64> {
    let mut p = s.split('-');
    let (y, m, d) = (p.next()?, p.next()?, p.next()?);
    if p.next().is_some() || y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return None;
    }
    let (y, m, d) = (y.parse().ok()?, m.parse().ok()?, d.parse().ok()?);
    let days = civil_to_days(y, m, d);
    (days_to_civil(days) == (y, m, d)).then_some(days)
}

fn style_decls(e: &El) -> Option<Vec<(&str, &str)>> {
    e.attr("style")?
        .split(';')
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(|d| d.split_once(':').map(|(k, v)| (k.trim(), v.trim())))
        .collect()
}

fn span_marks(e: &El) -> Option<Vec<Mark>> {
    if !only_attrs(e, &["style"]) {
        return None;
    }
    style_decls(e)?
        .into_iter()
        .map(|(k, v)| {
            let kind = match k {
                "color" => "textColor",
                "background-color" => "backgroundColor",
                _ => return None,
            };
            Some(Mark::new(kind, Some(json!({"color": v}))))
        })
        .collect()
}

fn mention(e: &El) -> Option<Node> {
    if !only_attrs(e, &[]) {
        return None;
    }
    let (mut id, mut text) = (None, None);
    for k in &e.kids {
        match k {
            X::El(u)
                if u.name == "ri:user" && id.is_none() && only_attrs(u, &["ri:account-id"]) =>
            {
                id = Some(u.attr("ri:account-id")?);
            }
            X::El(b) if b.name == "ac:plain-text-link-body" && text.is_none() => {
                text = Some(b.text());
            }
            k if blank(k) => {}
            _ => return None,
        }
    }
    let mut a = Map::new();
    a.insert("id".into(), id?.into());
    if let Some(t) = text {
        a.insert("text".into(), t.into());
    }
    Some(Node::new("mention").attrs(Value::Object(a)))
}

fn emoji(e: &El) -> Option<Node> {
    let allowed = [
        "ac:name",
        "ac:emoji-shortname",
        "ac:emoji-id",
        "ac:emoji-fallback",
    ];
    if !only_attrs(e, &allowed) || !e.kids.is_empty() {
        return None;
    }
    let short = e
        .attr("ac:emoji-shortname")
        .map(String::from)
        .or_else(|| e.attr("ac:name").map(|n| format!(":{n}:")))?;
    let mut a = Map::new();
    a.insert("shortName".into(), short.into());
    for (from, to) in [("ac:emoji-id", "id"), ("ac:emoji-fallback", "text")] {
        if let Some(v) = e.attr(from) {
            a.insert(to.into(), v.into());
        }
    }
    Some(Node::new("emoji").attrs(Value::Object(a)))
}

/// Inline storage elements this module owns (nodes and mark-producing wrappers).
pub(crate) fn inline(e: &El, marks: &[Mark]) -> Result<Option<Vec<Node>>, String> {
    let wrap = |m: Mark| -> Result<Option<Vec<Node>>, String> {
        inlines(&e.kids, &[marks, &[m]].concat()).map(Some)
    };
    Ok(match e.name.as_str() {
        "ac:structured-macro" => Some(vec![macro_node(e, true)?]),
        "u" | "ins" => return wrap(Mark::new("underline", None)),
        "s" | "del" | "strike" => return wrap(Mark::new("strike", None)),
        "sub" | "sup" => return wrap(Mark::new("subsup", Some(json!({"type": e.name})))),
        "span" => match span_marks(e) {
            Some(ms) => inlines(&e.kids, &[marks, &ms].concat()).map(Some)?,
            None => None,
        },
        "ac:inline-comment-marker" if only_attrs(e, &["ac:ref"]) && e.attr("ac:ref").is_some() => {
            let id = e.attr("ac:ref");
            return wrap(Mark::new(
                "annotation",
                Some(json!({"id": id, "annotationType": "inlineComment"})),
            ));
        }
        "a" if card(e) == Some("inline") => Some(vec![
            Node::new("inlineCard").attrs(json!({"url": e.attr("href").unwrap_or("")}))
        ]),
        "time" if only_attrs(e, &["datetime"]) && e.kids.is_empty() => {
            let days = e.attr("datetime").and_then(parse_day);
            days.map(|d| {
                vec![Node::new("date").attrs(json!({"timestamp": (d * DAY_MS).to_string()}))]
            })
        }
        "ac:link" => mention(e).map(|n| vec![n]),
        "ac:emoticon" => emoji(e).map(|n| vec![n]),
        _ => None,
    })
}

fn style_marks(e: &El) -> Vec<Mark> {
    let mut marks = vec![];
    for (k, v) in style_decls(e).unwrap_or_default() {
        match k {
            "text-align" if matches!(v, "center" | "right" | "end") => {
                let align = if v == "center" { "center" } else { "end" };
                marks.insert(0, Mark::new("alignment", Some(json!({"align": align}))));
            }
            "margin-left" => {
                let px = v
                    .strip_suffix("px")
                    .and_then(|n| n.trim().parse::<f64>().ok());
                if let Some(level) = px
                    .map(|p| p / INDENT_PX)
                    .filter(|l| l.fract() == 0.0 && *l >= 1.0)
                {
                    marks.push(Mark::new(
                        "indentation",
                        Some(json!({"level": level as u64})),
                    ));
                }
            }
            _ => {}
        }
    }
    marks
}

/// Alignment and indentation of a paragraph or heading from its `style` attribute.
pub(crate) fn block_marks(e: &El) -> Option<Vec<Mark>> {
    let m = style_marks(e);
    (!m.is_empty()).then_some(m)
}

/// ` style="..."` for paragraph/heading marks; `None` when a mark has no storage form.
pub(crate) fn block_style(n: &Node) -> Option<String> {
    let mut css = String::new();
    for m in n.marks.as_deref().unwrap_or(&[]) {
        let a = m.attrs.as_ref()?;
        match m.kind.as_str() {
            "alignment" => match a["align"].as_str()? {
                "center" => css.push_str("text-align: center; "),
                "end" => css.push_str("text-align: right; "),
                _ => return None,
            },
            "indentation" => {
                let level = a["level"].as_u64().filter(|l| (1..=6).contains(l))?;
                css.push_str(&format!("margin-left: {}px; ", level * INDENT_PX as u64));
            }
            _ => return None,
        }
    }
    Some(if css.is_empty() {
        css
    } else {
        format!(" style=\"{}\"", css.trim_end())
    })
}

fn color_ok(c: &str) -> bool {
    !c.is_empty()
        && c.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "#(),.% -".contains(ch))
}

/// Opening and closing storage tags for a text mark; `None` when it cannot be expressed.
pub(crate) fn mark_tags(m: &Mark) -> Option<(String, &'static str)> {
    let a = m.attrs.as_ref();
    let attrs_only = |keys: &[&str]| {
        a.and_then(Value::as_object)
            .is_some_and(|o| o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k)))
    };
    let s = |k: &str| a.and_then(|a| a[k].as_str());
    Some(match m.kind.as_str() {
        "strong" if a.is_none() => ("<strong>".into(), "</strong>"),
        "em" if a.is_none() => ("<em>".into(), "</em>"),
        "code" if a.is_none() => ("<code>".into(), "</code>"),
        "strike" if a.is_none() => ("<s>".into(), "</s>"),
        "underline" if a.is_none() => ("<u>".into(), "</u>"),
        "link" if attrs_only(&["href"]) => {
            (format!("<a href=\"{}\">", esc_attr(s("href")?)), "</a>")
        }
        "textColor" | "backgroundColor" if attrs_only(&["color"]) && color_ok(s("color")?) => {
            let prop = if m.kind == "textColor" {
                "color"
            } else {
                "background-color"
            };
            (
                format!("<span style=\"{prop}: {};\">", esc_attr(s("color")?)),
                "</span>",
            )
        }
        "subsup" if attrs_only(&["type"]) => match s("type")? {
            "sub" => ("<sub>".into(), "</sub>"),
            "sup" => ("<sup>".into(), "</sup>"),
            _ => return None,
        },
        "annotation"
            if attrs_only(&["id", "annotationType"])
                && s("annotationType") == Some("inlineComment") =>
        {
            (
                format!(
                    "<ac:inline-comment-marker ac:ref=\"{}\">",
                    esc_attr(s("id")?)
                ),
                "</ac:inline-comment-marker>",
            )
        }
        _ => return None,
    })
}

fn macro_open(name: &str, id: &str) -> String {
    format!("<ac:structured-macro ac:name=\"{name}\" ac:schema-version=\"1\"{id}>")
}

fn param(name: &str, value: &str) -> String {
    format!(
        "<ac:parameter ac:name=\"{name}\">{}</ac:parameter>",
        esc(value)
    )
}

fn panel_out(n: &Node) -> Option<String> {
    if !keys_in(n, &["panelType", "localId"]) {
        return None;
    }
    let t = attr(n, "panelType")?;
    let body = format!(
        "<ac:rich-text-body>{}</ac:rich-text-body>",
        block_list(children(n))
    );
    let open = match PANELS.iter().find(|(_, p)| *p == t) {
        Some((name, _)) => macro_open(name, &macro_id(n)),
        None if t == "error" => macro_open("panel", &macro_id(n)) + &param("panelType", t),
        None => return None,
    };
    Some(format!("{open}{body}</ac:structured-macro>"))
}

fn expand_out(n: &Node) -> Option<String> {
    if !keys_in(n, &["title", "localId"]) {
        return None;
    }
    let title = attr(n, "title")
        .map(|t| param("title", t))
        .unwrap_or_default();
    Some(format!(
        "{}{title}<ac:rich-text-body>{}</ac:rich-text-body></ac:structured-macro>",
        macro_open("expand", &macro_id(n)),
        block_list(children(n))
    ))
}

fn task_list_out(n: &Node) -> Option<String> {
    if !keys_in(n, &["localId"]) {
        return None;
    }
    let mut items = String::new();
    for t in children(n) {
        let state = match attr(t, "state")? {
            "TODO" => "incomplete",
            "DONE" => "complete",
            _ => return None,
        };
        if t.kind != "taskItem" || t.marks.is_some() || !keys_in(t, &["localId", "state"]) {
            return None;
        }
        let id = TASK_ID.with(|c| {
            c.set(c.get() + 1);
            c.get()
        });
        let uuid = attr(t, "localId")
            .map(|u| format!("<ac:task-uuid>{}</ac:task-uuid>", esc(u)))
            .unwrap_or_default();
        items.push_str(&format!(
            "<ac:task><ac:task-id>{id}</ac:task-id>{uuid}<ac:task-status>{state}</ac:task-status>\
             <ac:task-body>{}</ac:task-body></ac:task>",
            inline_list(children(t))
        ));
    }
    let local = plain_attrs(&[("kk:local-id", attr(n, "localId").map(String::from))]);
    Some(format!("<ac:task-list{local}>{items}</ac:task-list>"))
}

fn adf_attribute(key: &str, value: Option<&str>) -> String {
    value
        .map(|v| {
            format!(
                "<ac:adf-attribute key=\"{key}\">{}</ac:adf-attribute>",
                esc(v)
            )
        })
        .unwrap_or_default()
}

fn decision_out(n: &Node) -> Option<String> {
    if !keys_in(n, &["localId"]) {
        return None;
    }
    let mut items = String::new();
    for d in children(n) {
        if d.kind != "decisionItem" || d.marks.is_some() || !keys_in(d, &["localId", "state"]) {
            return None;
        }
        items.push_str(&format!(
            "<ac:adf-node type=\"decision-item\">{}{}<ac:adf-content>{}</ac:adf-content></ac:adf-node>",
            adf_attribute("local-id", attr(d, "localId")),
            adf_attribute("state", Some(attr(d, "state")?)),
            inline_list(children(d))
        ));
    }
    Some(format!(
        "<ac:adf-extension><ac:adf-node type=\"decision-list\">{}{items}</ac:adf-node></ac:adf-extension>",
        adf_attribute("local-id", attr(n, "localId"))
    ))
}

fn media_ri(m: &Node, allowed: &[&str]) -> Option<String> {
    if m.kind != "media" || m.marks.is_some() || m.content.is_some() || !keys_in(m, allowed) {
        return None;
    }
    match attr(m, "type")? {
        "file" => Some(format!(
            "<ri:attachment{}/>",
            plain_attrs(&[
                ("ri:filename", Some(attr(m, "__fileName")?.into())),
                ("kk:media-id", attr(m, "id").map(String::from)),
                ("kk:collection", attr(m, "collection").map(String::from)),
            ])
        )),
        "external" => Some(format!(
            "<ri:url{}/>",
            plain_attrs(&[("ri:value", Some(attr(m, "url")?.into()))])
        )),
        _ => None,
    }
}

fn media_single_out(n: &Node) -> Option<String> {
    if !keys_in(n, &["layout", "width"]) {
        return None;
    }
    let (m, caption) = match children(n) {
        [m] => (m, String::new()),
        [m, c] if c.kind == "caption" && c.attrs.is_none() && c.marks.is_none() => (
            m,
            format!("<ac:caption>{}</ac:caption>", inline_list(children(c))),
        ),
        _ => return None,
    };
    let ri = media_ri(
        m,
        &[
            "type",
            "id",
            "collection",
            "url",
            "width",
            "height",
            "alt",
            "__fileName",
        ],
    )?;
    let int = |n: &Node, k: &str| -> Option<Option<String>> {
        match n.attrs.as_ref().and_then(|a| a.get(k)) {
            None => Some(None),
            Some(v) => Some(Some(v.as_u64()?.to_string())),
        }
    };
    let (width, ow, oh) = (int(n, "width")?, int(m, "width")?, int(m, "height")?);
    let attrs = plain_attrs(&[
        ("ac:layout", attr(n, "layout").map(String::from)),
        ("ac:width", width),
        ("ac:alt", attr(m, "alt").map(String::from)),
        ("ac:original-width", ow),
        ("ac:original-height", oh),
    ]);
    Some(format!("<p><ac:image{attrs}>{ri}{caption}</ac:image></p>"))
}

fn media_group_out(n: &Node) -> Option<String> {
    if !keys_in(n, &[]) || children(n).is_empty() {
        return None;
    }
    let files = children(n)
        .iter()
        .map(|m| {
            let ri = media_ri(m, &["type", "id", "collection", "__fileName"])?;
            (attr(m, "type") == Some("file")).then(|| param_raw("name", &ri))
        })
        .collect::<Option<String>>()?;
    Some(format!(
        "{}{files}</ac:structured-macro>",
        macro_open("view-file", "")
    ))
}

fn param_raw(name: &str, xml: &str) -> String {
    format!("<ac:parameter ac:name=\"{name}\">{xml}</ac:parameter>")
}

fn layout_out(n: &Node) -> Option<String> {
    if !keys_in(n, &[]) {
        return None;
    }
    let cols = children(n);
    let widths = cols
        .iter()
        .map(|c| {
            (c.kind == "layoutColumn" && c.marks.is_none() && keys_in(c, &["width"]))
                .then(|| c.attrs.as_ref()?.get("width")?.as_f64())
                .flatten()
        })
        .collect::<Option<Vec<_>>>()?;
    let (kind, _) = LAYOUTS.iter().find(|(_, w)| {
        w.len() == widths.len() && w.iter().zip(&widths).all(|(a, b)| (a - b).abs() < 0.005)
    })?;
    let cells: String = cols
        .iter()
        .map(|c| {
            format!(
                "<ac:layout-cell>{}</ac:layout-cell>",
                block_list(children(c))
            )
        })
        .collect();
    Some(format!(
        "<ac:layout><ac:layout-section ac:type=\"{kind}\">{cells}</ac:layout-section></ac:layout>"
    ))
}

fn macro_out(n: &Node) -> Option<String> {
    let allowed = [
        "extensionType",
        "extensionKey",
        "parameters",
        "text",
        "localId",
    ];
    if attr(n, "extensionType") != Some(MACRO_TYPE) || !keys_in(n, &allowed) {
        return None;
    }
    let obj = |v: Option<&Value>, keys: &[&str]| -> Option<Map<String, Value>> {
        match v {
            None => Some(Map::new()),
            Some(Value::Object(o)) if o.keys().all(|k| keys.contains(&k.as_str())) => {
                Some(o.clone())
            }
            _ => None,
        }
    };
    let parameters = obj(
        n.attrs.as_ref()?.get("parameters"),
        &["macroParams", "macroMetadata"],
    )?;
    let meta = obj(parameters.get("macroMetadata"), &["schemaVersion"])?;
    let value = |v: &Value| -> Option<String> {
        let o = v.as_object().filter(|o| o.len() == 1)?;
        Some(o.get("value")?.as_str()?.to_string())
    };
    let schema = match meta.get("schemaVersion") {
        None => None,
        Some(v) => Some(value(v)?),
    };
    let mut open = format!(
        "<ac:structured-macro ac:name=\"{}\"",
        esc_attr(attr(n, "extensionKey")?)
    );
    open.push_str(&plain_attrs(&[("ac:schema-version", schema)]));
    open.push_str(&macro_id(n));
    open.push('>');
    let params = match parameters.get("macroParams") {
        None => Map::new(),
        Some(Value::Object(o)) => o.clone(),
        Some(_) => return None,
    };
    for (k, v) in params {
        open.push_str(&param(&esc_attr(&k), &value(&v)?));
    }
    let text = attr(n, "text");
    let body = match (n.kind.as_str(), text, n.content.is_some()) {
        ("bodiedExtension", None, true) => {
            format!(
                "<ac:rich-text-body>{}</ac:rich-text-body>",
                block_list(children(n))
            )
        }
        ("extension" | "inlineExtension", Some(t), false) => {
            format!("<ac:plain-text-body>{}</ac:plain-text-body>", cdata(t))
        }
        ("extension" | "inlineExtension", None, false) => String::new(),
        _ => return None,
    };
    Some(format!("{open}{body}</ac:structured-macro>"))
}

fn code_out(n: &Node) -> Option<String> {
    if !keys_in(n, &["language", "uniqueId"]) {
        return None;
    }
    let lang = attr(n, "language")
        .map(|l| param("language", l))
        .unwrap_or_default();
    let body: String = children(n)
        .iter()
        .filter_map(|t| t.text.as_deref())
        .collect();
    Some(format!(
        "{}{lang}<ac:plain-text-body>{}</ac:plain-text-body></ac:structured-macro>",
        macro_open(
            "code",
            &attr(n, "uniqueId")
                .map(|i| format!(" ac:macro-id=\"{}\"", esc_attr(i)))
                .unwrap_or_default()
        ),
        cdata(&body)
    ))
}

pub(crate) fn block_out(n: &Node) -> Option<String> {
    match n.kind.as_str() {
        "codeBlock" => code_out(n),
        "panel" => panel_out(n),
        "expand" | "nestedExpand" => expand_out(n),
        "taskList" => task_list_out(n),
        "decisionList" => decision_out(n),
        "mediaSingle" => media_single_out(n),
        "mediaGroup" => media_group_out(n),
        "layoutSection" => layout_out(n),
        "blockCard" => {
            let u = attr(n, "url").filter(|_| keys_in(n, &["url"]))?;
            Some(format!(
                "<p><a href=\"{0}\" data-card-appearance=\"block\">{0}</a></p>",
                esc_attr(u)
            ))
        }
        "extension" | "bodiedExtension" => macro_out(n),
        _ => None,
    }
}

pub(crate) fn inline_out(n: &Node) -> Option<String> {
    if n.marks.is_some() {
        return None;
    }
    match n.kind.as_str() {
        "status" => {
            if !keys_in(n, &["text", "color", "localId", "style"]) {
                return None;
            }
            let (colour, _) = COLOURS.iter().find(|(_, c)| Some(*c) == attr(n, "color"))?;
            let subtle = match attr(n, "style") {
                None => String::new(),
                Some("subtle") => param("subtle", "true"),
                Some(_) => return None,
            };
            Some(format!(
                "{}{}{}{subtle}</ac:structured-macro>",
                macro_open("status", &macro_id(n)),
                param("title", attr(n, "text")?),
                param("colour", colour)
            ))
        }
        "date" => {
            let ms: i64 = attr(n, "timestamp")
                .filter(|_| keys_in(n, &["timestamp"]))?
                .parse()
                .ok()?;
            let (y, m, d) = days_to_civil((ms % DAY_MS == 0).then_some(ms / DAY_MS)?);
            Some(format!("<time datetime=\"{y:04}-{m:02}-{d:02}\" />"))
        }
        "mention" => {
            if !keys_in(n, &["id", "text"]) {
                return None;
            }
            let body = attr(n, "text")
                .map(|t| {
                    format!(
                        "<ac:plain-text-link-body>{}</ac:plain-text-link-body>",
                        cdata(t)
                    )
                })
                .unwrap_or_default();
            Some(format!(
                "<ac:link><ri:user ri:account-id=\"{}\"/>{body}</ac:link>",
                esc_attr(attr(n, "id")?)
            ))
        }
        "emoji" => {
            if !keys_in(n, &["shortName", "id", "text"]) {
                return None;
            }
            let short = attr(n, "shortName")?;
            let attrs = plain_attrs(&[
                ("ac:name", Some(short.trim_matches(':').into())),
                ("ac:emoji-shortname", Some(short.into())),
                ("ac:emoji-id", attr(n, "id").map(String::from)),
                ("ac:emoji-fallback", attr(n, "text").map(String::from)),
            ]);
            Some(format!("<ac:emoticon{attrs}/>"))
        }
        "inlineCard" => {
            let u = attr(n, "url").filter(|_| keys_in(n, &["url"]))?;
            Some(format!(
                "<a href=\"{0}\" data-card-appearance=\"inline\">{0}</a>",
                esc_attr(u)
            ))
        }
        "inlineExtension" => macro_out(n),
        _ => None,
    }
}

#[cfg(test)]
#[path = "rich_tests.rs"]
mod tests;
