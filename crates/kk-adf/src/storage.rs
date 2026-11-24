//! Storage format (XHTML with `ac:`/`ri:` namespaces) <-> ADF.
//!
//! Storage elements we do not convert become `extension`/`inlineExtension` nodes holding
//! the verbatim XML; ADF nodes with no storage form are emitted as `<kk:adf-node>` holding
//! their JSON. Both directions are therefore lossless.

use serde_json::{json, Value};

use crate::{
    rich,
    xml::{esc, parse, raw_el, El, X},
    Doc, Mark, Node,
};

const RAW_TYPE: &str = "kk.storage";
const ADF_NODE: &str = "kk:adf-node";
const INLINE_NAMES: &[&str] = &[
    "a",
    "strong",
    "b",
    "em",
    "i",
    "code",
    "br",
    "span",
    "u",
    "s",
    "sub",
    "sup",
    "time",
    "ac:link",
    "ac:emoticon",
    "ac:image",
    "ac:inline-comment-marker",
    "ac:placeholder",
];
const INLINE_NODES: &[&str] = &[
    "text",
    "hardBreak",
    "inlineExtension",
    "mention",
    "emoji",
    "date",
    "status",
    "inlineCard",
    "mediaInline",
];

pub(crate) fn raw_ext(e: &El, kind: &str) -> Node {
    let mut xml = String::new();
    raw_el(e, &mut xml);
    Node::new(kind).attrs(json!({
        "extensionType": RAW_TYPE,
        "extensionKey": e.name,
        "parameters": {"xml": xml},
    }))
}

fn adf_node(e: &El) -> Result<Node, String> {
    serde_json::from_str(&e.text()).map_err(|err| format!("bad {ADF_NODE}: {err}"))
}

fn is_block(e: &El) -> bool {
    if e.name == ADF_NODE {
        return adf_node(e).map_or(true, |n| !INLINE_NODES.contains(&n.kind.as_str()));
    }
    !INLINE_NAMES.contains(&e.name.as_str())
}

pub(crate) fn blocks(kids: &[X]) -> Result<Vec<Node>, String> {
    let mut out = vec![];
    let mut run: Vec<&X> = vec![];
    for k in kids {
        match k {
            X::El(e) if is_block(e) => {
                flush(&mut run, &mut out)?;
                out.extend(block(e)?);
            }
            _ => run.push(k),
        }
    }
    flush(&mut run, &mut out)?;
    Ok(out)
}

fn flush(run: &mut Vec<&X>, out: &mut Vec<Node>) -> Result<(), String> {
    let blank = run
        .iter()
        .all(|x| matches!(x, X::Text(t) if t.trim().is_empty()));
    if !blank {
        out.push(Node::new("paragraph").content(inlines(run.iter().copied(), &[])?));
    }
    run.clear();
    Ok(())
}

fn block(e: &El) -> Result<Vec<Node>, String> {
    match rich::block(e)? {
        Some(nodes) => Ok(nodes),
        None => block_one(e).map(|n| vec![n]),
    }
}

fn block_one(e: &El) -> Result<Node, String> {
    let name = e.name.as_str();
    Ok(match name {
        "p" => match rich::sole(e) {
            Some(n) => n,
            None => styled(Node::new("paragraph"), e, inlines(&e.kids, &[])?),
        },
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => styled(
            Node::new("heading").attrs(json!({"level": name[1..].parse::<u8>().unwrap_or(1)})),
            e,
            inlines(&e.kids, &[])?,
        ),
        "ul" | "ol" => {
            let items = e
                .kids
                .iter()
                .filter_map(|k| match k {
                    X::El(li) if li.name == "li" => Some(li),
                    _ => None,
                })
                .map(|li| Ok(Node::new("listItem").content(blocks(&li.kids)?)))
                .collect::<Result<Vec<_>, String>>()?;
            Node::new(if name == "ul" {
                "bulletList"
            } else {
                "orderedList"
            })
            .content(items)
        }
        "blockquote" => Node::new("blockquote").content(blocks(&e.kids)?),
        "hr" => Node::new("rule"),
        "table" => {
            let mut rows = vec![];
            table_rows(&e.kids, &mut rows)?;
            Node::new("table").content(rows)
        }
        ADF_NODE => adf_node(e)?,
        _ => raw_ext(e, "extension"),
    })
}

