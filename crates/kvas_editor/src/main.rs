mod assets_panel;
mod animation_editor;
mod audio_panel;
mod console;
mod editor;
mod export;
mod fields;
mod gizmo;
mod gui_editor;
mod hierarchy_panel;
mod hub;
mod help;
mod i18n;
mod icons;
mod library;
mod inspector;
mod image_document;
mod image_editor;
mod lang_ru;
mod lang_ru_nodes;
mod lang_ru_v4;
mod build_dialog;
mod code_editor;
mod wgsl_edit;
mod node_editor;
mod mesh_editor;
mod neural_editor;
mod paint_tool;
mod prefs;
mod script_files;
mod script_templates;
mod scripting;
mod settings;
mod settings_extra;
mod selection;
mod snap;
mod theme;
mod ui_kit;
mod viewport;

use eframe::egui::{self, Color32, Key, Modifiers, RichText};
use i18n::t;
use icons::Icon;
use editor::{Editor, NewProject};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use kvas_engine::project::Project;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Tab {
    Hierarchy,
    Scene,
    Inspector,
    Project,
    Console,
    Audio,
    /// The node editor of the open material / shader graph, a tab next to the Scene.
    Nodes,
}

struct Viewer<'a>(&'a mut Editor);

impl TabViewer for Viewer<'_> {
    type Tab = Tab;

    #[allow(deprecated)]
    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(match tab {
            Tab::Hierarchy => "tab-hierarchy",
            Tab::Scene => "tab-scene",
            Tab::Inspector => "tab-inspector",
            Tab::Project => "tab-project",
            Tab::Console => "tab-console",
            Tab::Audio => "tab-audio",
            Tab::Nodes => "tab-nodes",
        })
    }

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        match tab {
            Tab::Hierarchy => t("Hierarchy"),
            Tab::Scene => if self.0.is_playing() { t("Game") } else { t("Scene") },
            Tab::Inspector => t("Inspector"),
            Tab::Project => t("Project"),
            Tab::Console => t("Console"),
            Tab::Audio => t("Sound"),
            Tab::Nodes => return self.0.node_tab_title().into(),
        }
        .into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        if *tab == Tab::Scene {
            self.0.viewport_ui(ui);
            return;
        }
        if *tab == Tab::Nodes {
            self.0.node_editor_tab(ui);
            return;
        }
        egui::Frame::NONE.inner_margin(egui::Margin::same(8)).show(ui, |ui| match tab {
            Tab::Hierarchy => self.0.hierarchy_ui(ui),
            Tab::Inspector => self.0.inspector_ui(ui),
            Tab::Project => self.0.assets_ui(ui),
            Tab::Console => self.0.console.ui(ui),
            Tab::Audio => self.0.audio_ui(ui),
            Tab::Scene | Tab::Nodes => {}
        });
    }

    fn is_closeable(&self, tab: &Tab) -> bool {
        *tab == Tab::Nodes
    }

    fn on_close(&mut self, tab: &mut Tab) -> egui_dock::widgets::tab_viewer::OnCloseResponse {
        if *tab == Tab::Nodes {
            self.0.close_node_editor();
        }
        egui_dock::widgets::tab_viewer::OnCloseResponse::Close
    }

    fn scroll_bars(&self, _tab: &Tab) -> [bool; 2] {
        [false, false]
    }
}

fn default_layout() -> DockState<Tab> {
    let mut dock = DockState::new(vec![Tab::Scene]);
    let surface = dock.main_surface_mut();
    let [center, _] = surface.split_right(NodeIndex::root(), 0.77, vec![Tab::Inspector]);
    let [center, _] = surface.split_left(center, 0.2, vec![Tab::Hierarchy]);
    surface.split_below(center, 0.7, vec![Tab::Project, Tab::Console, Tab::Audio]);
    dock
}

const TITLE_H: f32 = 38.0;
const TOOLBAR_H: f32 = 36.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RunShape {
    Play,
    Stop,
}

