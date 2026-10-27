//! Project hub shown when the editor starts without a project: recent projects, open, create.

use crate::export;
use crate::i18n::t;
use crate::icons::{self, Icon};
use crate::prefs::Prefs;
use crate::theme::{self, pal};
use eframe::egui::{self, Color32, RichText};
use std::path::{Path, PathBuf};

fn config_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))).unwrap_or_else(|| PathBuf::from("."));
    base.join("KvasEngine")
}

fn recent_file() -> PathBuf {
    config_dir().join("recent.json")
}

pub fn recent_projects() -> Vec<PathBuf> {
    std::fs::read_to_string(recent_file()).ok().and_then(|t| serde_json::from_str::<Vec<String>>(&t).ok()).unwrap_or_default().into_iter().map(PathBuf::from).filter(|p| p.join("project.json").exists()).collect()
}

/// Moves a project to the top of the recent list.
pub fn add_recent(root: &Path) {
    let root = root.to_string_lossy().to_string();
    let mut list: Vec<String> = recent_projects().into_iter().map(|p| p.to_string_lossy().to_string()).filter(|p| *p != root).collect();
    list.insert(0, root);
    list.truncate(12);
    let _ = std::fs::create_dir_all(config_dir());
    let _ = std::fs::write(recent_file(), serde_json::to_string_pretty(&list).unwrap_or_default());
}

fn default_location() -> String {
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_default();
    home.join("Documents").join("Kvas Projects").to_string_lossy().to_string()
}

/// Opens the editor on a project in a new process, so each project gets a fresh window.
fn launch(root: &Path) {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).arg(root).spawn();
    }
}

#[derive(PartialEq)]
enum Page {
    Projects,
    New,
    Settings,
}

pub struct Hub {
    page: Page,
    recent: Vec<PathBuf>,
    search: String,
    name: String,
    location: String,
    kind: &'static str,
    error: String,
    prefs: Prefs,
}

impl Hub {
    pub fn new(prefs: Prefs) -> Self {
        Self { page: Page::Projects, recent: recent_projects(), search: String::new(), name: "My Game".into(), location: default_location(), kind: "3D", error: String::new(), prefs }
    }

    fn open_and_exit(&self, ctx: &egui::Context, root: &Path) {
        launch(root);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

fn nav_button(ui: &mut egui::Ui, icon: Icon, label: &str, selected: bool) -> egui::Response {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
    crate::ui_kit::tile(ui, rect, resp.id, resp.hovered(), selected);
    let tint = if selected { theme::ACCENT } else { p.text_dim };
    icon.image(16.0, tint).paint_at(ui, egui::Rect::from_center_size(rect.left_center() + egui::vec2(18.0, 0.0), egui::Vec2::splat(16.0)));
    ui.painter().text(rect.left_center() + egui::vec2(36.0, 0.0), egui::Align2::LEFT_CENTER, label, egui::FontId::proportional(13.5), if selected { p.text } else { p.text.gamma_multiply(0.9) });
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(label).color(Color32::WHITE)).fill(theme::ACCENT).stroke(egui::Stroke::NONE).min_size(egui::vec2(0.0, 28.0)))
}

