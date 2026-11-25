//! Task items inside a document, for the task API.

use serde_json::Value;

use crate::{Doc, Node};

pub struct Task {
    pub local_id: String,
    pub done: bool,
    pub content: Vec<Node>,
    /// Epoch milliseconds of the first `date` node.
    pub due: Option<i64>,
    /// Id of the first mentioned user.
    pub assignee: Option<String>,
}

fn walk(nodes: &[Node], f: &mut impl FnMut(&Node)) {
    for n in nodes {
        f(n);
        walk(n.content.as_deref().unwrap_or(&[]), f);
    }
}

fn attr<'a>(n: &'a Node, key: &str) -> Option<&'a str> {
    n.attrs.as_ref()?.get(key)?.as_str()
}

pub fn tasks(doc: &Doc) -> Vec<Task> {
    let mut out = Vec::new();
    walk(&doc.content, &mut |n| {
        let Some(local_id) = (n.kind == "taskItem").then(|| attr(n, "localId")).flatten() else {
            return;
        };
        let content = n.content.clone().unwrap_or_default();
        let (mut due, mut assignee) = (None, None);
        walk(&content, &mut |c| match c.kind.as_str() {
            "date" if due.is_none() => due = attr(c, "timestamp").and_then(|t| t.parse().ok()),
            "mention" if assignee.is_none() => assignee = attr(c, "id").map(String::from),
            _ => {}
        });
        out.push(Task {
            local_id: local_id.into(),
            done: attr(n, "state") == Some("DONE"),
            content,
            due,
            assignee,
        });
    });
    out
}

/// Sets a task item's state; false when no such item exists.
pub fn set_task_done(doc: &mut Doc, local_id: &str, done: bool) -> bool {
    fn go(nodes: &mut [Node], id: &str, done: bool) -> bool {
        for n in nodes {
            if n.kind == "taskItem" && attr(n, "localId") == Some(id) {
                let attrs = n
                    .attrs
                    .get_or_insert_with(|| Value::Object(Default::default()));
                attrs["state"] = Value::String(if done { "DONE" } else { "TODO" }.into());
                return true;
            }
            if n.content.as_mut().is_some_and(|c| go(c, id, done)) {
                return true;
            }
        }
        false
    }
    go(&mut doc.content, local_id, done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc() -> Doc {
        serde_json::from_value(json!({"version": 1, "type": "doc", "content": [
            {"type": "taskList", "attrs": {"localId": "l"}, "content": [
                {"type": "taskItem", "attrs": {"localId": "a", "state": "TODO"}, "content": [
                    {"type": "text", "text": "ship "},
                    {"type": "mention", "attrs": {"id": "42", "text": "@x"}},
                    {"type": "date", "attrs": {"timestamp": "1700000000000"}}]},
                {"type": "taskItem", "attrs": {"localId": "b", "state": "DONE"}}]}]}))
        .unwrap()
    }

    #[test]
    fn extracts_state_assignee_and_due() {
        let t = tasks(&doc());
        assert_eq!(t.len(), 2);
        assert_eq!(
            (t[0].local_id.as_str(), t[0].done, t[0].due),
            ("a", false, Some(1700000000000))
        );
        assert_eq!(t[0].assignee.as_deref(), Some("42"));
        assert!(t[1].done && t[1].content.is_empty());
    }

    #[test]
    fn sets_state_by_local_id() {
        let mut d = doc();
        assert!(set_task_done(&mut d, "a", true));
        assert!(tasks(&d)[0].done);
        assert!(!set_task_done(&mut d, "zzz", true));
    }
}
