use eframe::egui::{
    self, text::LayoutJob, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText,
    TextFormat,
};
use kk_adf::{Doc, Node};
use std::sync::Arc;

const SAMPLE: &str = include_str!("../../sample.json");
const TREE: &str = include_str!("../../tree.txt");

struct App {
    doc: Doc,
    tree: Vec<(usize, &'static str)>,
    sel: usize,
    draft: String,
    frame: u32,
    shot: Option<String>,
}

fn family(strong: bool, em: bool) -> FontFamily {
    match (strong, em) {
        (false, false) => FontFamily::Proportional,
        (true, false) => FontFamily::Name("bold".into()),
        (false, true) => FontFamily::Name("italic".into()),
        (true, true) => FontFamily::Name("bolditalic".into()),
    }
}

// egui ships no bold/italic faces; load them from the system font collection.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/Helvetica.ttc") {
        for (name, index) in [("bold", 1), ("italic", 2), ("bolditalic", 3)] {
            let mut data = FontData::from_owned(bytes.clone());
            data.index = index;
            fonts.font_data.insert(name.into(), Arc::new(data));
            fonts.families.insert(FontFamily::Name(name.into()), vec![name.into()]);
        }
    }
    ctx.set_fonts(fonts);
}

fn inline_job(nodes: &[Node], size: f32, base_strong: bool, ctx: &egui::Context) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    let color = ctx.global_style().visuals.text_color();
    for n in nodes {
        let marks = n.marks.as_deref().unwrap_or(&[]);
        let has = |k: &str| marks.iter().any(|m| m.kind == k);
        let mut f = TextFormat {
            font_id: FontId::new(size, family(base_strong || has("strong"), has("em"))),
            color,
            ..Default::default()
        };
        if has("code") {
            f.font_id = FontId::monospace(size * 0.9);
            f.background = Color32::from_gray(128).gamma_multiply(0.25);
        }
        if has("link") {
            f.color = Color32::from_rgb(0x1d, 0x6f, 0xe0);
            f.underline = egui::Stroke::new(1.0, f.color);
        }
        match n.kind.as_str() {
            "text" => job.append(n.text.as_deref().unwrap_or(""), 0.0, f),
            "hardBreak" => job.append("\n", 0.0, f),
            _ => {}
        }
    }
    job
}

fn kids(n: &Node) -> &[Node] {
    n.content.as_deref().unwrap_or(&[])
}

fn block(ui: &mut egui::Ui, n: &Node, depth: usize) {
    let ctx = ui.ctx().clone();
    match n.kind.as_str() {
        "heading" => {
            let lvl = n.attrs.as_ref().and_then(|a| a["level"].as_u64()).unwrap_or(1);
            let size = [28.0, 22.0, 18.0, 16.0, 15.0, 14.0][(lvl as usize - 1).min(5)];
            ui.add_space(6.0);
            ui.add(egui::Label::new(inline_job(kids(n), size, true, &ctx)).wrap());
        }
        "paragraph" => {
            ui.add(egui::Label::new(inline_job(kids(n), 14.0, false, &ctx)).wrap());
        }
        "bulletList" | "orderedList" => {
            for (i, item) in kids(n).iter().enumerate() {
                let marker = if n.kind == "bulletList" { "•".to_string() } else { format!("{}.", i + 1) };
                ui.horizontal_top(|ui| {
                    ui.add_space(12.0 * depth as f32);
                    ui.label(marker);
                    ui.vertical(|ui| {
                        for c in kids(item) {
                            block(ui, c, depth + 1);
                        }
                    });
                });
            }
        }
        "codeBlock" => {
            let text: String = kids(n).iter().filter_map(|t| t.text.clone()).collect();
            egui::Frame::new()
                .fill(Color32::from_gray(128).gamma_multiply(0.15))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(text).monospace());
                });
        }
        "blockquote" => {
            ui.horizontal_top(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(3.0, 0.0), egui::Sense::hover());
                let start = r.min;
                ui.vertical(|ui| {
                    for c in kids(n) {
                        block(ui, c, depth + 1);
                    }
                });
                let end = ui.min_rect().bottom();
                ui.painter().line_segment(
                    [start, egui::pos2(start.x, end)],
                    egui::Stroke::new(3.0, Color32::from_gray(150)),
                );
            });
        }
        "rule" => {
            ui.separator();
        }
        "table" => {
            egui::Grid::new(format!("t{depth}")).striped(true).num_columns(3).min_col_width(80.0).show(ui, |ui| {
                for row in kids(n) {
                    for cell in kids(row) {
                        ui.vertical(|ui| {
                            for c in kids(cell) {
                                block(ui, c, depth + 1);
                            }
                        });
                    }
                    ui.end_row();
                }
            });
        }
        other => {
            ui.label(format!("[unsupported {other}]"));
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !ctx.egui_wants_keyboard_input() {
            let (up, down) = ctx.input(|i| (i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown)));
            if up && self.sel > 0 { self.sel -= 1 }
            if down && self.sel + 1 < self.tree.len() { self.sel += 1 }
        }
        egui::Panel::left("tree").resizable(true).default_size(200.0).show_inside(ui, |ui| {
            ui.heading("Pages");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, (d, t)) in self.tree.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_space(14.0 * *d as f32);
                        if ui.selectable_label(self.sel == i, *t).clicked() {
                            self.sel = i;
                        }
                    });
                }
            });
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                ui.heading(self.tree[self.sel].1);
                ui.separator();
                let doc = self.doc.clone();
                for n in &doc.content {
                    block(ui, n, 0);
                    ui.add_space(6.0);
                }
                ui.separator();
                ui.label("Edit paragraph");
                ui.add(egui::TextEdit::multiline(&mut self.draft).desired_width(f32::INFINITY).desired_rows(5));
            });
        });
        self.frame += 1;
        if let Some(path) = self.shot.clone() {
            if self.frame == 20 {
                ctx.request_screenshot(move |img| {
                    let [w, h] = img.size;
                    image::save_buffer(&path, img.as_raw(), w as u32, h as u32, image::ColorType::Rgba8).unwrap();
                    std::process::exit(0);
                });
            }
            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let width: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000.0);
    let shot = args.get(2).cloned();
    eframe::run_native(
        "kaijukit spike",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([width, 700.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            Ok(Box::new(App {
                doc: Doc::parse(SAMPLE).unwrap(),
                tree: TREE.lines().map(|l| { let (d, t) = l.split_once(' ').unwrap(); (d.parse().unwrap(), t) }).collect(),
                sel: 1,
                draft: "Edit me. Select with mouse or shift+arrows, copy, paste, undo with Cmd+Z.".into(),
                frame: 0,
                shot,
            }))
        }),
    )
}
