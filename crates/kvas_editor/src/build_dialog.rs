//! The "Build Game" window: icon, name, window and graphics options of the standalone game.

use crate::editor::Editor;
use crate::i18n::t;
use crate::theme;
use eframe::egui::{self, RichText};

/// Window state.
#[derive(Default)]
pub struct BuildDialog {
    pub open: bool,
    icon_texture: Option<(String, egui::TextureHandle)>,
}

impl Editor {
    /// Opens the build window (the toolbar and File menu call this instead of building directly).
    pub fn open_build_dialog(&mut self) {
        self.build_dialog.open = true;
    }

    pub fn build_dialog_ui(&mut self, ctx: &egui::Context) {
        if !self.build_dialog.open {
            return;
        }
        let mut open = true;
        let mut start = false;
        let mut changed = false;
        let mut settings = self.project.config.build.clone();
        egui::Window::new(t("Build Game")).open(&mut open).collapsible(false).resizable(false).default_width(470.0).default_pos([260.0, 120.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Icon preview and picker.
                let (rect, _) = ui.allocate_exact_size(egui::vec2(96.0, 96.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 16.0, theme::pal().field);
                let key = settings.icon.clone();
                if self.build_dialog.icon_texture.as_ref().is_none_or(|(k, _)| *k != key) {
                    let path = if key.is_empty() {
                        None
                    } else if std::path::Path::new(&key).is_absolute() {
                        Some(std::path::PathBuf::from(&key))
                    } else {
                        Some(self.project.root.join(&key))
                    };
                    let image = path
                        .and_then(|p| image::open(p).ok())
                        .unwrap_or_else(|| image::load_from_memory(include_bytes!("../assets/logo-256.png")).expect("logo"))
                        .resize(128, 128, image::imageops::FilterType::Triangle)
                        .to_rgba8();
                    let size = [image.width() as usize, image.height() as usize];
                    let tex = ctx.load_texture("build-icon", egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()), Default::default());
                    self.build_dialog.icon_texture = Some((key, tex));
                }
                if let Some((_, tex)) = &self.build_dialog.icon_texture {
                    ui.painter().image(tex.id(), rect.shrink(6.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                }
                ui.vertical(|ui| {
                    ui.label(RichText::new(t("Application icon")).family(theme::bold_family()));
                    ui.weak(t("PNG, JPG or ICO. Shown on the exe file, taskbar and window."));
                    ui.horizontal(|ui| {
                        if ui.button(t("Choose icon...")).clicked() {
                            if let Some(path) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "ico"]).pick_file() {
                                settings.icon = path.strip_prefix(&self.project.root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| path.to_string_lossy().to_string());
                                changed = true;
                            }
                        }
                        if !settings.icon.is_empty() && ui.button(t("Use default")).clicked() {
                            settings.icon.clear();
                            changed = true;
                        }
                    });
                    if !settings.icon.is_empty() {
                        ui.weak(&settings.icon);
                    }
                });
            });
            ui.add_space(8.0);
            egui::Grid::new("build-grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                let mut text = |ui: &mut egui::Ui, label: &'static str, value: &mut String, hint: &str| {
                    ui.label(t(label));
                    changed |= ui.add(egui::TextEdit::singleline(value).hint_text(hint).desired_width(280.0)).changed();
                    ui.end_row();
                };
                text(ui, "Window title", &mut settings.title, &self.project.config.name);
                text(ui, "File name", &mut settings.exe_name, &self.project.config.name);
                text(ui, "Version", &mut settings.version, "1.0.0");
                text(ui, "Company", &mut settings.company, "");
                ui.label(t("Window size"));
                ui.horizontal(|ui| {
                    changed |= ui.add(egui::DragValue::new(&mut settings.width).range(320..=7680).suffix(" px")).changed();
                    ui.label("×");
                    changed |= ui.add(egui::DragValue::new(&mut settings.height).range(240..=4320).suffix(" px")).changed();
                });
                ui.end_row();
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                changed |= ui.checkbox(&mut settings.fullscreen, t("Fullscreen")).changed();
                changed |= ui.checkbox(&mut settings.maximized, t("Maximized")).changed();
                changed |= ui.checkbox(&mut settings.resizable, t("Resizable")).changed();
                changed |= ui.checkbox(&mut settings.vsync, t("VSync")).changed();
            });
            changed |= ui.checkbox(&mut settings.release, t("Optimised (release) build")).on_hover_text(t("Slower to build, much faster to run. Turn off for quick test builds.")).changed();
            ui.add_space(4.0);
            ui.weak(t("Graphics quality (upscaler, shadows, post effects, fog) is taken from the scene settings."));
            ui.weak(t("After building, the game is started for a few seconds to check that it runs."));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add_enabled(self.build.is_none(), egui::Button::new(RichText::new(t("Build")).color(egui::Color32::WHITE)).fill(theme::ACCENT)).clicked() {
                    start = true;
                }
                if self.build.is_some() {
                    ui.spinner();
                    ui.label(t("Building game..."));
                }
                if self.project.root.join("build").exists() && ui.button(t("Open build folder")).clicked() {
                    crate::assets_panel::open_external(&self.project.root.join("build"));
                }
            });
        });
        if changed {
            self.project.config.build = settings;
            if let Err(e) = self.project.save() {
                self.console.error(e.to_string());
            }
        }
        if start {
            self.start_game_build();
        }
        self.build_dialog.open = open;
    }
}