/// The green Run / red Stop button of the toolbar (drawn, so it is a filled glyph).
fn run_button(ui: &mut egui::Ui, shape: RunShape, tip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    let hover = ui_kit::animate(ui, resp.id.with("hover"), resp.hovered());
    ui.painter().rect_filled(rect, 6.0, ui_kit::mix(Color32::TRANSPARENT, theme::pal().hover, hover));
    let c = rect.center();
    match shape {
        RunShape::Play => {
            let pts = vec![c + egui::vec2(-4.5, -6.5), c + egui::vec2(6.5, 0.0), c + egui::vec2(-4.5, 6.5)];
            ui.painter().add(egui::Shape::convex_polygon(pts, theme::green(), egui::Stroke::NONE));
        }
        RunShape::Stop => {
            let pulse = (ui.input(|i| i.time) as f32 * 3.0).sin() * 0.5 + 0.5;
            ui.painter().rect_stroke(rect.shrink(1.0), 7.0, egui::Stroke::new(1.5, theme::red().gamma_multiply(0.25 + 0.5 * pulse)), egui::StrokeKind::Inside);
            ui.painter().rect_filled(egui::Rect::from_center_size(c, egui::vec2(11.0, 11.0)), 2.0, theme::red());
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tip)
}

/// A thin vertical divider between toolbar groups.
fn separator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 20.0), egui::Sense::hover());
    ui.painter().line_segment([rect.center_top(), rect.center_bottom()], egui::Stroke::new(1.0, theme::pal().border_strong));
}

struct App {
    editor: Editor,
    dock: DockState<Tab>,
    title: String,
}

