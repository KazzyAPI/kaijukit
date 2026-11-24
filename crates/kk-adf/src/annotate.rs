//! Inline-comment anchors: `annotation` marks on text nodes.

use serde_json::json;

use crate::{Doc, Mark, Node};

fn is_anchor(m: &Mark, id: Option<&str>) -> bool {
    m.kind == "annotation"
        && m.attrs.as_ref().is_some_and(|a| {
            a["annotationType"] == "inlineComment" && id.is_none_or(|i| a["id"] == i)
        })
}

fn count(nodes: &[Node], selection: &str) -> usize {
    nodes
        .iter()
        .map(|n| match (&n.text, &n.content) {
            (Some(t), _) => t.matches(selection).count(),
            (_, Some(c)) => count(c, selection),
            _ => 0,
        })
        .sum()
}

fn mark(nodes: &mut Vec<Node>, selection: &str, target: usize, seen: &mut usize, id: &str) -> bool {
    for i in 0..nodes.len() {
        if let Some(text) = nodes[i].text.clone() {
            for (at, _) in text.match_indices(selection) {
                if *seen != target {
                    *seen += 1;
                    continue;
                }
                let n = &nodes[i];
                let marks = n.marks.clone().unwrap_or_default();
                let anchor = Mark::new(
                    "annotation",
                    Some(json!({"id": id, "annotationType": "inlineComment"})),
                );
                let end = at + selection.len();
                let pieces = [
                    (&text[..at], marks.clone()),
                    (selection, [marks.clone(), vec![anchor]].concat()),
                    (&text[end..], marks),
                ];
                let split: Vec<Node> = pieces
                    .into_iter()
                    .filter(|(t, _)| !t.is_empty())
                    .map(|(t, m)| Node::text(t, &m))
                    .collect();
                nodes.splice(i..=i, split);
                return true;
            }
        } else if let Some(c) = nodes[i].content.as_mut() {
            if mark(c, selection, target, seen, id) {
                return true;
            }
        }
    }
    false
}

/// Anchors comment `id` on the `index`th (0-based) occurrence of `selection` inside a single
/// text node. `expected` is the occurrence count the client saw; a mismatch is an error.
pub fn annotate(
    doc: &mut Doc,
    selection: &str,
    expected: usize,
    index: usize,
    id: &str,
) -> Result<(), String> {
    if selection.is_empty() {
        return Err("textSelection must not be empty".into());
    }
    let found = count(&doc.content, selection);
    if found == 0 {
        return Err("textSelection was not found in the page body".into());
    }
    if found != expected {
        return Err(format!(
            "textSelectionMatchCount is {expected} but the selection occurs {found} times"
        ));
    }
    if index >= found {
        return Err("textSelectionMatchIndex is out of range".into());
    }
    mark(&mut doc.content, selection, index, &mut 0, id);
    Ok(())
}

fn strip(nodes: &mut [Node], id: &str) {
    for n in nodes {
        if let Some(m) = n.marks.as_mut() {
            m.retain(|m| !is_anchor(m, Some(id)));
            if m.is_empty() {
                n.marks = None;
            }
        }
        if let Some(c) = n.content.as_mut() {
            strip(c, id);
        }
    }
}

pub fn unannotate(doc: &mut Doc, id: &str) {
    strip(&mut doc.content, id);
}

fn collect(nodes: &[Node], out: &mut Vec<String>) {
    for n in nodes {
        for m in n.marks.iter().flatten().filter(|m| is_anchor(m, None)) {
            if let Some(id) = m.attrs.as_ref().and_then(|a| a["id"].as_str()) {
                out.push(id.to_string());
            }
        }
        collect(n.content.as_deref().unwrap_or(&[]), out);
    }
}

/// Ids of every inline-comment anchor present in the document.
pub fn annotation_ids(doc: &Doc) -> Vec<String> {
    let mut out = vec![];
    collect(&doc.content, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn doc(v: Value) -> Doc {
        serde_json::from_value(json!({"version": 1, "type": "doc", "content": v})).unwrap()
    }

    fn para(text: &str) -> Value {
        json!({"type": "paragraph", "content": [{"type": "text", "text": text}]})
    }

    #[test]
    fn splits_the_chosen_occurrence() {
        let mut d = doc(json!([para("ab ab"), para("xab")]));
        annotate(&mut d, "ab", 3, 1, "c1").unwrap();
        let anchor =
            json!({"type": "annotation", "attrs": {"id": "c1", "annotationType": "inlineComment"}});
        assert_eq!(
            serde_json::to_value(&d).unwrap()["content"],
            json!([
                {"type": "paragraph", "content": [
                    {"type": "text", "text": "ab "},
                    {"type": "text", "text": "ab", "marks": [anchor]},
                ]},
                para("xab"),
            ])
        );
        assert_eq!(annotation_ids(&d), vec!["c1"]);
        unannotate(&mut d, "c1");
        assert!(annotation_ids(&d).is_empty());
    }

    #[test]
    fn rejects_bad_selections() {
        let mut d = doc(json!([para("ab ab")]));
        assert_eq!(
            annotate(&mut d, "zz", 1, 0, "c").unwrap_err(),
            "textSelection was not found in the page body"
        );
        assert_eq!(
            annotate(&mut d, "ab", 1, 0, "c").unwrap_err(),
            "textSelectionMatchCount is 1 but the selection occurs 2 times"
        );
        assert_eq!(
            annotate(&mut d, "ab", 2, 2, "c").unwrap_err(),
            "textSelectionMatchIndex is out of range"
        );
    }
}
