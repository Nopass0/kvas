//! Settings window (JetBrains style: category list on the left, page on the right). Editor
//! preferences apply and save immediately; project settings are saved with the Save button.

use crate::editor::Editor;
use crate::i18n::{self, t, tf, LANGUAGES};
use crate::icons::{self, Icon};
use crate::prefs::Prefs;
use crate::theme::{self, pal, Theme};
use eframe::egui::{self, RichText};
use kvas_engine::input::Source;
use kvas_engine::project::{default_bindings, InputBinding};

#[derive(Default)]
pub struct SettingsState {
    pub open: bool,
    pub page: Page,
    /// Action index waiting for the next key / mouse press.
    listening: Option<usize>,
    pub dirty: bool,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Page {
    #[default]
    Appearance,
    CodeEditor,
    General,
    Input,
    Simulation,
    Audio,
    Graphics,
    Build,
    Viewport,
    Shortcuts,
}

impl Page {
    fn is_project(self) -> bool {
        matches!(self, Page::General | Page::Input | Page::Simulation | Page::Audio | Page::Build)
    }

    fn title(self) -> &'static str {
        match self {
            Page::Appearance => t("Appearance"),
            Page::CodeEditor => t("Code editor"),
            Page::General => t("General"),
            Page::Input => t("Input"),
            Page::Simulation => t("Simulation"),
            Page::Audio => t("Sound"),
            Page::Graphics => t("Rendering"),
            Page::Build => t("Build Game"),
            Page::Viewport => t("Scene"),
            Page::Shortcuts => t("Shortcuts"),
        }
    }
}

fn pressed_source(ctx: &egui::Context) -> Option<&'static str> {
    ctx.input(|i| {
        for e in &i.events {
            match e {
                egui::Event::Key { key, pressed: true, repeat: false, .. } => {
                    if let Some(s) = Source::parse(key.name()) {
                        return Some(s.name());
                    }
                }
                egui::Event::PointerButton { button, pressed: true, .. } => {
                    return Some(match button {
                        egui::PointerButton::Primary => "MouseLeft",
                        egui::PointerButton::Secondary => "MouseRight",
                        _ => "MouseMiddle",
                    });
                }
                _ => {}
            }
        }
        None
    })
}

/// Language, theme and scale. Applies the change to the running UI and saves it. Shared by the
/// settings window and the project hub.
pub fn appearance_ui(ui: &mut egui::Ui, prefs: &mut Prefs) {
    let ctx = ui.ctx().clone();
    let mut changed = false;
    egui::Grid::new("appearance").num_columns(2).spacing([16.0, 12.0]).show(ui, |ui| {
        ui.label(t("Language"));
        egui::ComboBox::from_id_salt("lang").selected_text(prefs.language.name()).width(180.0).show_ui(ui, |ui| {
            for (lang, name) in LANGUAGES {
                if ui.selectable_label(prefs.language == *lang, *name).clicked() {
                    prefs.language = *lang;
                    changed = true;
                }
            }
        });
        ui.end_row();

        ui.label(t("Theme"));
        ui.horizontal(|ui| {
            for th in [Theme::Dark, Theme::Light] {
                if ui.selectable_label(prefs.theme == th, th.name()).clicked() {
                    prefs.theme = th;
                    changed = true;
                }
            }
        });
        ui.end_row();

        ui.label(t("Interface scale"));
        ui.horizontal(|ui| {
            // The scale is applied on release so the slider does not jump under the cursor.
            let id = ui.scope_id().with("scale-pending");
            let mut pct = ui.data(|d| d.get_temp::<i32>(id)).unwrap_or((prefs.ui_scale * 100.0).round() as i32);
            let r = ui.add(egui::Slider::new(&mut pct, 75..=200).suffix("%").step_by(5.0));
            ui.data_mut(|d| d.insert_temp(id, pct));
            if r.drag_stopped() || (r.changed() && !r.dragged()) {
                prefs.ui_scale = pct as f32 / 100.0;
                ui.data_mut(|d| d.remove_temp::<i32>(id));
                changed = true;
            }
            if ui.small_button(t("Reset")).clicked() && prefs.ui_scale != 1.0 {
                prefs.ui_scale = 1.0;
                ui.data_mut(|d| d.remove_temp::<i32>(id));
                changed = true;
            }
        });
        ui.end_row();
    });
    if changed {
        i18n::set(prefs.language);
        theme::apply(&ctx, prefs.theme, prefs.ui_scale);
        prefs.save();
    }
}