fn styled(mut n: Node, e: &El, content: Vec<Node>) -> Node {
    n.marks = rich::block_marks(e);
    n.content(content)
}

fn table_rows(kids: &[X], out: &mut Vec<Node>) -> Result<(), String> {
    for k in kids {
        let X::El(e) = k else { continue };
        match e.name.as_str() {
            "tbody" | "thead" | "tfoot" => table_rows(&e.kids, out)?,
            "tr" => {
                let cells = e
                    .kids
                    .iter()
                    .filter_map(|c| match c {
                        X::El(c) if c.name == "th" || c.name == "td" => Some(c),
                        _ => None,
                    })
                    .map(|c| {
                        let mut attrs = serde_json::Map::new();
                        for key in ["colspan", "rowspan"] {
                            if let Some(n) = c.attr(key).and_then(|v| v.parse::<u32>().ok()) {
                                attrs.insert(key.into(), n.into());
                            }
                        }
                        let kind = if c.name == "th" {
                            "tableHeader"
                        } else {
                            "tableCell"
                        };
                        let mut content = blocks(&c.kids)?;
                        for b in content.iter_mut().filter(|b| b.kind == "expand") {
                            b.kind = "nestedExpand".into();
                        }
                        let mut n = Node::new(kind).content(content);
                        if !attrs.is_empty() {
                            n = n.attrs(Value::Object(attrs));
                        }
                        Ok(n)
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                out.push(Node::new("tableRow").content(cells));
            }
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn inlines<'a>(
    kids: impl IntoIterator<Item = &'a X>,
    marks: &[Mark],
) -> Result<Vec<Node>, String> {
    let with = |m: Mark| [marks, &[m]].concat();
    let mut out = vec![];
    for k in kids {
        match k {
            X::Text(t) | X::Cdata(t) => {
                if !t.is_empty() {
                    out.push(Node::text(t, marks));
                }
            }
            X::El(e) => match (rich::inline(e, marks)?, e.name.as_str()) {
                (Some(nodes), _) => out.extend(nodes),
                (None, "strong" | "b") => {
                    out.extend(inlines(&e.kids, &with(Mark::new("strong", None)))?)
                }
                (None, "em" | "i") => out.extend(inlines(&e.kids, &with(Mark::new("em", None)))?),
                (None, "code") => out.extend(inlines(&e.kids, &with(Mark::new("code", None)))?),
                (None, "a") => {
                    let href = e.attr("href").unwrap_or("");
                    out.extend(inlines(
                        &e.kids,
                        &with(Mark::new("link", Some(json!({"href": href})))),
                    )?);
                }
                (None, "br") => out.push(Node::new("hardBreak")),
                (None, ADF_NODE) => out.push(adf_node(e)?),
                (None, _) => out.push(raw_ext(e, "inlineExtension")),
            },
        }
    }
    Ok(out)
}

pub fn from_storage(xml: &str) -> Result<Doc, String> {
    let tree = parse(xml)?;
    Ok(Doc::new(blocks(&tree)?))
}

pub(crate) fn fallback(n: &Node) -> String {
    let json = serde_json::to_string(n).unwrap_or_default();
    format!(
        "<{ADF_NODE}><![CDATA[{}]]></{ADF_NODE}>",
        json.replace("]]>", "]]]]><![CDATA[>")
    )
}

pub(crate) fn children(n: &Node) -> &[Node] {
    n.content.as_deref().unwrap_or(&[])
}

pub(crate) fn block_list(ns: &[Node]) -> String {
    ns.iter().map(block_out).collect()
}

pub(crate) fn inline_list(ns: &[Node]) -> String {
    ns.iter().map(inline_out).collect()
}

fn block_out(n: &Node) -> String {
    let c = children(n);
    let heading = n.kind == "heading";
    if n.marks.is_some() && !heading && n.kind != "paragraph" {
        return fallback(n);
    }
    if let Some(s) = rich::block_out(n) {
        return s;
    }
    match n.kind.as_str() {
        "paragraph" | "heading" => {
            let Some(style) = rich::block_style(n) else {
                return fallback(n);
            };
            let tag = if heading {
                let l = n
                    .attrs
                    .as_ref()
                    .and_then(|a| a["level"].as_u64())
                    .unwrap_or(1)
                    .clamp(1, 6);
                format!("h{l}")
            } else {
                "p".into()
            };
            format!("<{tag}{style}>{}</{tag}>", inline_list(c))
        }
        "bulletList" => format!("<ul>{}</ul>", block_list(c)),
        "orderedList" => format!("<ol>{}</ol>", block_list(c)),
        "listItem" => format!("<li>{}</li>", block_list(c)),
        "blockquote" => format!("<blockquote>{}</blockquote>", block_list(c)),
        "rule" => "<hr />".into(),
        "table" => format!("<table><tbody>{}</tbody></table>", block_list(c)),
        "tableRow" => format!("<tr>{}</tr>", block_list(c)),
        "tableHeader" | "tableCell" => {
            let tag = if n.kind == "tableHeader" { "th" } else { "td" };
            let span: String = ["colspan", "rowspan"]
                .iter()
                .filter_map(|k| {
                    let v = n.attrs.as_ref()?[*k].as_u64()?;
                    Some(format!(" {k}=\"{v}\""))
                })
                .collect();
            format!("<{tag}{span}>{}</{tag}>", block_list(c))
        }
        "extension" => raw_out(n),
        _ => fallback(n),
    }
}

fn raw_out(n: &Node) -> String {
    let xml = n.attrs.as_ref().and_then(|a| {
        (a["extensionType"] == RAW_TYPE)
            .then(|| a["parameters"]["xml"].as_str().map(String::from))
            .flatten()
    });
    xml.unwrap_or_else(|| fallback(n))
}

fn inline_out(n: &Node) -> String {
    if let Some(s) = rich::inline_out(n) {
        return s;
    }
    match n.kind.as_str() {
        "text" => {
            let tags: Option<Vec<_>> = n
                .marks
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(rich::mark_tags)
                .collect();
            let Some(tags) = tags else {
                return fallback(n);
            };
            let open: String = tags.iter().map(|(o, _)| o.as_str()).collect();
            let close: String = tags.iter().rev().map(|(_, c)| *c).collect();
            format!("{open}{}{close}", esc(n.text.as_deref().unwrap_or("")))
        }
        "hardBreak" => "<br />".into(),
        "inlineExtension" => raw_out(n),
        _ => fallback(n),
    }
}

pub fn to_storage(doc: &Doc) -> String {
    rich::reset_task_ids();
    block_list(&doc.content)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adf(xml: &str) -> Value {
        serde_json::to_value(from_storage(xml).unwrap()).unwrap()
    }

    fn round(xml: &str) {
        let doc = from_storage(xml).unwrap();
        assert_eq!(from_storage(&to_storage(&doc)).unwrap(), doc);
    }

    #[test]
    fn paragraph_with_marks() {
        let xml = "<p>Hi <strong>bold <em>both</em></strong> <code>x</code> <a href=\"https://e.io/?a=1&amp;b=2\">link</a></p>";
        assert_eq!(
            adf(xml),
            json!({"version": 1, "type": "doc", "content": [{"type": "paragraph", "content": [
                {"type": "text", "text": "Hi "},
                {"type": "text", "text": "bold ", "marks": [{"type": "strong"}]},
                {"type": "text", "text": "both", "marks": [{"type": "strong"}, {"type": "em"}]},
                {"type": "text", "text": " "},
                {"type": "text", "text": "x", "marks": [{"type": "code"}]},
                {"type": "text", "text": " "},
                {"type": "text", "text": "link", "marks": [{"type": "link", "attrs": {"href": "https://e.io/?a=1&b=2"}}]},
            ]}]})
        );
        round(xml);
    }

    #[test]
    fn heading_rule_quote_break() {
        let xml = "<h2>Title</h2><hr /><blockquote><p>q<br />r</p></blockquote>";
        assert_eq!(
            adf(xml)["content"],
            json!([
                {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Title"}]},
                {"type": "rule"},
                {"type": "blockquote", "content": [{"type": "paragraph", "content": [
                    {"type": "text", "text": "q"}, {"type": "hardBreak"}, {"type": "text", "text": "r"}]}]},
            ])
        );
        round(xml);
    }

    #[test]
    fn lists_wrap_bare_text_in_paragraphs() {
        assert_eq!(
            adf("<ul><li>a</li><li><p>b</p></li></ul><ol><li>c</li></ol>")["content"],
            json!([
                {"type": "bulletList", "content": [
                    {"type": "listItem", "content": [{"type": "paragraph", "content": [{"type": "text", "text": "a"}]}]},
                    {"type": "listItem", "content": [{"type": "paragraph", "content": [{"type": "text", "text": "b"}]}]},
                ]},
                {"type": "orderedList", "content": [
                    {"type": "listItem", "content": [{"type": "paragraph", "content": [{"type": "text", "text": "c"}]}]},
                ]},
            ])
        );
        round("<ul><li><p>a</p></li></ul><ol><li><p>c</p></li></ol>");
    }

    #[test]
    fn code_block_keeps_cdata_terminator() {
        let doc = from_storage(
            "<ac:structured-macro ac:name=\"code\"><ac:parameter ac:name=\"language\">rust</ac:parameter>\
             <ac:plain-text-body><![CDATA[a < b]]></ac:plain-text-body></ac:structured-macro>",
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(&doc.content[0]).unwrap(),
            json!({"type": "codeBlock", "attrs": {"language": "rust"}, "content": [{"type": "text", "text": "a < b"}]})
        );
        let tricky = Node::new("codeBlock").content(vec![Node::text("x]]>y", &[])]);
        let again = from_storage(&to_storage(&Doc::new(vec![tricky.clone()]))).unwrap();
        assert_eq!(again.content, vec![tricky]);
    }

    #[test]
    fn simple_table() {
        let xml = "<table><tbody><tr><th><p>h</p></th></tr><tr><td colspan=\"2\"><p>c</p></td></tr></tbody></table>";
        assert_eq!(
            adf(xml)["content"][0],
            json!({"type": "table", "content": [
                {"type": "tableRow", "content": [{"type": "tableHeader", "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "h"}]}]}]},
                {"type": "tableRow", "content": [{"type": "tableCell", "attrs": {"colspan": 2}, "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "c"}]}]}]},
            ]})
        );
        round(xml);
    }

    #[test]
    fn unknown_elements_are_preserved() {
        let xml = "<p>a <ac:placeholder>x</ac:placeholder> b</p><ac:structured-macro ac:name=\"toc\"><ac:parameter ac:name=\"x\"><ri:page/></ac:parameter></ac:structured-macro>";
        let v = adf(xml);
        assert_eq!(
            v["content"][0]["content"][1],
            json!({"type": "inlineExtension", "attrs": {
                "extensionType": "kk.storage", "extensionKey": "ac:placeholder",
                "parameters": {"xml": "<ac:placeholder>x</ac:placeholder>"}}})
        );
        assert_eq!(v["content"][1]["attrs"]["extensionType"], "kk.storage");
        round(xml);
    }

    #[test]
    fn unsupported_adf_survives_storage() {
        let panel = Node::new("panel")
            .attrs(json!({"panelType": "info"}))
            .content(vec![Node::new("paragraph")]);
        let marked = Node::text("u", &[Mark::new("underline", None)]);
        let doc = Doc::new(vec![panel, Node::new("paragraph").content(vec![marked])]);
        assert_eq!(from_storage(&to_storage(&doc)).unwrap(), doc);
    }

    #[test]
    fn malformed_input_is_an_error() {
        assert_eq!(from_storage("<p>x").unwrap_err(), "unclosed <p>");
        assert_eq!(from_storage("</p>").unwrap_err(), "unexpected </p>");
        assert!(from_storage("<p>&bogus;</p>").is_err());
    }

    #[test]
    fn entities_and_whitespace() {
        assert_eq!(
            adf("\n<p>a&nbsp;&#65;&lt;</p>\n")["content"],
            json!([{"type": "paragraph", "content": [{"type": "text", "text": "a\u{a0}A<"}]}])
        );
    }
}