impl App {
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F1)) { self.editor.help.open = true; }
        if ctx.egui_wants_keyboard_input() || self.editor.mat.visible || self.editor.code.open || self.editor.image_editor.open {
            return;
        }
        let ed = &mut self.editor;
        let consume = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));
        if consume(Modifiers::COMMAND | Modifiers::ALT, Key::S) {
            ed.settings.open = true;
            ed.settings.page = settings::Page::Appearance;
        }
        if consume(Modifiers::COMMAND, Key::S) {
            ed.save_scene(false);
        }
        if consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z) || consume(Modifiers::COMMAND, Key::Y) {
            if ed.paint.active { ed.paint_history(false); } else { ed.redo(); }
        } else if consume(Modifiers::COMMAND, Key::Z) {
            if ed.paint.active { ed.paint_history(true); } else { ed.undo(); }
        }
        if consume(Modifiers::COMMAND, Key::P) {
            if ed.is_playing() {
                ed.stop_play();
            } else {
                ed.start_play();
            }
        }
        if ed.selection.is_some() {
            if consume(Modifiers::COMMAND, Key::D) {
                ed.duplicate_selection();
            } else if !ed.is_playing() && consume(Modifiers::NONE, Key::Delete) {
                ed.delete_selection();
            }
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let ed = &mut self.editor;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(t("File"), |ui| {
                if ui.button(t("New Scene")).clicked() {
                    ed.new_scene();
                    ui.close();
                }
                ui.menu_button(t("Open Scene"), |ui| {
                    let scenes: Vec<String> = ed.asset_list.iter().filter(|a| a.ends_with(".scene.json")).cloned().collect();
                    if scenes.is_empty() {
                        ui.label(t("No scenes yet"));
                    }
                    for s in scenes {
                        if ui.button(&s).clicked() {
                            ed.open_scene(&s);
                            ui.close();
                        }
                    }
                });
                if ui.add(egui::Button::new(t("Save Scene")).shortcut_text("Ctrl+S")).clicked() {
                    ed.save_scene(false);
                    ui.close();
                }
                if ui.button(t("Save Scene As...")).clicked() {
                    ed.save_scene(true);
                    ui.close();
                }
                if ui.add_enabled(ed.last_view.is_some(), egui::Button::new(t("Export View PNG...")))
                    .on_hover_text(t("Export a clean image from the Scene/Game camera without editor overlays or HUD")).clicked() {
                    ed.export_view_png();
                    ui.close();
                }
                if ui.add(egui::Button::new(t("Settings...")).shortcut_text("Ctrl+Alt+S")).clicked() {
                    ed.settings.open = true;
                    ed.settings.page = settings::Page::Appearance;
                    ui.close();
                }
                if ui.button(t("Project Settings...")).clicked() {
                    ed.settings.open = true;
                    ed.settings.page = settings::Page::General;
                    ui.close();
                }
                ui.separator();
                if ui.button(t("New Project...")).clicked() {
                    ed.new_project = Some(NewProject { name: "My Game".into(), folder: String::new() });
                    ui.close();
                }
                if ui.button(t("Open Project...")).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().set_title(t("Select a folder containing project.json")).pick_folder() {
                        ed.open_project(&dir);
                    }
                    ui.close();
                }
                ui.separator();
                let building = ed.build.is_some();
                if ui.add_enabled(!building, egui::Button::new(t("Build Game"))).on_hover_text(t("Packs assets into game.pak and writes a runnable game to <project>/build")).clicked() {
                    ed.open_build_dialog();
                    ui.close();
                }
                ui.separator();
                if ui.button(t("Exit")).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button(t("Edit"), |ui| {
                if ui.add_enabled(ed.can_undo(), egui::Button::new(t("Undo")).shortcut_text("Ctrl+Z")).clicked() {
                    ed.undo();
                    ui.close();
                }
                if ui.add_enabled(ed.can_redo(), egui::Button::new(t("Redo")).shortcut_text("Ctrl+Y")).clicked() {
                    ed.redo();
                    ui.close();
                }
                ui.separator();
                let sel = ed.selection;
                if ui.add_enabled(sel.is_some(), egui::Button::new(t("Duplicate")).shortcut_text("Ctrl+D")).clicked() {
                    ed.duplicate_entity(sel.unwrap());
                    ui.close();
                }
                if ui.add_enabled(sel.is_some(), egui::Button::new(t("Delete")).shortcut_text("Del")).clicked() {
                    ed.delete_entity(sel.unwrap());
                    ui.close();
                }
            });
            ui.menu_button(t("GameObject"), |ui| ed.create_menu(ui));
            ui.menu_button(t("Scripts"), |ui| {
                if ui.button(t("Build Scripts Now")).clicked() {
                    ed.scripts.start_build(&ed.project, ed.console.sender());
                    ui.close();
                }
                ui.checkbox(&mut ed.scripts.auto_reload, t("Rebuild when files change"));
                if ui.button(t("Install starter library")).on_hover_text(t("Adds ready-made scripts (in folders) and shaders to this project")).clicked() {
                    match library::install(&ed.project.root) {
                        Ok(r) => {
                            ed.console.info(i18n::tf("Library installed: {} scripts, {} shaders", &[&r.scripts.to_string(), &r.shaders.to_string()]));
                            ed.reload_assets();
                            ed.scripts.start_build(&ed.project, ed.console.sender());
                        }
                        Err(e) => ed.console.error(e),
                    }
                    ui.close();
                }
                ui.separator();
                if ui.button(t("Open Scripts in Code Editor")).clicked() {
                    ed.prefs.open(&ed.scripts_file());
                    ui.close();
                }
                if ui.button(t("Open Project in Code Editor")).clicked() {
                    ed.prefs.open(&ed.project.root);
                    ui.close();
                }
            });
            ui.menu_button(t("Window"), |ui| {
                if ui.button("GUI — интерфейс игры").clicked() { ed.gui_open = true; ui.close(); }
                if ui.button("Изображения — слои и кисти").clicked() { ed.image_editor.open = true; ui.close(); }
                if ui.button("Геометрия — редактировать выбранный объект").clicked() { ed.start_mesh_edit(); ui.close(); }
                if ui.button("Анимация — таймлайн").clicked() { ed.animation_open = true; ui.close(); }
                if ui.button("AI — обучение / веса").clicked() { ed.training.open = true; ui.close(); }
                if ui.button(t("Reset Layout")).clicked() {
                    self.dock = default_layout();
                    ui.close();
                }
            });
            ui.menu_button(t("Help"), |ui| {
                if ui.button("Документация (F1)").clicked() { ed.help.open = true; ui.close(); }
                ui.label(t("Scene view: RMB + WASD fly, MMB pan, scroll zoom,"));
                ui.label(t("Alt+LMB orbit, F focus, W/E/R move/rotate/scale."));
                ui.label(t("Ctrl while dragging a handle snaps."));
            });
        });
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let p = theme::pal();
        let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        let bar = egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.available_width(), TITLE_H));

        // The whole bar drags the window; widgets added afterwards sit on top and win the clicks.
        let drag = ui.interact(bar, ui.scope_id().with("titlebar-drag"), egui::Sense::click_and_drag());
        if drag.drag_started_by(egui::PointerButton::Primary) {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if drag.double_clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
        }

        let buttons_w = 3.0 * 46.0;
        let left = egui::Rect::from_min_max(bar.min, egui::pos2(bar.max.x - buttons_w, bar.max.y));
        ui.scope_builder(egui::UiBuilder::new().max_rect(left).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.spacing_mut().button_padding = egui::vec2(8.0, 3.0);
            ui.add_space(10.0);
            ui.add(icons::logo(22.0));
            ui.add_space(8.0);
            self.menu_bar(ui);
        });

        // Project / scene pill in the middle, like the JetBrains project widget.
        let title = format!("{}  \u{203a}  {}{}", self.editor.project.config.name, self.editor.scene_path.as_deref().unwrap_or(t("untitled")), if self.editor.dirty { " *" } else { "" });
        let galley = ui.painter().layout_no_wrap(title, egui::FontId::proportional(13.0), p.text);
        let pill = egui::Rect::from_center_size(bar.center(), egui::vec2(galley.size().x + 28.0, 26.0));
        if pill.left() > bar.left() + 400.0 {
            ui.painter().rect_filled(pill, 13.0, p.hover.gamma_multiply(if theme::is_light() { 0.7 } else { 0.9 }));
            ui.painter().galley(pill.center() - galley.size() / 2.0, galley, p.text);
        }

        let right = egui::Rect::from_min_max(egui::pos2(bar.max.x - buttons_w, bar.min.y), bar.max);
        ui.scope_builder(egui::UiBuilder::new().max_rect(right).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (icon, tip, danger) in [(Icon::Minus, t("Minimize"), false), (if maximized { Icon::Copy } else { Icon::Maximize }, if maximized { t("Restore") } else { t("Maximize") }, false), (Icon::X, t("Close"), true)] {
                let (r, resp) = ui.allocate_exact_size(egui::vec2(46.0, TITLE_H), egui::Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(r, 0.0, if danger { Color32::from_rgb(196, 43, 28) } else { p.hover });
                }
                let tint = if danger && resp.hovered() { Color32::WHITE } else { p.text };
                icon.image(13.0, tint).paint_at(ui, egui::Rect::from_center_size(r.center(), egui::vec2(13.0, 13.0)));
                if resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    ctx.send_viewport_cmd(if icon == Icon::Minus {
                        egui::ViewportCommand::Minimized(true)
                    } else if icon == Icon::X {
                        egui::ViewportCommand::Close
                    } else {
                        egui::ViewportCommand::Maximized(!maximized)
                    });
                }
            }
        });
        ui.allocate_rect(bar, egui::Sense::hover());
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let ed = &mut self.editor;
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let playing = ed.is_playing();
            if playing {
                if run_button(ui, RunShape::Stop, &format!("{} (Ctrl+P)", t("Stop"))).clicked() {
                    ed.stop_play();
                }
                let paused = ed.play.as_ref().is_some_and(|p| p.paused);
                if icons::button(ui, Icon::Pause, t("Pause"), paused).clicked() {
                    if let Some(p) = &mut ed.play {
                        p.paused = !p.paused;
                    }
                }
                if icons::button(ui, Icon::SkipForward, t("Step one frame"), false).clicked() {
                    if let Some(p) = &mut ed.play {
                        p.paused = true;
                        p.step = true;
                    }
                }
                ui.colored_label(theme::green(), t("Playing (changes are discarded on Stop)"));
            } else if run_button(ui, RunShape::Play, &format!("{} (Ctrl+P)", t("Play"))).clicked() {
                ed.start_play();
            }
            separator(ui);
            if ed.scripts.is_building() {
                ui.spinner();
                ui.label(t("Building scripts..."));
            } else if icons::button(ui, Icon::Hammer, t("Build scripts and hot-reload them"), false).clicked() {
                ed.scripts.start_build(&ed.project, ed.console.sender());
            }
            if ed.build.is_some() {
                ui.spinner();
                ui.label(t("Building game..."));
            }
            separator(ui);
            if ed.build.is_some() {
                ui.spinner();
            } else if icons::button(ui, Icon::Package, &format!("{} - {}", t("Build Game"), t("Packs assets into game.pak and writes a runnable game to <project>/build")), false).clicked() {
                ed.open_build_dialog();
            }
            if icons::button(ui, Icon::Save, &format!("{} (Ctrl+S)", t("Save scene")), false).clicked() {
                ed.save_scene(false);
            }
            if icons::button(ui, Icon::Undo2, &format!("{} (Ctrl+Z)", t("Undo")), false).clicked() {
                ed.undo();
            }
            if icons::button(ui, Icon::Redo2, &format!("{} (Ctrl+Y)", t("Redo")), false).clicked() {
                ed.redo();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icons::button(ui, Icon::Settings, &format!("{} (Ctrl+Alt+S)", t("Settings")), false).clicked() {
                    ed.settings.open = true;
                    ed.settings.page = settings::Page::Appearance;
                }
            });
        });
    }

    fn new_project_window(&mut self, ctx: &egui::Context) {
        let Some(np) = &mut self.editor.new_project else { return };
        let mut open = true;
        let mut create = false;
        egui::Window::new(t("New Project")).open(&mut open).collapsible(false).resizable(false).default_pos([260.0, 120.0]).show(ctx, |ui| {
            egui::Grid::new("new-project").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
                ui.label(t("Name"));
                ui.add(egui::TextEdit::singleline(&mut np.name).desired_width(280.0));
                ui.end_row();
                ui.label(t("Parent folder"));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut np.folder).desired_width(200.0));
                    if ui.button(t("Browse...")).clicked() {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            np.folder = dir.to_string_lossy().to_string();
                        }
                    }
                });
                ui.end_row();
            });
            ui.add_space(4.0);
            ui.weak(t("A folder named after the project is created inside the parent folder."));
            ui.add_space(6.0);
            ui.add_enabled_ui(!np.name.trim().is_empty() && !np.folder.trim().is_empty(), |ui| {
                if ui.add(egui::Button::new(RichText::new(t("Create")).color(Color32::WHITE)).fill(theme::ACCENT)).clicked() {
                    create = true;
                }
            });
        });
        if create {
            let (name, dir) = (np.name.trim().to_string(), PathBuf::from(np.folder.trim()).join(np.name.trim()));
            self.editor.new_project = None;
            self.editor.create_project(&dir, &name, "3D");
        } else if !open {
            self.editor.new_project = None;
        }
    }

    /// Remembers the window size and the panel layout for the next start.
    fn remember_window(&mut self, ctx: &egui::Context) {
        let (rect, maximized, fullscreen) = ctx.input(|i| (i.viewport().inner_rect, i.viewport().maximized.unwrap_or(false), i.viewport().fullscreen.unwrap_or(false)));
        self.editor.prefs.maximized = maximized;
        if let Some(r) = rect.filter(|_| !maximized && !fullscreen) {
            // `inner_rect` is in points; they stay stable when the interface scale changes.
            self.editor.prefs.window_size = Some([r.width().round(), r.height().round()]);
        }
    }

    /// Keeps the Nodes tab in step with the node editor: opening a graph adds the tab next to the
    /// Scene (and shows it), closing the graph removes it.
    fn sync_node_tab(&mut self) {
        let open = self.editor.mat.is_open();
        let present = self.dock.find_tab(&Tab::Nodes);
        match (open, present) {
            (true, None) => {
                let path = self.dock.find_tab(&Tab::Scene);
                match path.and_then(|p| self.dock.leaf_mut(p.node_path()).ok()) {
                    Some(leaf) => leaf.append_tab(Tab::Nodes),
                    None => self.dock.push_to_first_leaf(Tab::Nodes),
                }
                self.editor.mat.focus_request = false;
            }
            (true, Some(path)) if self.editor.mat.focus_request => {
                let _ = self.dock.set_active_tab(path);
                self.editor.mat.focus_request = false;
            }
            (false, Some(_)) => self.dock.retain_tabs(|t| *t != Tab::Nodes),
            _ => {}
        }
    }

    fn save_prefs(&mut self) {
        self.editor.sync_prefs();
        let mut dock = self.dock.clone();
        dock.retain_tabs(|t| *t != Tab::Nodes);
        self.editor.prefs.layout = serde_json::to_string(&dock).ok();
        self.editor.prefs.save();
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.editor.update(&ctx);
        self.shortcuts(&ctx);
        self.remember_window(&ctx);
        ctx.request_repaint(); // viewport animates (shaders, play mode)

        let title = format!(
            "{} - {}{} - Kvas Editor",
            self.editor.project.config.name,
            self.editor.scene_path.as_deref().unwrap_or(t("untitled scene")),
            if self.editor.dirty { "*" } else { "" }
        );
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }

        let p = theme::pal();
        egui::Panel::top("titlebar").frame(egui::Frame::NONE).exact_size(TITLE_H).show(ui, |ui| {
            ui_kit::bar_background(ui, egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.available_width(), TITLE_H)));
            self.title_bar(ui)
        });
        egui::Panel::top("toolbar").frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(10, 0))).exact_size(TOOLBAR_H).show(ui, |ui| {
            let r = ui.max_rect();
            ui_kit::bar_background(ui, egui::Rect::from_min_size(r.min - egui::vec2(10.0, 0.0), egui::vec2(r.width() + 20.0, TOOLBAR_H)));
            let building = self.editor.scripts.is_building() || self.editor.build.is_some();
            if building {
                // Indeterminate progress line along the bottom edge of the toolbar.
                let t = ui.input(|i| i.time) as f32;
                let (w, y) = (r.width() + 20.0, r.bottom() - 1.0);
                let head = ((t * 0.7) % 1.4 - 0.2) * w;
                let (a, b) = ((head - 0.25 * w).clamp(0.0, w), head.clamp(0.0, w));
                ui.painter().line_segment([egui::pos2(r.left() - 10.0 + a, y), egui::pos2(r.left() - 10.0 + b, y)], egui::Stroke::new(2.0, theme::ACCENT));
                ui.ctx().request_repaint();
            }
            self.toolbar(ui)
        });
        egui::Panel::bottom("status").frame(egui::Frame::NONE.fill(p.bar).inner_margin(egui::Margin::symmetric(12, 3))).exact_size(24.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new(&self.editor.status).size(12.0).color(p.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let last = self.editor.console.lines.last().map(|(_, m)| m.as_str()).unwrap_or("");
                    ui.label(RichText::new(last).size(12.0).color(p.text_dim));
                });
            });
        });
        self.sync_node_tab();
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(p.gap)).show(ui, |ui| {
            let style = theme::dock_style(ui.style().as_ref());
            DockArea::new(&mut self.dock).style(style).show_inside(ui, &mut Viewer(&mut self.editor));
        });
        self.new_project_window(&ctx);
        self.editor.settings_ui(&ctx);
        self.editor.material_editor_ui(&ctx);
        self.editor.build_dialog_ui(&ctx);
        self.editor.code_editor_ui(&ctx);
        self.editor.animation_window(&ctx);
        self.editor.neural_training_window(&ctx);
        self.editor.mesh_editor_window(&ctx);
        self.editor.image_editor_window(&ctx);
        self.editor.gui_window(&ctx);
        self.editor.help_window(&ctx);
        ui_kit::drag_ghost(&ctx);
    }

    fn on_exit(&mut self) {
        self.save_prefs();
    }
}

