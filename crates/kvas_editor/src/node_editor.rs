//! Node editor for materials (`.kmat`), object shaders (`.kshader`) and camera effects (`.kfx`).
//! Zoom, pan, box selection, copy / paste, undo / redo, a searchable add menu, comment frames,
//! reroute points, a mini-map, auto layout and a live preview (of the whole result or the selected node).

use crate::editor::Editor;
use crate::i18n::{t, td, tf};
use crate::theme::{self, pal};
use eframe::egui::{self, Color32, Key, Modifiers, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use kvas_engine::material_graph::{all_specs, available, is_output, spec, GraphMode, MatNode, MaterialGraph, NodeSpec};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

const NODE_W: f32 = 196.0;
const HEADER_H: f32 = 24.0;
const ROW_H: f32 = 24.0;
const SOCKET_R: f32 = 5.5;
const PREVIEW_FILE: &str = ".node_preview/preview";

/// The "add node" popup.
#[derive(Clone)]
pub struct AddMenu {
    /// Where the node will appear (graph coordinates).
    at: Vec2,
    /// Where the popup is drawn (screen).
    screen: Pos2,
    search: String,
    /// A link was dropped on empty canvas: connect this node's output to the new node.
    connect_from: Option<u32>,
    just_opened: bool,
}

#[derive(Default)]
pub struct MaterialEditor {
    pub open: bool,
    path: String,
    graph: Option<MaterialGraph>,
    pan: Vec2,
    zoom: f32,
    selected: BTreeSet<u32>,
    /// A link being dragged from the output of this node.
    link_from: Option<u32>,
    dirty_since: Option<Instant>,
    error: String,
    /// Mesh used by the preview: `primitive:sphere` or any model of the project.
    preview_mesh: String,
    /// True when a `.wgsl` shader (not a node graph) is open: code on the left, preview on the right.
    shader_mode: bool,
    code: String,
    /// JSON of the graph at the last stable state, and the history around it.
    snapshot: String,
    undo: Vec<String>,
    redo: Vec<String>,
    last_push: Option<Instant>,
    clipboard: Vec<MatNode>,
    menu: Option<AddMenu>,
    box_start: Option<Pos2>,
    /// Show only the selected node in the preview.
    preview_node: bool,
    preview_json: String,
    editing_comment: Option<u32>,
    fit_next: bool,
}

fn category_color(category: &str) -> Color32 {
    match category {
        "Output" => Color32::from_rgb(0xC2, 0x4D, 0x4D),
        "Input" => Color32::from_rgb(0x3E, 0x8E, 0x5B),
        "Effect" => Color32::from_rgb(0x2F, 0x9C, 0x9C),
        "Utility" => Color32::from_rgb(0x70, 0x76, 0x80),
        "Math" => Color32::from_rgb(0x3D, 0x6E, 0xC9),
        "Vector" => Color32::from_rgb(0x4A, 0x5F, 0xB8),
        "Colour" => Color32::from_rgb(0xB5, 0x7A, 0x2C),
        _ => Color32::from_rgb(0x8B, 0x5C, 0xC4),
    }
}

/// Menu order of the node categories.
const CATEGORIES: &[&str] = &["Input", "Effect", "Utility", "Math", "Vector", "Colour", "Procedural"];

/// A constant default like `vec3<f32>(0.5)` or `vec3<f32>(0.1, 0.2, 0.3)` as numbers, when it is one.
fn parse_default(expr: &str) -> Option<[f32; 3]> {
    let inner = expr.strip_prefix("vec3<f32>(")?.strip_suffix(')')?;
    let nums: Vec<f32> = inner.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    match nums.as_slice() {
        [v] => Some([*v; 3]),
        [a, b, c] => Some([*a, *b, *c]),
        _ => None,
    }
}

fn is_colour_input(name: &str) -> bool {
    matches!(name, "Base color" | "Emission" | "Colour" | "From" | "To" | "Color" | "Tint" | "Low" | "Middle" | "High")
}

fn node_height(spec: &NodeSpec) -> f32 {
    let params = if spec.params.is_empty() { 0 } else if spec.color { 1 } else { spec.params.len() };
    let rows = (spec.inputs.len() + params).max(1);
    HEADER_H + rows as f32 * ROW_H + 8.0
}

/// Size of a node on the canvas in graph units.
fn node_size(node: &MatNode, s: &NodeSpec) -> Vec2 {
    match node.kind.as_str() {
        "Comment" => Vec2::new(node.size[0].max(90.0), node.size[1].max(54.0)),
        "Reroute" => Vec2::new(30.0, 24.0),
        _ => Vec2::new(NODE_W, node_height(s)),
    }
}

/// Which graph kind a file extension opens.
pub fn is_graph_file(path: &str) -> bool {
    path.ends_with(".kmat") || path.ends_with(".kshader") || path.ends_with(".kfx")
}

impl Editor {
    /// Opens a graph (`.kmat`, `.kshader`, `.kfx`) in the node editor, or a `.wgsl` shader as code with a preview.
    pub fn open_material(&mut self, path: &str) {
        let preview_mesh = if self.mat.preview_mesh.is_empty() { "primitive:sphere".to_string() } else { self.mat.preview_mesh.clone() };
        if path.ends_with(".wgsl") {
            let code = self.assets.read_string(path).unwrap_or_default();
            self.mat = MaterialEditor { open: true, path: path.to_string(), shader_mode: true, code, preview_mesh, zoom: 1.0, ..Default::default() };
            return;
        }
        let mode = GraphMode::from_path(path);
        let graph = self.assets.read_string(path).ok().filter(|s| !s.trim().is_empty()).map(|s| MaterialGraph::from_json(&s)).unwrap_or_else(|| Ok(MaterialGraph::default_graph_for(mode)));
        match graph {
            Ok(mut g) => {
                g.mode = mode;
                let snapshot = serde_json::to_string(&g).unwrap_or_default();
                self.mat = MaterialEditor { open: true, path: path.to_string(), graph: Some(g), pan: Vec2::new(20.0, 20.0), zoom: 1.0, preview_mesh, snapshot, fit_next: true, ..Default::default() };
            }
            Err(e) => self.console.error(format!("{path}: {e}")),
        }
    }

    /// Creates a new material (`.kmat`) and opens it.
    pub fn new_material(&mut self) {
        self.new_graph(GraphMode::Material);
    }

    /// Creates a new graph file of the given kind in the current folder and opens it.
    pub fn new_graph(&mut self, mode: GraphMode) {
        let (default_folder, base) = match mode {
            GraphMode::Material => ("materials", "material"),
            GraphMode::Shader => ("shaders", "shader_graph"),
            GraphMode::Effect => ("shaders/fx", "effect_graph"),
        };
        let folder = if self.browser.folder.is_empty() { default_folder.to_string() } else { self.browser.folder.clone() };
        let dir = self.project.assets_dir().join(&folder);
        let _ = std::fs::create_dir_all(&dir);
        let mut n = 1;
        let ext = mode.extension();
        let mut path = dir.join(format!("{base}.{ext}"));
        while path.exists() {
            n += 1;
            path = dir.join(format!("{base}_{n}.{ext}"));
        }
        if let Err(e) = std::fs::write(&path, MaterialGraph::default_graph_for(mode).to_json()) {
            self.console.error(e.to_string());
            return;
        }
        self.reload_assets();
        let rel = path.strip_prefix(self.project.assets_dir()).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        self.open_material(&rel);
    }

    fn save_material(&mut self) {
        if self.mat.shader_mode {
            let full = self.project.assets_dir().join(&self.mat.path);
            match std::fs::write(&full, &self.mat.code) {
                Ok(()) => {
                    self.mat.dirty_since = None;
                    self.renderer.clear_thumbnails();
                }
                Err(e) => self.console.error(e.to_string()),
            }
            return;
        }
        let Some(g) = &self.mat.graph else { return };
        let full = self.project.assets_dir().join(&self.mat.path);
        match std::fs::write(&full, g.to_json()) {
            Ok(()) => {
                self.mat.dirty_since = None;
                self.renderer.clear_thumbnails();
                self.reload_assets();
            }
            Err(e) => self.console.error(e.to_string()),
        }
    }

    /// Records edits for undo and schedules the save.
    fn track_changes(&mut self) {
        let Some(g) = &self.mat.graph else { return };
        let now = serde_json::to_string(g).unwrap_or_default();
        if now == self.mat.snapshot {
            return;
        }
        if !self.mat.snapshot.is_empty() {
            if self.mat.last_push.is_none_or(|t| t.elapsed() > Duration::from_millis(450)) {
                let old = std::mem::take(&mut self.mat.snapshot);
                self.mat.undo.push(old);
                if self.mat.undo.len() > 200 {
                    self.mat.undo.remove(0);
                }
            }
            self.mat.redo.clear();
        }
        self.mat.last_push = Some(Instant::now());
        self.mat.snapshot = now;
        self.mat.error.clear();
        self.mat.dirty_since.get_or_insert_with(Instant::now);
    }

    fn undo_redo(&mut self, undo: bool) {
        let (from, to) = if undo { (&mut self.mat.undo, &mut self.mat.redo) } else { (&mut self.mat.redo, &mut self.mat.undo) };
        let Some(text) = from.pop() else { return };
        to.push(std::mem::take(&mut self.mat.snapshot));
        if let Ok(mut g) = MaterialGraph::from_json(&text) {
            g.mode = GraphMode::from_path(&self.mat.path);
            self.mat.graph = Some(g);
            self.mat.snapshot = text;
            self.mat.selected.clear();
            self.mat.dirty_since = Some(Instant::now());
        }
    }

    pub fn material_editor_ui(&mut self, ctx: &egui::Context) {
        if !self.mat.open || (self.mat.graph.is_none() && !self.mat.shader_mode) {
            return;
        }
        let mut open = true;
        let kind = match self.mat.graph.as_ref().map(|g| g.mode) {
            Some(GraphMode::Shader) => t("Shader graph"),
            Some(GraphMode::Effect) => t("Camera effect graph"),
            Some(GraphMode::Material) => t("Material editor"),
            None => t("Shader editor"),
        };
        let title = format!("{kind}  -  {}", self.mat.path);
        egui::Window::new(title).id(egui::Id::unique("material-editor")).open(&mut open).default_size([1180.0, 720.0]).collapsible(false).show(ctx, |ui| {
            egui::Panel::right("mat-side").exact_size(262.0).resizable(false).frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 10, right: 0, top: 0, bottom: 0 })).show(ui, |ui| self.material_side(ui));
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
                if self.mat.shader_mode {
                    self.shader_code_ui(ui);
                } else {
                    self.material_canvas(ui);
                }
            });
        });
        self.add_menu_ui(ctx);
        self.track_changes();
        if let Some(since) = self.mat.dirty_since {
            if since.elapsed() > Duration::from_millis(350) {
                self.save_material();
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
        if !open {
            if self.mat.dirty_since.is_some() {
                self.save_material();
            }
            self.mat.open = false;
        }
    }

    /// The path the preview should draw: the graph itself, or a temporary graph that shows only the selected node.
    fn preview_path(&mut self) -> String {
        let real = self.mat.path.clone();
        let (Some(g), true) = (&self.mat.graph, self.mat.preview_node) else { return real };
        if g.mode == GraphMode::Effect || self.mat.selected.len() != 1 {
            return real;
        }
        let id = *self.mat.selected.iter().next().unwrap();
        let Some(node) = g.nodes.iter().find(|n| n.id == id) else { return real };
        if is_output(&node.kind) || node.kind == "Comment" {
            return real;
        }
        let mut tmp = g.clone();
        if let Some(out) = tmp.nodes.iter_mut().find(|n| is_output(&n.kind)) {
            // Show the node as glowing colour on an otherwise black, unlit surface.
            let (colour, glow, amount) = match g.mode {
                GraphMode::Material => (0, 3, None),
                _ => (0, 2, Some(3)),
            };
            out.inputs[colour].link = None;
            out.inputs[colour].value = [0.0; 3];
            out.inputs[colour].override_default = true;
            out.inputs[glow].link = Some(id);
            if let Some(a) = amount {
                out.inputs[a].link = None;
                out.inputs[a].value = [0.0; 3];
                out.inputs[a].override_default = true;
            }
        }
        let json = tmp.to_json();
        let rel = format!("{PREVIEW_FILE}.{}", g.mode.extension());
        if json != self.mat.preview_json {
            let full = self.project.assets_dir().join(&rel);
            let _ = std::fs::create_dir_all(full.parent().unwrap());
            if std::fs::write(&full, &json).is_ok() {
                self.mat.preview_json = json;
                self.renderer.clear_thumbnails();
            }
        }
        rel
    }

    fn material_side(&mut self, ui: &mut egui::Ui) {
        let mode = self.mat.graph.as_ref().map(|g| g.mode);
        let path = if self.mat.shader_mode { self.mat.path.clone() } else { self.preview_path() };
        ui.label(RichText::new(t("Preview")).family(theme::bold_family()));
        let models: Vec<String> = self.asset_list.iter().filter(|a| matches!(a.rsplit('.').next().map(str::to_ascii_lowercase).as_deref(), Some("glb" | "gltf" | "obj" | "fbx"))).cloned().collect();
        let shown = self.mat.preview_mesh.trim_start_matches("primitive:").rsplit('/').next().unwrap_or("sphere").to_string();
        egui::ComboBox::from_id_salt("mat-preview-mesh").selected_text(shown).width(200.0).show_ui(ui, |ui| {
            for p in kvas_engine::mesh::PRIMITIVES {
                let key = format!("primitive:{p}");
                if ui.selectable_label(self.mat.preview_mesh == key, *p).clicked() {
                    self.mat.preview_mesh = key;
                }
            }
            ui.separator();
            ui.weak(t("Project meshes"));
            for m in &models {
                if ui.selectable_label(self.mat.preview_mesh == *m, m).clicked() {
                    self.mat.preview_mesh = m.clone();
                }
            }
        });
        let size = ui.available_width().min(240.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
        ui.painter().rect_filled(rect, 10.0, pal().field);
        if mode == Some(GraphMode::Effect) {
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, t("Camera effects show on the scene camera"), egui::FontId::proportional(12.0), pal().text_dim);
        } else {
            let preview_mesh = self.mat.preview_mesh.clone();
            match self.renderer.thumbnail(&preview_mesh, &path, 256) {
                Some(tex) => {
                    ui.painter().image(tex, rect.shrink(4.0), Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                }
                None => {
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, t("Compiling..."), egui::FontId::proportional(12.0), pal().text_dim);
                }
            }
        }
        for e in self.renderer.take_errors() {
            if e.contains(".kmat") || e.contains(".kshader") || e.contains(".kfx") || e.contains(&path) {
                self.mat.error = e;
            }
        }
        if !self.mat.error.is_empty() {
            ui.colored_label(theme::red(), egui::RichText::new(&self.mat.error).size(11.0));
        }
        if self.mat.shader_mode {
            ui.add_space(6.0);
            if ui.button(t("Save")).clicked() {
                self.save_material();
            }
            ui.weak(t("Edit the code on the left; the preview updates when you stop typing."));
            ui.add_space(6.0);
            ui.label(RichText::new(t("Reference")).family(theme::bold_family()));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (name, sig, doc) in crate::wgsl_edit::API {
                    ui.collapsing(RichText::new(*name).monospace(), |ui| {
                        ui.label(RichText::new(*sig).monospace().size(11.0));
                        ui.label(RichText::new(*doc).size(11.5));
                    });
                }
            });
            return;
        }
        let Some(mode) = mode else { return };
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(4.0);
            if mode != GraphMode::Effect {
                ui.checkbox(&mut self.mat.preview_node, t("Preview only the selected node")).on_hover_text(t("Shows what the selected node outputs, as glowing colour"));
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button(t("Export as WGSL shader")).on_hover_text(t("Writes the generated shader next to the graph so you can edit it as code")).clicked() {
                    self.export_material_wgsl();
                }
                if mode == GraphMode::Effect && ui.button(t("Add to scene camera")).on_hover_text(t("Adds this effect to the camera effects of the scene")).clicked() {
                    let path = self.mat.path.clone();
                    self.drop_asset(None, &path);
                }
            });
            if mode != GraphMode::Effect {
                ui.add_space(8.0);
                ui.label(RichText::new(t("Images")).family(theme::bold_family()));
                ui.weak(t("Used by Texture nodes: slot 0 albedo, 1 normal map, 2 ORM."));
                let images: Vec<String> = self.asset_list.iter().filter(|a| matches!(a.rsplit('.').next().map(str::to_ascii_lowercase).as_deref(), Some("png" | "jpg" | "jpeg"))).cloned().collect();
                for (i, label) in ["albedo", "normal", "ORM"].iter().enumerate() {
                    let Some(g) = &mut self.mat.graph else { return };
                    ui.horizontal(|ui| {
                        ui.label(format!("{i}  {label}"));
                        let shown = if g.textures[i].is_empty() { t("None").to_string() } else { g.textures[i].rsplit('/').next().unwrap_or("").to_string() };
                        let combo = egui::ComboBox::from_id_salt(("mat-image", i)).selected_text(shown).width(130.0).show_ui(ui, |ui| {
                            if ui.selectable_label(g.textures[i].is_empty(), t("None")).clicked() {
                                g.textures[i].clear();
                            }
                            for img in &images {
                                if ui.selectable_label(g.textures[i] == *img, img).clicked() {
                                    g.textures[i] = img.clone();
                                }
                            }
                        });
                        // Dropping an image from the Project panel assigns it too.
                        if let Some(drop) = combo.response.dnd_release_payload::<crate::assets_panel::AssetDrag>() {
                            if images.contains(&drop.0) {
                                g.textures[i] = drop.0.clone();
                            }
                        }
                    });
                }
            }
            // Help for the selected node, then the controls.
            ui.add_space(8.0);
            let selected_kind = (self.mat.selected.len() == 1).then(|| self.mat.selected.iter().next().copied()).flatten().and_then(|id| self.mat.graph.as_ref().and_then(|g| g.nodes.iter().find(|n| n.id == id)).map(|n| n.kind.clone()));
            if let Some(kind) = selected_kind {
                ui.label(RichText::new(td(&kind)).family(theme::bold_family()));
                let help = kvas_engine::material_nodes::help(&kind);
                ui.label(RichText::new(if help.is_empty() { t("Connect its output to another node's input.") } else { help }).size(11.5).color(pal().text_dim));
            }
            ui.add_space(6.0);
            ui.label(RichText::new(t("Controls")).family(theme::bold_family()));
            for line in [
                "Right-click: add node / comment / paste",
                "Shift+A: search and add a node",
                "Drag a socket onto empty space: add a connected node",
                "Wheel: zoom, middle button or Space+drag: pan",
                "Drag on empty canvas: box select (Shift adds)",
                "Ctrl+C / V / D: copy, paste, duplicate",
                "Ctrl+Z / Ctrl+Y: undo, redo; F: fit; Delete: remove",
            ] {
                ui.label(RichText::new(t(line)).size(11.0).color(pal().text_dim));
            }
        });
    }

    /// Code view of a `.wgsl` shader with a live preview on the chosen mesh.
    fn shader_code_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.weak(t("Shader code (WGSL). Comments are green; hints for Kvas functions appear below as you type."));
            // A shader exported from a node graph can go back to its nodes.
            let source = self.mat.code.lines().next().and_then(|l| l.strip_prefix("// Generated from ")).and_then(|r| r.split(' ').next()).map(str::to_string);
            if let Some(src) = source.filter(|s| is_graph_file(s)) {
                if ui.button(t("Open node graph")).clicked() {
                    self.open_material(&src);
                }
            }
        });
        if crate::wgsl_edit::code_editor(ui, ui.make_persistent_id("wgsl-code"), &mut self.mat.code) {
            self.mat.dirty_since = Some(Instant::now());
        }
    }

    /// Writes the generated WGSL of the open node material to `shaders/<name>.wgsl`.
    fn export_material_wgsl(&mut self) {
        let Some(g) = &self.mat.graph else { return };
        let body = match kvas_engine::material_graph::to_wgsl(g) {
            Ok(b) => b,
            Err(e) => {
                self.console.error(e);
                return;
            }
        };
        let name = self.mat.path.rsplit('/').next().unwrap_or("material").split('.').next().unwrap_or("material").to_string();
        let rel = format!("shaders/{name}.wgsl");
        let text = format!("// Generated from {} by the node editor. Edit freely; re-export overwrites.\n#include \"shaders/kvas_mesh_api.wgsl\"\n#include \"builtin:lib/color\"\n#include \"builtin:lib/noise\"\n#include \"builtin:lib/math\"\n#include \"builtin:lib/nodes\"\n{body}", self.mat.path);
        let full = self.project.assets_dir().join(&rel);
        let _ = std::fs::create_dir_all(full.parent().unwrap());
        match std::fs::write(&full, text) {
            Ok(()) => {
                self.console.info(tf("Exported {}", &[&rel]));
                self.reload_assets();
            }
            Err(e) => self.console.error(e.to_string()),
        }
    }

    /// Menu entries (kind names) that match a search text, grouped by category order.
    fn menu_entries(mode: GraphMode, search: &str) -> Vec<(&'static str, &'static str)> {
        let q = search.trim().to_lowercase();
        let mut out = Vec::new();
        for cat in CATEGORIES {
            for s in all_specs().filter(|s| s.category == *cat && !is_output(s.kind) && available(s.kind, mode)) {
                let hit = q.is_empty() || s.kind.to_lowercase().contains(&q) || td(s.kind).to_lowercase().contains(&q) || kvas_engine::material_nodes::help(s.kind).to_lowercase().contains(&q);
                if hit {
                    out.push((s.kind, *cat));
                }
            }
        }
        // Names that start with the search first.
        if !q.is_empty() {
            out.sort_by_key(|(k, _)| !(k.to_lowercase().starts_with(&q) || td(k).to_lowercase().starts_with(&q)));
        }
        out
    }

    /// Adds a node at a graph position and optionally connects `from` to its first input.
    fn add_node_at(&mut self, kind: &str, at: Vec2, connect_from: Option<u32>) {
        let Some(g) = &mut self.mat.graph else { return };
        let id = g.add_node(kind, [at.x, at.y]);
        if let (Some(from), Some(s)) = (connect_from, spec(kind)) {
            if !s.inputs.is_empty() {
                g.connect(from, id, 0);
            }
        }
        self.mat.selected = BTreeSet::from([id]);
    }

    /// The searchable popup for adding nodes.
    pub fn add_menu_ui(&mut self, ctx: &egui::Context) {
        let Some(mut menu) = self.mat.menu.clone() else { return };
        let Some(mode) = self.mat.graph.as_ref().map(|g| g.mode) else { return };
        let mut picked: Option<&'static str> = None;
        let mut close = false;
        let area = egui::Area::new(egui::Id::unique("add-node-menu")).order(egui::Order::Foreground).fixed_pos(menu.screen).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(300.0);
                let r = ui.add(egui::TextEdit::singleline(&mut menu.search).hint_text(t("Search nodes...")).desired_width(f32::INFINITY));
                if menu.just_opened {
                    r.request_focus();
                    menu.just_opened = false;
                }
                let entries = Self::menu_entries(mode, &menu.search);
                if ui.input(|i| i.key_pressed(Key::Enter)) {
                    picked = entries.first().map(|e| e.0);
                }
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    close = true;
                }
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    let mut last_cat = "";
                    for (kind, cat) in &entries {
                        if *cat != last_cat && menu.search.trim().is_empty() {
                            ui.add_space(4.0);
                            ui.label(RichText::new(td(cat)).size(11.0).color(category_color(cat)).family(theme::bold_family()));
                            last_cat = cat;
                        }
                        let resp = ui.add(egui::Button::new(RichText::new(td(kind))).frame(false).min_size(Vec2::new(ui.available_width(), 22.0)));
                        let help = kvas_engine::material_nodes::help(kind);
                        let resp = if help.is_empty() { resp } else { resp.on_hover_text(help) };
                        if resp.clicked() {
                            picked = Some(kind);
                        }
                    }
                    if entries.is_empty() {
                        ui.weak(t("No node matches"));
                    }
                });
            });
        });
        if area.response.clicked_elsewhere() && !menu.just_opened {
            close = true;
        }
        if let Some(kind) = picked {
            self.add_node_at(kind, menu.at, menu.connect_from);
            self.mat.menu = None;
        } else if close {
            self.mat.menu = None;
        } else {
            self.mat.menu = Some(menu);
        }
    }

    fn open_add_menu(&mut self, screen: Pos2, at: Vec2, connect_from: Option<u32>) {
        self.mat.menu = Some(AddMenu { at, screen, search: String::new(), connect_from, just_opened: true });
    }

    /// Copies the selected nodes (links between them are kept).
    fn copy_selection(&mut self) {
        let Some(g) = &self.mat.graph else { return };
        let ids = &self.mat.selected;
        self.mat.clipboard = g.nodes.iter().filter(|n| ids.contains(&n.id) && !is_output(&n.kind)).cloned().collect();
    }

    /// Pastes the clipboard with its top-left corner at `at` (graph coordinates).
    fn paste_clipboard(&mut self, at: Vec2) {
        let clip = self.mat.clipboard.clone();
        let Some(g) = &mut self.mat.graph else { return };
        if clip.is_empty() {
            return;
        }
        let min = clip.iter().fold(Vec2::splat(f32::MAX), |m, n| Vec2::new(m.x.min(n.pos[0]), m.y.min(n.pos[1])));
        let mut map = std::collections::HashMap::new();
        for n in &clip {
            map.insert(n.id, g.next_id);
            g.next_id += 1;
        }
        let mut selected = BTreeSet::new();
        for n in &clip {
            let mut copy = n.clone();
            copy.id = map[&n.id];
            copy.pos = [at.x + n.pos[0] - min.x, at.y + n.pos[1] - min.y];
            for s in &mut copy.inputs {
                s.link = s.link.and_then(|l| map.get(&l).copied());
            }
            selected.insert(copy.id);
            g.nodes.push(copy);
        }
        self.mat.selected = selected;
    }

    /// Lays the graph out in columns from the result node to the left.
    fn auto_arrange(&mut self) {
        let Some(g) = &mut self.mat.graph else { return };
        let Some(out) = g.nodes.iter().find(|n| is_output(&n.kind)).map(|n| n.id) else { return };
        let mut depth: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        depth.insert(out, 0);
        let mut stack = vec![out];
        while let Some(id) = stack.pop() {
            let d = depth[&id];
            let links: Vec<u32> = g.nodes.iter().find(|n| n.id == id).map(|n| n.inputs.iter().filter_map(|s| s.link).collect()).unwrap_or_default();
            for l in links {
                if depth.get(&l).is_none_or(|old| *old < d + 1) {
                    depth.insert(l, d + 1);
                    stack.push(l);
                }
            }
        }
        let max_depth = depth.values().copied().max().unwrap_or(0);
        let mut columns: Vec<Vec<usize>> = vec![Vec::new(); max_depth + 2];
        for (i, n) in g.nodes.iter().enumerate() {
            if n.kind == "Comment" {
                continue;
            }
            columns[depth.get(&n.id).copied().unwrap_or(max_depth + 1)].push(i);
        }
        for (d, col) in columns.iter_mut().enumerate() {
            col.sort_by(|a, b| g.nodes[*a].pos[1].total_cmp(&g.nodes[*b].pos[1]));
            let mut y = 0.0;
            for i in col.clone() {
                let size = spec(&g.nodes[i].kind).map_or(Vec2::splat(60.0), |s| node_size(&g.nodes[i], s));
                g.nodes[i].pos = [(max_depth as f32 - d as f32) * (NODE_W + 70.0), y];
                y += size.y + 24.0;
            }
        }
        self.mat.fit_next = true;
    }

    /// Pan and zoom so that every node is visible.
    fn fit_view(&mut self, size: Vec2) {
        let Some(g) = &self.mat.graph else { return };
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for n in &g.nodes {
            let Some(s) = spec(&n.kind) else { continue };
            let sz = node_size(n, s);
            min = Vec2::new(min.x.min(n.pos[0]), min.y.min(n.pos[1]));
            max = Vec2::new(max.x.max(n.pos[0] + sz.x), max.y.max(n.pos[1] + sz.y));
        }
        if min.x > max.x {
            return;
        }
        let content = max - min + Vec2::splat(80.0);
        let z = (size.x / content.x).min(size.y / content.y).clamp(0.25, 1.2);
        self.mat.zoom = z;
        self.mat.pan = (size - (max - min) * z) * 0.5 - min * z;
    }

    fn material_canvas(&mut self, ui: &mut egui::Ui) {
        let (rect, canvas) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        if self.mat.zoom <= 0.0 {
            self.mat.zoom = 1.0;
        }
        if self.mat.fit_next {
            self.mat.fit_next = false;
            self.fit_view(rect.size());
        }
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let hovered = canvas.hovered() || canvas.dragged();

        // Zoom with the wheel around the pointer.
        if hovered && self.mat.menu.is_none() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let old = self.mat.zoom;
                let z = (old * (scroll * 0.0016).exp()).clamp(0.25, 2.0);
                if let Some(p) = pointer {
                    let rel = p - rect.min;
                    self.mat.pan = rel - (rel - self.mat.pan) * (z / old);
                }
                self.mat.zoom = z;
            }
        }
        let z = self.mat.zoom;
        // Pan: middle button, Alt + drag, or Space + drag.
        let pan_keys = ui.input(|i| i.modifiers.alt || i.key_down(Key::Space));
        if canvas.dragged_by(egui::PointerButton::Middle) || (canvas.dragged_by(egui::PointerButton::Primary) && pan_keys) {
            self.mat.pan += ui.input(|i| i.pointer.delta());
        }
        let origin = rect.min + self.mat.pan;
        let to_screen = |p: Vec2| origin + p * z;
        let to_graph = |p: Pos2| (p - origin) / z;

        // Background grid.
        painter.rect_filled(rect, 8.0, pal().field);
        let step = 28.0 * z.max(0.5);
        let (ox, oy) = (self.mat.pan.x.rem_euclid(step), self.mat.pan.y.rem_euclid(step));
        let mut y = rect.top() + oy;
        while y < rect.bottom() {
            let mut x = rect.left() + ox;
            while x < rect.right() {
                painter.circle_filled(Pos2::new(x, y), 1.0, pal().border_strong.gamma_multiply(0.7));
                x += step;
            }
            y += step;
        }

        let Some(graph) = self.mat.graph.clone() else { return };
        let mode = graph.mode;
        let geom = |n: &MatNode| spec(&n.kind).map(|s| Rect::from_min_size(to_screen(Vec2::from(n.pos)), node_size(n, s) * z));
        let input_pos = |n: &MatNode, r: Rect, i: usize| {
            if n.kind == "Reroute" {
                Pos2::new(r.left(), r.center().y)
            } else {
                Pos2::new(r.left(), r.top() + (HEADER_H + i as f32 * ROW_H + ROW_H * 0.5) * z)
            }
        };
        let output_pos = |n: &MatNode, r: Rect| {
            if n.kind == "Reroute" {
                Pos2::new(r.right(), r.center().y)
            } else {
                Pos2::new(r.right(), r.top() + (HEADER_H + ROW_H * 0.5) * z)
            }
        };
        let shift = ui.input(|i| i.modifiers.shift);

        let mut delete: Vec<u32> = Vec::new();
        let mut new_link: Option<(u32, u32, usize)> = None;
        let mut unlink: Option<(u32, usize)> = None;
        let mut start_link: Option<u32> = None;
        let mut drag_move: Option<(u32, Vec2)> = None;
        let mut edits: Vec<(u32, usize, [f32; 3])> = Vec::new();
        let mut param_edits: Vec<(u32, usize, f32)> = Vec::new();
        let mut clicked_node: Option<u32> = None;
        let mut hovered_node: Option<u32> = None;
        let mut dup = false;
        let mut preview_this: Option<u32> = None;
        let mut resize: Option<(u32, Vec2)> = None;
        let mut comment_text: Vec<(u32, String)> = Vec::new();
        let mut start_editing: Option<u32> = None;
        let mut disconnect: Option<u32> = None;

        // Comment frames sit behind everything.
        for node in graph.nodes.iter().filter(|n| n.kind == "Comment") {
            let Some(r) = geom(node) else { continue };
            if !r.intersects(rect) {
                continue;
            }
            let selected = self.mat.selected.contains(&node.id);
            painter.rect_filled(r, 10.0 * z, theme::ACCENT.gamma_multiply(0.10));
            painter.rect_stroke(r, 10.0 * z, Stroke::new(if selected { 2.0 } else { 1.0 }, theme::ACCENT.gamma_multiply(if selected { 1.0 } else { 0.5 })), egui::StrokeKind::Inside);
            let header = Rect::from_min_size(r.min, Vec2::new(r.width(), 26.0 * z));
            painter.rect_filled(header, egui::CornerRadius::same((8.0 * z) as u8), theme::ACCENT.gamma_multiply(0.22));
            let drag = ui.interact(header, ui.scope_id().with(("comment", node.id)), Sense::click_and_drag());
            if drag.clicked() || drag.drag_started() {
                clicked_node = Some(node.id);
            }
            if drag.double_clicked() {
                start_editing = Some(node.id);
            }
            if drag.dragged() {
                drag_move = Some((node.id, ui.input(|i| i.pointer.delta()) / z));
            }
            if self.mat.editing_comment == Some(node.id) {
                let mut text = node.text.clone();
                ui.scope_builder(egui::UiBuilder::new().max_rect(header.shrink2(Vec2::new(6.0, 2.0))), |ui| {
                    let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(f32::INFINITY));
                    r.request_focus();
                    if r.changed() {
                        comment_text.push((node.id, text.clone()));
                    }
                    if r.lost_focus() {
                        start_editing = Some(u32::MAX);
                    }
                });
            } else {
                painter.text(header.left_center() + Vec2::new(10.0 * z, 0.0), egui::Align2::LEFT_CENTER, &node.text, egui::FontId::new((13.0 * z).max(6.0), theme::bold_family()), pal().text);
            }
            let handle = Rect::from_min_size(r.max - Vec2::splat(16.0 * z), Vec2::splat(16.0 * z));
            painter.text(handle.center(), egui::Align2::CENTER_CENTER, "◢", egui::FontId::proportional(11.0 * z), theme::ACCENT);
            let rz = ui.interact(handle, ui.scope_id().with(("comment-resize", node.id)), Sense::drag());
            if rz.dragged() {
                resize = Some((node.id, ui.input(|i| i.pointer.delta()) / z));
            }
            if rz.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
            }
            drag.context_menu(|ui| {
                if ui.button(t("Delete")).clicked() {
                    delete.push(node.id);
                    ui.close();
                }
            });
        }

        // Links under the nodes.
        for node in graph.nodes.iter().filter(|n| n.kind != "Comment") {
            let Some(r) = geom(node) else { continue };
            for (i, socket) in node.inputs.iter().enumerate() {
                let Some(from) = socket.link.and_then(|id| graph.nodes.iter().find(|n| n.id == id)) else { continue };
                let Some(fr) = geom(from) else { continue };
                let (a, b) = (output_pos(from, fr), input_pos(node, r, i));
                let highlight = self.mat.selected.contains(&node.id) || self.mat.selected.contains(&from.id);
                let k = ((b.x - a.x).abs() * 0.5).max(40.0 * z);
                let colour = category_color(spec(&from.kind).map_or("", |s| s.category));
                let stroke = Stroke::new(if highlight { 3.0 } else { 2.0 } * z.max(0.6), if highlight { theme::ACCENT } else { colour.gamma_multiply(1.1) });
                painter.add(egui::epaint::CubicBezierShape::from_points_stroke([a, a + Vec2::new(k, 0.0), b - Vec2::new(k, 0.0), b], false, Color32::TRANSPARENT, stroke));
            }
        }

        let widgets = z >= 0.6;
        for node in graph.nodes.iter().filter(|n| n.kind != "Comment") {
            let Some(s) = spec(&node.kind) else { continue };
            let Some(r) = geom(node) else { continue };
            if !r.expand(40.0).intersects(rect) {
                continue;
            }
            let selected = self.mat.selected.contains(&node.id);
            let is_reroute = node.kind == "Reroute";
            let out = is_output(&node.kind);
            let radius = 8.0 * z;
            let bold = egui::FontId::new((12.5 * z).max(6.0), theme::bold_family());
            let small = egui::FontId::proportional((12.0 * z).max(6.0));

            // The whole body picks and drags the node; sockets and fields are registered after and win.
            let body = ui.interact(r, ui.scope_id().with(("node", node.id)), Sense::click_and_drag());
            if body.hovered() {
                hovered_node = Some(node.id);
            }
            if body.clicked() || body.drag_started() {
                clicked_node = Some(node.id);
            }
            if body.dragged() && self.mat.link_from.is_none() {
                drag_move = Some((node.id, ui.input(|i| i.pointer.delta()) / z));
            }
            body.context_menu(|ui| {
                if !out && ui.button(t("Delete")).clicked() {
                    delete.push(node.id);
                    ui.close();
                }
                if !out && ui.button(t("Duplicate")).clicked() {
                    clicked_node = Some(node.id);
                    dup = true;
                    ui.close();
                }
                if ui.button(t("Preview this node")).clicked() {
                    preview_this = Some(node.id);
                    ui.close();
                }
                if node.inputs.iter().any(|s| s.link.is_some()) && ui.button(t("Disconnect inputs")).clicked() {
                    disconnect = Some(node.id);
                    ui.close();
                }
            });

            if is_reroute {
                painter.rect_filled(r, radius, pal().card);
                painter.rect_stroke(r, radius, Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { theme::ACCENT } else { pal().border_strong }), egui::StrokeKind::Inside);
            } else {
                painter.rect_filled(r.translate(Vec2::new(0.0, 3.0 * z)), radius, Color32::from_black_alpha(60));
                painter.rect_filled(r, radius, pal().card);
                painter.rect_stroke(r, radius, Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { theme::ACCENT } else { pal().border_strong }), egui::StrokeKind::Inside);
                let header = Rect::from_min_size(r.min, Vec2::new(r.width(), HEADER_H * z));
                painter.rect_filled(header, egui::CornerRadius { nw: radius as u8, ne: radius as u8, sw: 0, se: 0 }, category_color(s.category).gamma_multiply(0.85));
                painter.text(header.left_center() + Vec2::new(10.0 * z, 0.0), egui::Align2::LEFT_CENTER, td(&node.kind), bold, Color32::WHITE);
                let help = kvas_engine::material_nodes::help(&node.kind);
                if !help.is_empty() && pointer.is_some_and(|p| header.contains(p)) {
                    body.clone().on_hover_text(help);
                }
            }

            // Output socket.
            if !out {
                let p = output_pos(node, r);
                let resp = ui.interact(Rect::from_center_size(p, Vec2::splat(18.0 * z.max(0.7))), ui.scope_id().with(("out", node.id)), Sense::click_and_drag());
                let hot = resp.hovered() || self.mat.link_from == Some(node.id);
                painter.circle_filled(p, SOCKET_R * z.max(0.7), if hot { theme::ACCENT } else { category_color(s.category) });
                painter.circle_stroke(p, SOCKET_R * z.max(0.7), Stroke::new(1.5, pal().card));
                if resp.drag_started() {
                    start_link = Some(node.id);
                }
            }

            let mut row = 0usize;
            for (i, (name, default)) in s.inputs.iter().enumerate() {
                let y = r.top() + (HEADER_H + row as f32 * ROW_H) * z;
                let p = input_pos(node, r, i);
                let resp = ui.interact(Rect::from_center_size(p, Vec2::splat(18.0 * z.max(0.7))), ui.scope_id().with(("in", node.id, i)), Sense::click_and_drag());
                let linked = node.inputs[i].link.is_some();
                let near = self.mat.link_from.is_some() && pointer.is_some_and(|m| (m - p).length() < 14.0 * z.max(0.8));
                painter.circle_filled(p, SOCKET_R * z.max(0.7), if near || resp.hovered() { theme::ACCENT } else if linked { pal().text } else { pal().border_strong });
                painter.circle_stroke(p, SOCKET_R * z.max(0.7), Stroke::new(1.5, pal().card));
                if resp.clicked() || resp.drag_started() {
                    if let Some(from) = node.inputs[i].link {
                        unlink = Some((node.id, i));
                        start_link = Some(from);
                    }
                }
                if near && ui.input(|i| i.pointer.any_released()) {
                    if let Some(from) = self.mat.link_from {
                        new_link = Some((from, node.id, i));
                    }
                }
                if !is_reroute {
                    painter.text(Pos2::new(r.left() + 14.0 * z, y + ROW_H * z * 0.5), egui::Align2::LEFT_CENTER, td(name), small.clone(), pal().text);
                    if !linked {
                        let field = Rect::from_min_size(Pos2::new(r.left() + 96.0 * z, y + 2.0 * z), Vec2::new((NODE_W - 106.0) * z, (ROW_H - 4.0) * z));
                        let current = if node.inputs[i].override_default || node.inputs[i].value != [0.0; 3] { Some(node.inputs[i].value) } else { parse_default(default) };
                        match current {
                            Some(mut value) if widgets => {
                                let mut changed = false;
                                ui.scope_builder(egui::UiBuilder::new().max_rect(field), |ui| {
                                    ui.spacing_mut().interact_size.y = (ROW_H - 4.0) * z;
                                    ui.style_mut().override_font_id = Some(small.clone());
                                    if is_colour_input(name) {
                                        changed = ui.color_edit_button_rgb(&mut value).changed();
                                    } else {
                                        let mut v = value[0];
                                        if ui.add(egui::DragValue::new(&mut v).speed(0.01).max_decimals(3)).changed() {
                                            value = [v; 3];
                                            changed = true;
                                        }
                                    }
                                });
                                if changed {
                                    edits.push((node.id, i, value));
                                }
                            }
                            Some(_) => {}
                            None if widgets => {
                                painter.text(Pos2::new(field.left(), y + ROW_H * z * 0.5), egui::Align2::LEFT_CENTER, t("(default)"), egui::FontId::proportional(11.0 * z), pal().text_dim);
                            }
                            None => {}
                        }
                    }
                }
                row += 1;
            }
            // Parameters (one colour picker, or one row each).
            let first_param_row = row;
            if s.color && node.params.len() >= 3 {
                let y = r.top() + (HEADER_H + first_param_row as f32 * ROW_H) * z;
                let field = Rect::from_min_size(Pos2::new(r.left() + 14.0 * z, y + 2.0 * z), Vec2::new((NODE_W - 28.0) * z, (ROW_H - 4.0) * z));
                let mut rgb = [node.params[0], node.params[1], node.params[2]];
                if widgets {
                    ui.scope_builder(egui::UiBuilder::new().max_rect(field), |ui| {
                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                            for (k, v) in rgb.iter().enumerate() {
                                param_edits.push((node.id, k, *v));
                            }
                        }
                    });
                } else {
                    painter.rect_filled(field, 4.0, Color32::from_rgb((rgb[0].clamp(0.0, 1.0) * 255.0) as u8, (rgb[1].clamp(0.0, 1.0) * 255.0) as u8, (rgb[2].clamp(0.0, 1.0) * 255.0) as u8));
                }
            } else {
                for (k, (pname, _, min, max)) in s.params.iter().enumerate() {
                    let y = r.top() + (HEADER_H + (first_param_row + k) as f32 * ROW_H) * z;
                    let label = pname.split(" (").next().unwrap_or(pname);
                    painter.text(Pos2::new(r.left() + 14.0 * z, y + ROW_H * z * 0.5), egui::Align2::LEFT_CENTER, td(label), small.clone(), pal().text_dim);
                    let field = Rect::from_min_size(Pos2::new(r.left() + 96.0 * z, y + 2.0 * z), Vec2::new((NODE_W - 106.0) * z, (ROW_H - 4.0) * z));
                    let mut v = node.params.get(k).copied().unwrap_or(0.0);
                    if widgets {
                        ui.scope_builder(egui::UiBuilder::new().max_rect(field), |ui| {
                            ui.spacing_mut().interact_size.y = (ROW_H - 4.0) * z;
                            ui.style_mut().override_font_id = Some(small.clone());
                            if ui.add(egui::DragValue::new(&mut v).speed(((max - min) * 0.005).max(0.001)).range(*min..=*max).max_decimals(3)).changed() {
                                param_edits.push((node.id, k, v));
                            }
                        });
                    }
                }
            }
        }

        // Link being dragged.
        if let (Some(from), Some(m)) = (self.mat.link_from, pointer) {
            if let Some((n, fr)) = graph.nodes.iter().find(|n| n.id == from).and_then(|n| geom(n).map(|r| (n, r))) {
                let a = output_pos(n, fr);
                let k = ((m.x - a.x).abs() * 0.5).max(40.0 * z);
                painter.add(egui::epaint::CubicBezierShape::from_points_stroke([a, a + Vec2::new(k, 0.0), m - Vec2::new(k, 0.0), m], false, Color32::TRANSPARENT, Stroke::new(2.0, theme::ACCENT)));
            }
        }

        // Selection box.
        if canvas.drag_started_by(egui::PointerButton::Primary) && !pan_keys && pointer.is_some() && hovered_node.is_none() && self.mat.link_from.is_none() {
            self.mat.box_start = pointer;
        }
        if let (Some(a), Some(b)) = (self.mat.box_start, pointer) {
            let sel = Rect::from_two_pos(a, b);
            painter.rect_filled(sel, 2.0, theme::ACCENT.gamma_multiply(0.12));
            painter.rect_stroke(sel, 2.0, Stroke::new(1.0, theme::ACCENT), egui::StrokeKind::Inside);
            let mut picked: BTreeSet<u32> = if shift { self.mat.selected.clone() } else { BTreeSet::new() };
            for n in &graph.nodes {
                if let Some(nr) = geom(n) {
                    if sel.intersects(nr) {
                        picked.insert(n.id);
                    }
                }
            }
            self.mat.selected = picked;
        }
        if ui.input(|i| i.pointer.any_released()) {
            self.mat.box_start = None;
        }
        if canvas.clicked() && hovered_node.is_none() {
            if !shift {
                self.mat.selected.clear();
            }
            self.mat.editing_comment = None;
        }

        // Keyboard.
        let typing = ui.ctx().memory(|m| m.focused()).is_some() && self.mat.editing_comment.is_some();
        let (copy, paste, cut_dup, select_all, undo, redo, del, fit, add_key) = if hovered && !typing && self.mat.menu.is_none() {
            ui.input_mut(|i| {
                let copy = i.consume_key(Modifiers::COMMAND, Key::C) || i.events.iter().any(|e| matches!(e, egui::Event::Copy));
                let paste = i.consume_key(Modifiers::COMMAND, Key::V) || i.events.iter().any(|e| matches!(e, egui::Event::Paste(_)));
                (
                    copy,
                    paste,
                    i.consume_key(Modifiers::COMMAND, Key::D),
                    i.consume_key(Modifiers::COMMAND, Key::A),
                    i.consume_key(Modifiers::COMMAND, Key::Z),
                    i.consume_key(Modifiers::COMMAND, Key::Y) || i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z),
                    i.consume_key(Modifiers::NONE, Key::Delete) || i.consume_key(Modifiers::NONE, Key::Backspace),
                    i.consume_key(Modifiers::NONE, Key::F) || i.consume_key(Modifiers::NONE, Key::Home),
                    i.consume_key(Modifiers::SHIFT, Key::A),
                )
            })
        } else {
            (false, false, false, false, false, false, false, false, false)
        };

        // Selection from clicks.
        if let Some(id) = clicked_node {
            if shift {
                if !self.mat.selected.insert(id) {
                    self.mat.selected.remove(&id);
                }
            } else if !self.mat.selected.contains(&id) {
                self.mat.selected = BTreeSet::from([id]);
            }
        }
        if let Some(id) = start_editing {
            self.mat.editing_comment = (id != u32::MAX).then_some(id);
        }

        // Canvas context menu -> add node popup.
        let menu_screen = ui.input(|i| i.pointer.interact_pos()).unwrap_or(rect.center());
        let mut open_menu = false;
        canvas.context_menu(|ui| {
            if ui.button(t("Add node...")).clicked() {
                open_menu = true;
                ui.close();
            }
            if ui.button(t("Add comment frame")).clicked() {
                self.mat.menu = None;
                if let Some(g) = &mut self.mat.graph {
                    let at = to_graph(menu_screen);
                    let id = g.add_node("Comment", [at.x, at.y]);
                    self.mat.selected = BTreeSet::from([id]);
                }
                ui.close();
            }
            if ui.button(t("Paste")).clicked() {
                self.paste_clipboard(to_graph(menu_screen));
                ui.close();
            }
            ui.separator();
            if ui.button(t("Arrange nodes")).clicked() {
                self.auto_arrange();
                ui.close();
            }
            if ui.button(t("Fit to window")).clicked() {
                self.mat.fit_next = true;
                ui.close();
            }
        });
        if open_menu || add_key {
            let at = to_graph(if add_key { pointer.unwrap_or(menu_screen) } else { menu_screen });
            self.open_add_menu(if add_key { pointer.unwrap_or(menu_screen) } else { menu_screen }, at, None);
        }

        // Finish a link: onto a socket, onto a node body, or into the add popup.
        let released = ui.input(|i| i.pointer.any_released());
        if start_link.is_some() {
            self.mat.link_from = start_link;
        }
        if released {
            if let Some(from) = self.mat.link_from.take() {
                if new_link.is_none() && start_link.is_none() {
                    if let Some(p) = pointer.filter(|p| rect.contains(*p)) {
                        let over = graph.nodes.iter().find(|n| n.id != from && n.kind != "Comment" && geom(n).is_some_and(|r| r.contains(p)));
                        match over {
                            Some(n) => {
                                let free = n.inputs.iter().position(|s| s.link.is_none()).unwrap_or(0);
                                if !n.inputs.is_empty() {
                                    new_link = Some((from, n.id, free));
                                }
                            }
                            None => {
                                self.open_add_menu(p, to_graph(p), Some(from));
                            }
                        }
                    }
                }
            }
        }

        // Apply what happened.
        let ids: Vec<u32> = self.mat.selected.iter().copied().collect();
        let mut selected_delete = Vec::new();
        if del {
            selected_delete = ids.clone();
        }
        if select_all {
            self.mat.selected = graph.nodes.iter().map(|n| n.id).collect();
        }
        if copy {
            self.copy_selection();
        }
        if paste {
            self.paste_clipboard(to_graph(pointer.unwrap_or(rect.center())));
        }
        if cut_dup || dup {
            self.copy_selection();
            let at = ids.iter().filter_map(|id| graph.nodes.iter().find(|n| n.id == *id)).fold(Vec2::splat(f32::MAX), |m, n| Vec2::new(m.x.min(n.pos[0]), m.y.min(n.pos[1]))) + Vec2::splat(30.0);
            if at.x < f32::MAX {
                self.paste_clipboard(at);
            }
        }
        if undo {
            self.undo_redo(true);
        }
        if redo {
            self.undo_redo(false);
        }
        if fit {
            self.mat.fit_next = true;
        }
        if let Some(id) = preview_this {
            self.mat.selected = BTreeSet::from([id]);
            self.mat.preview_node = true;
        }
        let selected_now = self.mat.selected.clone();
        if let Some(g) = &mut self.mat.graph {
            if let Some((node, input)) = unlink {
                if let Some(n) = g.nodes.iter_mut().find(|n| n.id == node) {
                    n.inputs[input].link = None;
                }
            }
            if let Some(id) = disconnect {
                if let Some(n) = g.nodes.iter_mut().find(|n| n.id == id) {
                    for s in &mut n.inputs {
                        s.link = None;
                    }
                }
            }
            if let Some((from, to, input)) = new_link {
                g.connect(from, to, input);
            }
            if let Some((id, d)) = drag_move {
                // Move the whole selection; a comment also carries the nodes inside it.
                let mut moving: BTreeSet<u32> = if selected_now.contains(&id) { selected_now.clone() } else { BTreeSet::from([id]) };
                let frames: Vec<Rect> = g.nodes.iter().filter(|n| n.kind == "Comment" && moving.contains(&n.id)).map(|n| Rect::from_min_size(Pos2::new(n.pos[0], n.pos[1]), Vec2::from(n.size))).collect();
                for n in &g.nodes {
                    if n.kind == "Comment" {
                        continue;
                    }
                    let Some(s) = spec(&n.kind) else { continue };
                    let c = Pos2::new(n.pos[0], n.pos[1]) + node_size(n, s) * 0.5;
                    if frames.iter().any(|f| f.contains(c)) {
                        moving.insert(n.id);
                    }
                }
                for n in g.nodes.iter_mut().filter(|n| moving.contains(&n.id)) {
                    n.pos[0] += d.x;
                    n.pos[1] += d.y;
                }
            }
            if let Some((id, d)) = resize {
                if let Some(n) = g.nodes.iter_mut().find(|n| n.id == id) {
                    n.size = [(n.size[0] + d.x).max(90.0), (n.size[1] + d.y).max(54.0)];
                }
            }
            for (id, text) in comment_text {
                if let Some(n) = g.nodes.iter_mut().find(|n| n.id == id) {
                    n.text = text;
                }
            }
            for (id, i, v) in edits {
                if let Some(n) = g.nodes.iter_mut().find(|n| n.id == id) {
                    n.inputs[i].value = v;
                    n.inputs[i].override_default = true;
                }
            }
            for (id, k, v) in param_edits {
                if let Some(n) = g.nodes.iter_mut().find(|n| n.id == id) {
                    n.params[k] = v;
                }
            }
            for id in delete.into_iter().chain(selected_delete) {
                g.remove_node(id);
                self.mat.selected.remove(&id);
            }
        }

        // Overlays: toolbar and mini-map.
        self.canvas_toolbar(ui, rect, mode);
        self.minimap(ui, rect, &graph);
    }

    fn canvas_toolbar(&mut self, ui: &mut egui::Ui, rect: Rect, mode: GraphMode) {
        let bar = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::new(rect.width() - 16.0, 28.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(bar), |ui| {
            ui.horizontal(|ui| {
                if ui.button(t("+ Add node")).on_hover_text("Shift+A").clicked() {
                    let c = rect.center();
                    let at = (c - (rect.min + self.mat.pan)) / self.mat.zoom;
                    self.open_add_menu(c, at, None);
                }
                if ui.add_enabled(!self.mat.undo.is_empty(), egui::Button::new("↶")).on_hover_text("Ctrl+Z").clicked() {
                    self.undo_redo(true);
                }
                if ui.add_enabled(!self.mat.redo.is_empty(), egui::Button::new("↷")).on_hover_text("Ctrl+Y").clicked() {
                    self.undo_redo(false);
                }
                if ui.button(t("Fit")).on_hover_text("F").clicked() {
                    self.mat.fit_next = true;
                }
                if ui.button(t("Arrange")).on_hover_text(t("Tidy the nodes into columns")).clicked() {
                    self.auto_arrange();
                }
                ui.weak(format!("{:.0}%", self.mat.zoom * 100.0));
                ui.weak(match mode {
                    GraphMode::Material => t("Material"),
                    GraphMode::Shader => t("Shader"),
                    GraphMode::Effect => t("Camera effect"),
                });
            });
        });
    }

    fn minimap(&mut self, ui: &mut egui::Ui, rect: Rect, graph: &MaterialGraph) {
        let size = Vec2::new(170.0, 110.0);
        let map = Rect::from_min_size(rect.max - size - Vec2::splat(10.0), size);
        let painter = ui.painter_at(rect);
        painter.rect_filled(map, 6.0, pal().panel.gamma_multiply(0.92));
        painter.rect_stroke(map, 6.0, Stroke::new(1.0, pal().border_strong), egui::StrokeKind::Inside);
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for n in &graph.nodes {
            if let Some(s) = spec(&n.kind) {
                let sz = node_size(n, s);
                min = Vec2::new(min.x.min(n.pos[0]), min.y.min(n.pos[1]));
                max = Vec2::new(max.x.max(n.pos[0] + sz.x), max.y.max(n.pos[1] + sz.y));
            }
        }
        if min.x > max.x {
            return;
        }
        let view_min = (-self.mat.pan) / self.mat.zoom;
        let view_max = view_min + rect.size() / self.mat.zoom;
        min = Vec2::new(min.x.min(view_min.x), min.y.min(view_min.y));
        max = Vec2::new(max.x.max(view_max.x), max.y.max(view_max.y));
        let span = (max - min).max(Vec2::splat(1.0));
        let scale = ((size.x - 10.0) / span.x).min((size.y - 10.0) / span.y);
        let at = |p: Vec2| map.min + Vec2::splat(5.0) + (p - min) * scale;
        for n in &graph.nodes {
            if let Some(s) = spec(&n.kind) {
                let r = Rect::from_min_size(at(Vec2::from(n.pos)), node_size(n, s) * scale);
                let col = if n.kind == "Comment" { theme::ACCENT.gamma_multiply(0.2) } else { category_color(s.category) };
                painter.rect_filled(r, 1.5, col);
            }
        }
        let view = Rect::from_min_max(at(view_min), at(view_max)).intersect(map);
        painter.rect_stroke(view, 2.0, Stroke::new(1.5, theme::ACCENT), egui::StrokeKind::Inside);
        let resp = ui.interact(map, ui.scope_id().with("minimap"), Sense::click_and_drag());
        if resp.dragged() || resp.clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                let graph_pos = min + (p - map.min - Vec2::splat(5.0)) / scale;
                self.mat.pan = rect.size() * 0.5 - graph_pos * self.mat.zoom;
            }
        }
    }
}
