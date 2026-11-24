//! Storage format (XHTML with `ac:`/`ri:` namespaces) <-> ADF.
//!
//! Storage elements we do not convert become `extension`/`inlineExtension` nodes holding
//! the verbatim XML; ADF nodes with no storage form are emitted as `<kk:adf-node>` holding
//! their JSON. Both directions are therefore lossless.

use serde_json::{json, Value};

use crate::{Doc, Mark, Node};

const RAW_TYPE: &str = "kk.storage";
const ADF_NODE: &str = "kk:adf-node";
const MAX_DEPTH: usize = 200;
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
const KNOWN_MARKS: &[&str] = &["strong", "em", "code", "link"];

enum X {
    Text(String),
    Cdata(String),
    El(El),
}

struct El {
    name: String,
    attrs: Vec<(String, String)>,
    kids: Vec<X>,
}

impl El {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn text(&self) -> String {
        self.kids
            .iter()
            .filter_map(|k| match k {
                X::Text(t) | X::Cdata(t) => Some(t.as_str()),
                X::El(_) => None,
            })
            .collect()
    }
}

fn decode(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i + 1..];
        let j = tail.find(';').ok_or("unterminated entity")?;
        let ent = &tail[..j];
        match ent {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" => out.push('\''),
            "nbsp" => out.push('\u{a0}'),
            _ => {
                let code = if let Some(h) = ent.strip_prefix("#x") {
                    u32::from_str_radix(h, 16).ok()
                } else if let Some(d) = ent.strip_prefix('#') {
                    d.parse().ok()
                } else {
                    None
                };
                out.push(
                    code.and_then(char::from_u32)
                        .ok_or_else(|| format!("unknown entity &{ent};"))?,
                );
            }
        }
        rest = &tail[j + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
    depth: usize,
}

impl Parser<'_> {
    fn rest(&self) -> &str {
        &self.s[self.i..]
    }

    fn take_while(&mut self, f: impl Fn(char) -> bool) -> String {
        let r = self.rest();
        let n = r.find(|c| !f(c)).unwrap_or(r.len());
        let out = r[..n].to_string();
        self.i += n;
        out
    }

    fn nodes(&mut self, close: Option<&str>) -> Result<Vec<X>, String> {
        let mut out = vec![];
        loop {
            let r = self.rest();
            if r.is_empty() {
                return match close {
                    Some(c) => Err(format!("unclosed <{c}>")),
                    None => Ok(out),
                };
            }
            if let Some(t) = r.strip_prefix("</") {
                let end = t.find('>').ok_or("bad closing tag")?;
                let name = t[..end].trim();
                if Some(name) != close {
                    return Err(format!("unexpected </{name}>"));
                }
                self.i += 2 + end + 1;
                return Ok(out);
            }
            if let Some(t) = r.strip_prefix("<![CDATA[") {
                let end = t.find("]]>").ok_or("unterminated CDATA")?;
                out.push(X::Cdata(t[..end].into()));
                self.i += 9 + end + 3;
            } else if let Some(t) = r.strip_prefix("<!--") {
                let end = t.find("-->").ok_or("unterminated comment")?;
                self.i += 4 + end + 3;
            } else if r.starts_with('<') {
                out.push(X::El(self.element()?));
            } else {
                let end = r.find('<').unwrap_or(r.len());
                out.push(X::Text(decode(&r[..end])?));
                self.i += end;
            }
        }
    }

    fn element(&mut self) -> Result<El, String> {
        self.i += 1;
        let name = self.take_while(|c| !c.is_whitespace() && c != '/' && c != '>');
        if name.is_empty() {
            return Err("bad tag".into());
        }
        let mut attrs = vec![];
        loop {
            self.take_while(char::is_whitespace);
            let r = self.rest();
            if r.starts_with("/>") {
                self.i += 2;
                return Ok(El {
                    name,
                    attrs,
                    kids: vec![],
                });
            }
            if r.starts_with('>') {
                self.i += 1;
                self.depth += 1;
                if self.depth > MAX_DEPTH {
                    return Err("document nested too deeply".into());
                }
                let kids = self.nodes(Some(&name))?;
                self.depth -= 1;
                return Ok(El { name, attrs, kids });
            }
            let key = self.take_while(|c| c != '=' && !c.is_whitespace() && c != '/' && c != '>');
            if key.is_empty() || !self.rest().starts_with('=') {
                return Err(format!("bad attribute in <{name}>"));
            }
            self.i += 1;
            let q = self
                .rest()
                .chars()
                .next()
                .filter(|c| *c == '"' || *c == '\'')
                .ok_or("unquoted attribute")?;
            self.i += 1;
            let val = self.take_while(|c| c != q);
            if !self.rest().starts_with(q) {
                return Err("unterminated attribute".into());
            }
            self.i += 1;
            attrs.push((key, decode(&val)?));
        }
    }
}