/// The project named on the command line (a folder or its project.json), if any.
fn project_arg() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::args().skip(1).find(|a| !a.starts_with('-'))?);
    Some(if path.is_file() { path.parent()?.to_path_buf() } else { path })
}

/// `kvas_editor --build <project>`: builds the game without opening a window (for scripts and CI).
fn build_from_cli(dir: &str) -> ! {
    let project = Project::open(&PathBuf::from(dir)).unwrap_or_else(|e| {
        eprintln!("cannot open project: {e}");
        std::process::exit(1);
    });
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || export::build_game(&project, &tx));
    for (_, line) in rx {
        println!("{line}");
    }
    match worker.join().expect("build thread panicked") {
        Ok(out) => {
            println!("OK: {}", out.display());
            std::process::exit(0)
        }
        Err(e) => {
            eprintln!("build failed: {e}");
            std::process::exit(1)
        }
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--build") {
        build_from_cli(args.get(i + 1).map_or(".", |s| s.as_str()));
    }
    let prefs = prefs::Prefs::load();
    i18n::set(prefs.language);
    let Some(root) = project_arg() else {
        return hub::run(prefs);
    };
    let project = match Project::open(&root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cannot open project: {e}");
            std::process::exit(1);
        }
    };
    let autoplay = args.iter().any(|a| a == "--play");
    let size = prefs.window_size.unwrap_or([1500.0, 900.0]);
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(size)
        .with_min_inner_size([900.0, 600.0])
        .with_title("Kvas Editor")
        .with_decorations(false)
        .with_maximized(prefs.maximized)
        .with_drag_and_drop(true);
    if let Some(icon) = icons::window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Kvas Editor",
        options,
        Box::new(move |cc| {
            theme::init(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx, prefs.theme, prefs.ui_scale);
            icons::install(&cc.egui_ctx);
            let rs = cc.wgpu_render_state.as_ref().expect("wgpu backend required");
            let mut editor = Editor::new(rs, project);
            if let Some(i) = std::env::args().position(|a| a == "--scene") {
                if let Some(path) = std::env::args().nth(i + 1) { editor.open_scene(&path); }
            }
            if autoplay {
                editor.start_play();
            }
            if let Some(i) = std::env::args().position(|a| a == "--material") {
                if let Some(path) = std::env::args().nth(i + 1) {
                    editor.open_material(&path);
                }
            }
            let dock = prefs.layout.as_deref().and_then(|j| serde_json::from_str::<DockState<Tab>>(j).ok()).filter(has_all_tabs).unwrap_or_else(default_layout);
            Ok(Box::new(App { editor, dock, title: String::new() }))
        }),
    )
}

/// A saved layout is only used when every panel is still in it (a layout from another version could miss one).
fn has_all_tabs(dock: &DockState<Tab>) -> bool {
    let found: Vec<Tab> = dock.iter_all_tabs().map(|(_, t)| *t).collect();
    [Tab::Hierarchy, Tab::Scene, Tab::Inspector, Tab::Project, Tab::Console, Tab::Audio].iter().all(|t| found.contains(t))
}
