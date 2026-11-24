use iced::font::{Style, Weight};
use iced::widget::{
    button, column, container, pane_grid, rich_text, row, scrollable, span, text, text_editor,
};
use iced::{event, keyboard, window, Background, Border, Color, Element, Font, Length, Task};
use kk_adf::{Doc, Node};

const SAMPLE: &str = include_str!("../../sample.json");
const TREE: &str = include_str!("../../tree.txt");

#[derive(Debug, Clone)]
enum Msg {
    Select(usize),
    Key(keyboard::Key),
    Edit(text_editor::Action),
    Undo,
    Redo,
    Resize(pane_grid::ResizeEvent),
    Link(String),
    Capture,
    Shot(window::Screenshot),
}

#[derive(Clone, Copy)]
enum Pane {
    Tree,
    Content,
}

struct App {
    doc: Doc,
    tree: Vec<(usize, &'static str)>,
    sel: usize,
    content: text_editor::Content,
    // iced's text_editor has no undo; whole-text snapshots lose caret and selection.
    undo: Vec<String>,
    redo: Vec<String>,
    panes: pane_grid::State<Pane>,
    shot: Option<String>,
}

fn kids(n: &Node) -> &[Node] {
    n.content.as_deref().unwrap_or(&[])
}

fn spans<'a>(nodes: &'a [Node], size: f32, bold: bool) -> Vec<text::Span<'a, String, Font>> {
    nodes
        .iter()
        .filter_map(|n| {
            let marks = n.marks.as_deref().unwrap_or(&[]);
            let has = |k: &str| marks.iter().any(|m| m.kind == k);
            let mut f = Font::with_name("Helvetica Neue");
            if bold || has("strong") {
                f.weight = Weight::Bold;
            }
            if has("em") {
                f.style = Style::Italic;
            }
            let t = match n.kind.as_str() {
                "text" => n.text.clone().unwrap_or_default(),
                "hardBreak" => "\n".into(),
                _ => return None,
            };
            let mut s = span(t).size(size).font(f);
            if has("code") {
                s = s
                    .font(Font::MONOSPACE)
                    .background(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.25)))
                    .padding([0, 2]);
            }
            if let Some(m) = marks.iter().find(|m| m.kind == "link") {
                let href = m.attrs.as_ref().and_then(|a| a["href"].as_str()).unwrap_or("").to_string();
                s = s.color(Color::from_rgb8(0x1d, 0x6f, 0xe0)).underline(true).link(href);
            }
            Some(s)
        })
        .collect()
}

fn border() -> Border {
    Border { color: Color::from_rgb(0.7, 0.7, 0.7), width: 1.0, radius: 0.into() }
}