fn raw(x: &X, out: &mut String) {
    match x {
        X::Text(t) => out.push_str(&esc(t)),
        X::Cdata(c) => {
            out.push_str("<![CDATA[");
            out.push_str(c);
            out.push_str("]]>");
        }
        X::El(e) => raw_el(e, out),
    }
}

fn raw_el(e: &El, out: &mut String) {
    out.push('<');
    out.push_str(&e.name);
    for (k, v) in &e.attrs {
        out.push_str(&format!(" {k}=\"{}\"", esc_attr(v)));
    }
    if e.kids.is_empty() {
        out.push_str("/>");
    } else {
        out.push('>');
        e.kids.iter().for_each(|k| raw(k, out));
        out.push_str(&format!("</{}>", e.name));
    }
}

fn raw_ext(e: &El, kind: &str) -> Node {
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

fn blocks(kids: &[X]) -> Result<Vec<Node>, String> {
    let mut out = vec![];
    let mut run: Vec<&X> = vec![];
    for k in kids {
        match k {
            X::El(e) if is_block(e) => {
                flush(&mut run, &mut out)?;
                out.push(block(e)?);
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

fn block(e: &El) -> Result<Node, String> {
    let name = e.name.as_str();
    Ok(match name {
        "p" => Node::new("paragraph").content(inlines(&e.kids, &[])?),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Node::new("heading")
            .attrs(json!({"level": name[1..].parse::<u8>().unwrap_or(1)}))
            .content(inlines(&e.kids, &[])?),
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
        "ac:structured-macro" => code_block(e)?.unwrap_or_else(|| raw_ext(e, "extension")),
        ADF_NODE => adf_node(e)?,
        _ => raw_ext(e, "extension"),
    })
}

fn code_block(e: &El) -> Result<Option<Node>, String> {
    if e.attr("ac:name") != Some("code") {
        return Ok(None);
    }
    let mut lang = None;
    let mut body = String::new();
    for k in &e.kids {
        match k {
            X::El(p) if p.name == "ac:parameter" && p.attr("ac:name") == Some("language") => {
                lang = Some(p.text());
            }
            X::El(b) if b.name == "ac:plain-text-body" => body = b.text(),
            X::Text(t) if t.trim().is_empty() => {}
            _ => return Ok(None),
        }
    }
    let mut node = Node::new("codeBlock");
    if let Some(l) = lang {
        node = node.attrs(json!({"language": l}));
    }
    if !body.is_empty() {
        node = node.content(vec![Node::text(&body, &[])]);
    }
    Ok(Some(node))
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
                        let mut n = Node::new(kind).content(blocks(&c.kids)?);
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

fn inlines<'a>(kids: impl IntoIterator<Item = &'a X>, marks: &[Mark]) -> Result<Vec<Node>, String> {
    let with = |m: Mark| [marks, &[m]].concat();
    let mut out = vec![];
    for k in kids {
        match k {
            X::Text(t) | X::Cdata(t) => {
                if !t.is_empty() {
                    out.push(Node::text(t, marks));
                }
            }
            X::El(e) => match e.name.as_str() {
                "strong" | "b" => out.extend(inlines(&e.kids, &with(Mark::new("strong", None)))?),
                "em" | "i" => out.extend(inlines(&e.kids, &with(Mark::new("em", None)))?),
                "code" => out.extend(inlines(&e.kids, &with(Mark::new("code", None)))?),
                "a" => {
                    let href = e.attr("href").unwrap_or("");
                    out.extend(inlines(
                        &e.kids,
                        &with(Mark::new("link", Some(json!({"href": href})))),
                    )?);
                }
                "br" => out.push(Node::new("hardBreak")),
                ADF_NODE => out.push(adf_node(e)?),
                _ => out.push(raw_ext(e, "inlineExtension")),
            },
        }
    }
    Ok(out)
}

pub fn from_storage(xml: &str) -> Result<Doc, String> {
    let mut p = Parser {
        s: xml,
        i: 0,
        depth: 0,
    };
    let tree = p.nodes(None)?;
    Ok(Doc::new(blocks(&tree)?))
}

fn fallback(n: &Node) -> String {
    let json = serde_json::to_string(n).unwrap_or_default();
    format!(
        "<{ADF_NODE}><![CDATA[{}]]></{ADF_NODE}>",
        json.replace("]]>", "]]]]><![CDATA[>")
    )
}

fn children(n: &Node) -> &[Node] {
    n.content.as_deref().unwrap_or(&[])
}

fn block_list(ns: &[Node]) -> String {
    ns.iter().map(block_out).collect()
}

fn inline_list(ns: &[Node]) -> String {
    ns.iter().map(inline_out).collect()
}

fn block_out(n: &Node) -> String {
    let c = children(n);
    match n.kind.as_str() {
        "paragraph" => format!("<p>{}</p>", inline_list(c)),
        "heading" => {
            let l = n
                .attrs
                .as_ref()
                .and_then(|a| a["level"].as_u64())
                .unwrap_or(1)
                .clamp(1, 6);
            format!("<h{l}>{}</h{l}>", inline_list(c))
        }
        "bulletList" => format!("<ul>{}</ul>", block_list(c)),
        "orderedList" => format!("<ol>{}</ol>", block_list(c)),
        "listItem" => format!("<li>{}</li>", block_list(c)),
        "blockquote" => format!("<blockquote>{}</blockquote>", block_list(c)),
        "rule" => "<hr />".into(),
        "codeBlock" => {
            let lang = n
                .attrs
                .as_ref()
                .and_then(|a| a["language"].as_str())
                .map(|l| {
                    format!(
                        "<ac:parameter ac:name=\"language\">{}</ac:parameter>",
                        esc(l)
                    )
                })
                .unwrap_or_default();
            let body: String = c.iter().filter_map(|t| t.text.as_deref()).collect();
            format!(
                "<ac:structured-macro ac:name=\"code\" ac:schema-version=\"1\">{lang}\
                 <ac:plain-text-body><![CDATA[{}]]></ac:plain-text-body></ac:structured-macro>",
                body.replace("]]>", "]]]]><![CDATA[>")
            )
        }
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
    match n.kind.as_str() {
        "text" => {
            let marks = n.marks.as_deref().unwrap_or(&[]);
            if marks
                .iter()
                .any(|m| !KNOWN_MARKS.contains(&m.kind.as_str()))
            {
                return fallback(n);
            }
            let mut open = String::new();
            let mut close = String::new();
            for m in marks {
                let (o, c) = match m.kind.as_str() {
                    "strong" => ("<strong>".to_string(), "</strong>"),
                    "em" => ("<em>".to_string(), "</em>"),
                    "code" => ("<code>".to_string(), "</code>"),
                    _ => {
                        let href = m
                            .attrs
                            .as_ref()
                            .and_then(|a| a["href"].as_str())
                            .unwrap_or("");
                        (format!("<a href=\"{}\">", esc_attr(href)), "</a>")
                    }
                };
                open.push_str(&o);
                close.insert_str(0, c);
            }
            format!("{open}{}{close}", esc(n.text.as_deref().unwrap_or("")))
        }
        "hardBreak" => "<br />".into(),
        "inlineExtension" => raw_out(n),
        _ => fallback(n),
    }
}

pub fn to_storage(doc: &Doc) -> String {
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
        let xml = "<p>a <ac:emoticon ac:name=\"smile\"/> b</p><ac:structured-macro ac:name=\"toc\"><ac:parameter ac:name=\"x\">1</ac:parameter></ac:structured-macro>";
        let v = adf(xml);
        assert_eq!(
            v["content"][0]["content"][1],
            json!({"type": "inlineExtension", "attrs": {
                "extensionType": "kk.storage", "extensionKey": "ac:emoticon",
                "parameters": {"xml": "<ac:emoticon ac:name=\"smile\"/>"}}})
        );
        assert_eq!(v["content"][1]["type"], "extension");
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
