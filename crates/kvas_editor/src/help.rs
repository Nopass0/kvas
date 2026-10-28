//! Searchable bundled user documentation, available without a browser/network.
use crate::editor::Editor;
use eframe::egui;

#[derive(Default)]
pub struct HelpState { pub open: bool, query: String, page: usize }
const PAGES: &[(&str, &str)] = &[
    ("Редактор", include_str!("../../../docs/EDITOR_WORKFLOWS.md")),
    ("Руководство", include_str!("../../../docs/GUIDE.md")),
    ("API", include_str!("../../../docs/ENGINE_API.md")),
    ("NVIDIA / кадры", include_str!("../../../docs/NVIDIA.md")),
    ("Ассеты", include_str!("../../../docs/ASSETS.md")),
];

impl Editor {
    pub fn help_window(&mut self, ctx: &egui::Context) {
        if !self.help.open { return; }
        let mut open = true;
        egui::Window::new("Документация Kvas").open(&mut open).default_size([900.0, 680.0]).show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| { for (i, (name, _)) in PAGES.iter().enumerate() { ui.selectable_value(&mut self.help.page, i, *name); } });
            ui.add(egui::TextEdit::singleline(&mut self.help.query).hint_text("Поиск по разделам...").desired_width(f32::INFINITY));
            ui.separator();
            let query = self.help.query.to_lowercase();
            egui::ScrollArea::vertical().id_salt(("help-content", self.help.page)).show(ui, |ui| {
                for section in PAGES[self.help.page].1.split("\n## ") {
                    if !query.is_empty() && !section.to_lowercase().contains(&query) { continue; }
                    let mut code = false;
                    for (i, line) in section.lines().enumerate() {
                        if line.starts_with("```") { code = !code; continue; }
                        if line.trim().is_empty() { ui.add_space(5.0); continue; }
                        if code { ui.monospace(line); }
                        else if i == 0 || line.starts_with('#') { ui.heading(line.trim_start_matches('#').trim()); }
                        else { ui.label(line); }
                    }
                    ui.add_space(12.0); ui.separator();
                }
            });
        });
        self.help.open = open;
    }
}