fn nav_item(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
    crate::ui_kit::tile(ui, rect, resp.id, resp.hovered(), selected);
    let color = if selected { pal().text } else { pal().text_dim.gamma_multiply(1.25) };
    ui.painter().text(rect.left_center() + egui::vec2(12.0, 0.0), egui::Align2::LEFT_CENTER, label, egui::FontId::proportional(13.0), color);
    resp
}

fn nav_heading(ui: &mut egui::Ui, label: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(label).size(11.5).family(crate::theme::bold_family()).color(pal().text_dim));
    ui.add_space(2.0);
}

impl Editor {
    pub fn settings_ui(&mut self, ctx: &egui::Context) {
        if !self.settings.open {
            return;
        }
        let mut open = true;
        egui::Window::new(t("Settings")).open(&mut open).default_size([780.0, 500.0]).collapsible(false).default_pos([260.0, 120.0]).show(ctx, |ui| {
            let page = self.settings.page;
            egui::Panel::bottom("settings-footer").frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 0, top: 8, bottom: 0 })).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if page.is_project() {
                        ui.weak(t("Project settings are stored in project.json."));
                    } else {
                        ui.weak(t("Changes are applied and saved automatically."));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t("Close")).clicked() {
                            self.settings.open = false;
                        }
                        if page.is_project() && ui.add_enabled(self.settings.dirty, egui::Button::new(RichText::new(t("Save")).color(egui::Color32::WHITE)).fill(theme::ACCENT)).clicked() {
                            let errors = self.project.config.validate();
                            if errors.is_empty() { match self.project.save() {
                                Ok(()) => {
                                    self.settings.dirty = false;
                                    self.status = t("Project settings saved").into();
                                }
                                Err(e) => self.console.error(e.to_string()),
                            } } else { self.console.error(errors.join("\n")); }
                        }
                    });
                });
            });
            egui::Panel::left("settings-nav").exact_size(190.0).resizable(false).frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 0, right: 10, top: 0, bottom: 0 })).show(ui, |ui| {
                nav_heading(ui, t("EDITOR"));
                for p in [Page::Appearance, Page::CodeEditor, Page::Viewport, Page::Shortcuts] {
                    if nav_item(ui, p.title(), self.settings.page == p).clicked() {
                        self.settings.page = p;
                    }
                }
                nav_heading(ui, t("PROJECT"));
                for p in [Page::General, Page::Input, Page::Simulation, Page::Audio, Page::Build, Page::Graphics] {
                    if nav_item(ui, p.title(), self.settings.page == p).clicked() {
                        self.settings.page = p;
                    }
                }
            });
            egui::CentralPanel::default().frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 14, right: 0, top: 0, bottom: 0 })).show(ui, |ui| {
                ui.label(RichText::new(self.settings.page.title()).size(18.0).family(crate::theme::bold_family()));
                ui.add_space(8.0);
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| match self.settings.page {
                    Page::Appearance => appearance_ui(ui, &mut self.prefs),
                    Page::CodeEditor => self.settings_code_editor(ui),
                    Page::General => self.settings_general(ui),
                    Page::Input => self.settings_input(ui, ctx),
                    page => self.settings_extra(ui, page),
                });
            });
        });
        if !open {
            self.settings.open = false;
        }
        if !self.settings.open {
            self.settings.listening = None;
        }
    }

    fn settings_general(&mut self, ui: &mut egui::Ui) {
        let scenes: Vec<String> = self.asset_list.iter().filter(|a| a.ends_with(".scene.json")).cloned().collect();
        let cfg = &mut self.project.config;
        let mut changed = false;
        egui::Grid::new("settings-general").num_columns(2).spacing([16.0, 10.0]).show(ui, |ui| {
            ui.label(t("Project name"));
            changed |= ui.add(egui::TextEdit::singleline(&mut cfg.name).desired_width(260.0)).changed();
            ui.end_row();
            ui.label(t("Start scene"));
            egui::ComboBox::from_id_salt("start-scene").selected_text(&cfg.start_scene).width(260.0).show_ui(ui, |ui| {
                for s in &scenes {
                    if ui.selectable_label(cfg.start_scene == *s, s).clicked() {
                        cfg.start_scene = s.clone();
                        changed = true;
                    }
                }
            });
            ui.end_row();
            ui.label(t("Project type"));
            ui.label(&cfg.kind);
            ui.end_row();
            ui.label(t("Scripts package"));
            ui.label(&cfg.scripts_package);
            ui.end_row();
        });
        self.settings.dirty |= changed;
    }

    fn settings_input(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.weak(t("Actions are read in scripts: ctx.action_down(\"Jump\"), ctx.action_pressed(..), ctx.action_axis(..). Scripts can rebind them with ctx.rebind(..)."));
        ui.add_space(6.0);

        if let Some(idx) = self.settings.listening {
            if let Some(name) = pressed_source(ctx) {
                if let Some(b) = self.project.config.input.get_mut(idx) {
                    if !b.keys.iter().any(|k| k == name) {
                        b.keys.push(name.to_string());
                    }
                }
                self.settings.listening = None;
                self.settings.dirty = true;
            }
        }

        let listening = self.settings.listening;
        let mut remove_action: Option<usize> = None;
        let mut listen: Option<usize> = None;
        let mut changed = false;
        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            for (i, b) in self.project.config.input.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    changed |= ui.add(egui::TextEdit::singleline(&mut b.name).desired_width(120.0)).changed();
                    let mut drop_key: Option<usize> = None;
                    for (k, key) in b.keys.iter().enumerate() {
                        let chip = egui::Button::new(RichText::new(format!("{key}  \u{d7}")).size(12.0)).fill(pal().hover).corner_radius(10);
                        if ui.add(chip).on_hover_text(t("Click to remove")).clicked() {
                            drop_key = Some(k);
                        }
                    }
                    if let Some(k) = drop_key {
                        b.keys.remove(k);
                        changed = true;
                    }
                    let waiting = listening == Some(i);
                    if ui.add(egui::Button::new(if waiting { t("Press a key...") } else { t("+ Add") }).selected(waiting)).clicked() {
                        listen = Some(i);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icons::button(ui, Icon::Trash2, t("Delete action"), false).clicked() {
                            remove_action = Some(i);
                        }
                    });
                });
            }
        });
        if let Some(i) = listen {
            self.settings.listening = Some(i);
        }
        if let Some(i) = remove_action {
            self.project.config.input.remove(i);
            changed = true;
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t("Add action")).clicked() {
                self.project.config.input.push(InputBinding { name: tf("Action{}", &[&(self.project.config.input.len() + 1).to_string()]), keys: vec![] });
                changed = true;
            }
            if ui.button(t("Reset to defaults")).clicked() {
                self.project.config.input = default_bindings();
                changed = true;
            }
        });
        self.settings.dirty |= changed;
    }

    fn settings_code_editor(&mut self, ui: &mut egui::Ui) {
        ui.weak(t("Opens scripts, shaders and the project folder. Saved for you, not in the project."));
        ui.add_space(8.0);
        let mut changed = false;
        let current = crate::prefs::CODE_EDITORS.iter().find(|(_, c)| *c == self.prefs.code_editor).map(|(n, _)| t(n)).unwrap_or(t("Custom command"));
        egui::Grid::new("code-editor").num_columns(2).spacing([16.0, 10.0]).show(ui, |ui| {
            ui.label(t("Editor"));
            egui::ComboBox::from_id_salt("code-editor-combo").selected_text(current).width(220.0).show_ui(ui, |ui| {
                for (name, cmd) in crate::prefs::CODE_EDITORS {
                    if ui.selectable_label(self.prefs.code_editor == *cmd, t(name)).clicked() {
                        self.prefs.code_editor = cmd.to_string();
                        changed = true;
                    }
                }
            });
            ui.end_row();
            ui.label(t("Command"));
            changed |= ui.add(egui::TextEdit::singleline(&mut self.prefs.code_editor).hint_text(t("empty = system default")).desired_width(220.0)).changed();
            ui.end_row();
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button(t("Open project in editor")).clicked() {
                self.prefs.open(&self.project.root);
            }
            if ui.button(t("Open scripts")).clicked() {
                self.prefs.open(&self.scripts_file());
            }
        });
        if changed {
            self.prefs.save();
        }
    }
}