impl eframe::App for Hub {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let p = pal();
        egui::Panel::top("hub-title").frame(egui::Frame::NONE.fill(p.bar)).exact_size(36.0).show(ui, |ui| {
            let bar = egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.available_width(), 36.0));
            let drag = ui.interact(bar, ui.scope_id().with("hub-drag"), egui::Sense::click_and_drag());
            if drag.drag_started_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            ui.painter().text(bar.center(), egui::Align2::CENTER_CENTER, t("Welcome to Kvas"), egui::FontId::proportional(13.0), p.text_dim);
            ui.scope_builder(egui::UiBuilder::new().max_rect(bar).layout(egui::Layout::right_to_left(egui::Align::Center)), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (icon, danger) in [(Icon::X, true), (Icon::Minus, false)] {
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(46.0, 36.0), egui::Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(r, 0.0, if danger { Color32::from_rgb(196, 43, 28) } else { p.hover });
                    }
                    let tint = if danger && resp.hovered() { Color32::WHITE } else { p.text };
                    icon.image(13.0, tint).paint_at(ui, egui::Rect::from_center_size(r.center(), egui::vec2(13.0, 13.0)));
                    if resp.clicked() {
                        ctx.send_viewport_cmd(if danger { egui::ViewportCommand::Close } else { egui::ViewportCommand::Minimized(true) });
                    }
                }
            });
            ui.allocate_rect(bar, egui::Sense::hover());
        });

        egui::Panel::left("hub-side").exact_size(230.0).resizable(false).frame(egui::Frame::NONE.fill(p.panel).inner_margin(egui::Margin::symmetric(12, 16))).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(icons::logo(44.0));
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    ui.label(RichText::new("Kvas").size(20.0).family(crate::theme::bold_family()));
                    ui.label(RichText::new(format!("Engine {}", env!("CARGO_PKG_VERSION"))).size(11.5).color(p.text_dim));
                });
            });
            ui.add_space(20.0);
            for (page, icon, label) in [(Page::Projects, Icon::FolderOpen, t("Projects")), (Page::New, Icon::Plus, t("New project")), (Page::Settings, Icon::Settings, t("Settings"))] {
                let selected = self.page == page;
                if nav_button(ui, icon, label, selected).clicked() {
                    self.page = page;
                }
            }
        });

        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(p.field).inner_margin(egui::Margin::symmetric(28, 22))).show(ui, |ui| match self.page {
            Page::Projects => {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.search).hint_text(t("Search projects")).desired_width(260.0).margin(egui::Margin::symmetric(8, 5)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t("Open project...")).clicked() {
                            if let Some(file) = rfd::FileDialog::new().set_title(t("Select project.json")).add_filter("Kvas project", &["json"]).pick_file() {
                                if let Some(root) = file.parent() {
                                    self.open_and_exit(&ctx, root);
                                }
                            }
                        }
                        if primary_button(ui, t("New project")).clicked() {
                            self.page = Page::New;
                        }
                    });
                });
                ui.add_space(12.0);
                if self.recent.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| ui.label(RichText::new(t("No projects yet. Create one on the left.")).color(p.text_dim)));
                }
                let query = self.search.to_lowercase();
                let mut open: Option<PathBuf> = None;
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    for (i, root) in self.recent.iter().enumerate() {
                        let (name, kind) = kvas_engine::project::Project::open(root).map(|p| (p.config.name, p.config.kind)).unwrap_or_else(|_| ("?".into(), String::new()));
                        if !query.is_empty() && !name.to_lowercase().contains(&query) && !root.to_string_lossy().to_lowercase().contains(&query) {
                            continue;
                        }
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 56.0), egui::Sense::click());
                        crate::ui_kit::tile(ui, rect, resp.id, resp.hovered(), false);
                        let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(egui::vec2(10.0, 8.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
                        crate::ui_kit::avatar(&mut inner, &name, 40.0, i as u32 + name.len() as u32);
                        inner.add_space(6.0);
                        inner.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 1.0;
                            ui.label(RichText::new(format!("{name}   {kind}")).size(14.0).family(crate::theme::bold_family()));
                            ui.label(RichText::new(root.display().to_string()).size(12.0).color(p.text_dim));
                        });
                        if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            open = Some(root.clone());
                        }
                    }
                });
                if let Some(root) = open {
                    self.open_and_exit(&ctx, &root);
                }
            }
            Page::New => {
                ui.label(RichText::new(t("New project")).size(20.0).family(crate::theme::bold_family()));
                ui.add_space(14.0);
                egui::Grid::new("hub-new").num_columns(2).spacing([16.0, 12.0]).show(ui, |ui| {
                    ui.label(t("Name"));
                    ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(320.0));
                    ui.end_row();
                    ui.label(t("Location"));
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.location).desired_width(320.0));
                        if ui.button(t("Browse...")).clicked() {
                            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                self.location = dir.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.end_row();
                    ui.label(t("Type"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.kind, "3D", t("3D game"));
                        ui.selectable_value(&mut self.kind, "2D", t("2D game"));
                    });
                    ui.end_row();
                });
                ui.add_space(14.0);
                ui.add_enabled_ui(!self.name.trim().is_empty(), |ui| {
                    if primary_button(ui, t("Create project")).clicked() {
                        let dir = PathBuf::from(self.location.trim()).join(self.name.trim());
                        match export::create_project(&dir, self.name.trim(), self.kind) {
                            Ok(p) => self.open_and_exit(&ctx, &p.root),
                            Err(e) => self.error = e,
                        }
                    }
                });
                if !self.error.is_empty() {
                    ui.colored_label(theme::red(), &self.error);
                }
            }
            Page::Settings => {
                ui.label(RichText::new(t("Appearance")).size(20.0).family(crate::theme::bold_family()));
                ui.add_space(14.0);
                crate::settings::appearance_ui(ui, &mut self.prefs);
            }
        });
    }
}

pub fn run(prefs: Prefs) -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default().with_inner_size([900.0, 560.0]).with_resizable(false).with_decorations(false).with_title("Kvas Engine");
    if let Some(icon) = icons::window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Kvas Engine",
        options,
        Box::new(move |cc| {
            theme::init(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx, prefs.theme, prefs.ui_scale);
            icons::install(&cc.egui_ctx);
            Ok(Box::new(Hub::new(prefs)))
        }),
    )
}
