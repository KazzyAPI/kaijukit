use kk_adf::{Doc, Node};
use slint::{ComponentHandle, ModelRc, SharedString, StyledText, VecModel};
use std::rc::Rc;

slint::include_modules!();

const SAMPLE: &str = include_str!("../../sample.json");
const TREE: &str = include_str!("../../tree.txt");

fn kids(n: &Node) -> &[Node] {
    n.content.as_deref().unwrap_or(&[])
}

fn esc(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        if "\\`*_{}[]<>()#+-.!|~&".contains(c) {
            o.push('\\');
        }
        o.push(c);
    }
    o
}

fn markdown(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|n| {
            let marks = n.marks.as_deref().unwrap_or(&[]);
            let has = |k: &str| marks.iter().any(|m| m.kind == k);
            let t = n.text.as_deref().unwrap_or("");
            let mut s = if has("code") { format!("`{t}`") } else { esc(t) };
            if has("strong") { s = format!("**{s}**") }
            if has("em") { s = format!("*{s}*") }
            if let Some(m) = marks.iter().find(|m| m.kind == "link") {
                let href = m.attrs.as_ref().and_then(|a| a["href"].as_str()).unwrap_or("");
                s = format!("[{s}]({href})");
            }
            s
        })
        .collect()
}

fn styled(nodes: &[Node]) -> StyledText {
    StyledText::from_markdown(&markdown(nodes)).unwrap_or_default()
}

fn block(kind: &str) -> Block {
    Block {
        kind: kind.into(),
        level: 0,
        indent: 0,
        marker: "".into(),
        text: StyledText::default(),
        plain: "".into(),
        cells: ModelRc::default(),
        header: false,
    }
}

fn flatten(n: &Node, indent: i32, marker: &str, out: &mut Vec<Block>) {
    match n.kind.as_str() {
        "heading" => out.push(Block {
            level: n.attrs.as_ref().and_then(|a| a["level"].as_i64()).unwrap_or(1) as i32,
            text: styled(kids(n)),
            ..block("heading")
        }),
        "paragraph" => out.push(Block {
            indent,
            marker: marker.into(),
            text: styled(kids(n)),
            ..block("para")
        }),
        "bulletList" | "orderedList" => {
            for (i, item) in kids(n).iter().enumerate() {
                let m = if n.kind == "bulletList" { "•".to_string() } else { format!("{}.", i + 1) };
                for (j, c) in kids(item).iter().enumerate() {
                    flatten(c, indent + if j == 0 && c.kind == "paragraph" { 0 } else { 1 }, if j == 0 { &m } else { "" }, out);
                }
            }
        }
        "codeBlock" => out.push(Block {
            plain: kids(n).iter().filter_map(|t| t.text.clone()).collect::<String>().into(),
            ..block("code")
        }),
        "blockquote" => {
            let start = out.len();
            for c in kids(n) {
                flatten(c, 0, "", out);
            }
            for b in &mut out[start..] {
                if b.kind == "para" {
                    b.kind = "quote".into();
                }
            }
        }
        "rule" => out.push(block("rule")),
        "table" => {
            for r in kids(n) {
                let header = kids(r).first().is_some_and(|c| c.kind == "tableHeader");
                let cells: Vec<StyledText> = kids(r)
                    .iter()
                    .map(|c| styled(kids(c).first().map(kids).unwrap_or(&[])))
                    .collect();
                out.push(Block { cells: ModelRc::new(VecModel::from(cells)), header, ..block("row") });
            }
        }
        _ => {}
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let width: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000.0);
    let doc = Doc::parse(SAMPLE).unwrap();
    let mut blocks = vec![];
    for n in &doc.content {
        flatten(n, 0, "", &mut blocks);
    }
    let tree: Vec<TreeRow> = TREE
        .lines()
        .map(|l| {
            let (d, t) = l.split_once(' ').unwrap();
            TreeRow { depth: d.parse().unwrap(), title: SharedString::from(t) }
        })
        .collect();
    let app = App::new().unwrap();
    app.set_blocks(Rc::new(VecModel::from(blocks)).into());
    app.set_tree(Rc::new(VecModel::from(tree)).into());
    app.window().set_size(slint::LogicalSize::new(width, 700.0));
    let _timer = args.get(2).map(|path| {
        let (path, weak) = (path.clone(), app.as_weak());
        let t = slint::Timer::default();
        t.start(slint::TimerMode::SingleShot, std::time::Duration::from_millis(1500), move || {
            let snap = weak.unwrap().window().take_snapshot().unwrap();
            image::save_buffer(&path, snap.as_bytes(), snap.width(), snap.height(), image::ColorType::Rgba8).unwrap();
            slint::quit_event_loop().unwrap();
        });
        t
    });
    app.run().unwrap();
}
