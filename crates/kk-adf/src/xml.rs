//! Minimal XML tree for storage-format input: entities, CDATA, attributes, verbatim re-serialization.

const MAX_DEPTH: usize = 200;

pub(crate) enum X {
    Text(String),
    Cdata(String),
    El(El),
}

pub(crate) struct El {
    pub(crate) name: String,
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) kids: Vec<X>,
}

impl El {
    pub(crate) fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub(crate) fn text(&self) -> String {
        self.kids
            .iter()
            .filter_map(|k| match k {
                X::Text(t) | X::Cdata(t) => Some(t.as_str()),
                X::El(_) => None,
            })
            .collect()
    }
}

pub(crate) fn decode(s: &str) -> Result<String, String> {
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

pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
    depth: usize,
}

pub(crate) fn parse(xml: &str) -> Result<Vec<X>, String> {
    Parser {
        s: xml,
        i: 0,
        depth: 0,
    }
    .nodes(None)
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

pub(crate) fn raw(x: &X, out: &mut String) {
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

pub(crate) fn raw_el(e: &El, out: &mut String) {
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