fn block<'a>(n: &'a Node) -> Element<'a, Msg> {
    match n.kind.as_str() {
        "heading" => {
            let lvl = n.attrs.as_ref().and_then(|a| a["level"].as_u64()).unwrap_or(1) as usize;
            let size = [28.0, 22.0, 18.0, 16.0, 15.0, 14.0][(lvl - 1).min(5)];
            rich_text(spans(kids(n), size, true)).on_link_click(Msg::Link).into()
        }
        "paragraph" => rich_text(spans(kids(n), 14.0, false)).on_link_click(Msg::Link).into(),
        "bulletList" | "orderedList" => column(kids(n).iter().enumerate().map(|(i, item)| {
            let marker = if n.kind == "bulletList" { "•".to_string() } else { format!("{}.", i + 1) };
            row![text(marker).width(20), column(kids(item).iter().map(block)).spacing(4)].into()
        }))
        .spacing(4)
        .into(),
        "codeBlock" => {
            let t: String = kids(n).iter().filter_map(|t| t.text.clone()).collect();
            container(text(t).font(Font::MONOSPACE))
                .padding(8)
                .width(Length::Fill)
                .style(|_| container::Style::default().background(Color::from_rgba(0.5, 0.5, 0.5, 0.15)))
                .into()
        }
        "blockquote" => row![
            container("")
                .width(3)
                .height(Length::Fill)
                .style(|_| container::Style::default().background(Color::from_rgb(0.6, 0.6, 0.6))),
            container(column(kids(n).iter().map(block))).padding([0, 10])
        ]
        .height(Length::Shrink)
        .into(),
        "rule" => container("")
            .width(Length::Fill)
            .height(1)
            .style(|_| container::Style::default().background(Color::from_rgb(0.7, 0.7, 0.7)))
            .into(),
        "table" => column(kids(n).iter().map(|r| {
            row(kids(r).iter().map(|c| {
                container(column(kids(c).iter().map(block)))
                    .padding(6)
                    .width(Length::FillPortion(1))
                    .style(|_| container::Style::default().border(border()))
                    .into()
            }))
            .into()
        }))
        .into(),
        other => text(format!("[unsupported {other}]")).into(),
    }
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let (mut panes, tree) = pane_grid::State::new(Pane::Tree);
        let (_, split) = panes.split(pane_grid::Axis::Vertical, tree, Pane::Content).unwrap();
        panes.resize(split, 0.25);
        let shot = std::env::args().nth(2);
        let task = if shot.is_some() {
            Task::future(async {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                Msg::Capture
            })
        } else {
            Task::none()
        };
        (
            Self {
                doc: Doc::parse(SAMPLE).unwrap(),
                tree: TREE
                    .lines()
                    .map(|l| {
                        let (d, t) = l.split_once(' ').unwrap();
                        (d.parse().unwrap(), t)
                    })
                    .collect(),
                sel: 1,
                content: text_editor::Content::with_text(
                    "Edit me. Select with mouse or shift+arrows, copy, paste, undo with Cmd+Z.",
                ),
                undo: vec![],
                redo: vec![],
                panes,
                shot,
            },
            task,
        )
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Select(i) => self.sel = i,
            Msg::Key(k) => match k {
                keyboard::Key::Named(keyboard::key::Named::ArrowUp) if self.sel > 0 => self.sel -= 1,
                keyboard::Key::Named(keyboard::key::Named::ArrowDown) if self.sel + 1 < self.tree.len() => {
                    self.sel += 1
                }
                _ => {}
            },
            Msg::Edit(a) => {
                if a.is_edit() {
                    self.undo.push(self.content.text());
                    self.redo.clear();
                }
                self.content.perform(a);
            }
            Msg::Undo => {
                if let Some(t) = self.undo.pop() {
                    self.redo.push(self.content.text());
                    self.content = text_editor::Content::with_text(&t);
                }
            }
            Msg::Redo => {
                if let Some(t) = self.redo.pop() {
                    self.undo.push(self.content.text());
                    self.content = text_editor::Content::with_text(&t);
                }
            }
            Msg::Resize(e) => self.panes.resize(e.split, e.ratio),
            Msg::Link(_) => {}
            Msg::Capture => return window::oldest().and_then(window::screenshot).map(Msg::Shot),
            Msg::Shot(s) => {
                image::save_buffer(self.shot.as_ref().unwrap(), &s.rgba, s.size.width, s.size.height, image::ColorType::Rgba8)
                    .unwrap();
                std::process::exit(0);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Msg> {
        pane_grid(&self.panes, |_, pane, _| {
            let body: Element<'_, Msg> = match pane {
                Pane::Tree => scrollable(
                    column(self.tree.iter().enumerate().map(|(i, (d, t))| {
                        button(text(*t))
                            .style(if i == self.sel { button::primary } else { button::text })
                            .padding([4, 8 + 14 * *d as u16])
                            .width(Length::Fill)
                            .on_press(Msg::Select(i))
                            .into()
                    }))
                    .padding(4),
                )
                .into(),
                Pane::Content => column![
                    scrollable(
                        column![
                            text(self.tree[self.sel].1).size(14),
                            column(self.doc.content.iter().map(block)).spacing(10),
                        ]
                        .spacing(10)
                        .padding(16),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill),
                    column![
                        text("Edit paragraph"),
                        text_editor(&self.content).height(90).on_action(Msg::Edit).key_binding(|kp| {
                            if kp.modifiers.command() && kp.key.as_ref() == keyboard::Key::Character("z") {
                                Some(text_editor::Binding::Custom(if kp.modifiers.shift() {
                                    Msg::Redo
                                } else {
                                    Msg::Undo
                                }))
                            } else {
                                text_editor::Binding::from_key_press(kp)
                            }
                        }),
                    ]
                    .spacing(6)
                    .padding(16),
                ]
                .into(),
            };
            pane_grid::Content::new(body)
        })
        .on_resize(6, Msg::Resize)
        .into()
    }
}

fn main() -> iced::Result {
    let w: f32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1000.0);
    iced::application(App::new, App::update, App::view)
        .title("kaijukit spike")
        .window_size((w, 700.0))
        .subscription(|_| {
            event::listen_with(|e, status, _| match (e, status) {
                (iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }), event::Status::Ignored) => {
                    Some(Msg::Key(key))
                }
                _ => None,
            })
        })
        .run()
}
