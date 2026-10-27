//! Project panel: folder tree + thumbnail grid. Model and shader thumbnails are rendered by the
//! GPU renderer (a lit 3D preview), textures show the image.

use crate::editor::Editor;
use crate::i18n::{t, tf};
use crate::icons::{self, Icon};
use eframe::egui::{self, Color32, Sense};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Drag payload: an asset path (relative to `assets/`) being dragged.
pub struct AssetDrag(pub String);

const SHADER_TEMPLATE: &str = "// The engine prelude (globals, object, lighting helpers) is prepended automatically.
// Required entry points: vs_main and fs_main. Edit and save: the editor reloads it live.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tex = textureSample(albedo_tex, albedo_smp, in.uv * object.params.z);
    let base = object.color * tex;
    let lit = lighting(in.world_pos, normalize(in.normal), base.rgb, object.params.x);
    return vec4<f32>(to_display(lit), base.a);
}
";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Folder,
    Mesh,
    Texture,
    Shader,
    Audio,
    Script,
    Material,
    Scene,
    Other,
}

fn kind_of(name: &str, is_dir: bool) -> Kind {
    if is_dir {
        return Kind::Folder;
    }
    if name.ends_with(".scene.json") {
        return Kind::Scene;
    }
    match name.rsplit('.').next().map(|e| e.to_ascii_lowercase()).as_deref() {
        Some("obj" | "gltf" | "glb" | "fbx") => Kind::Mesh,
        Some("png" | "jpg" | "jpeg") => Kind::Texture,
        Some("wgsl") => Kind::Shader,
        Some("kmat" | "kshader" | "kfx") => Kind::Material,
        Some("wav" | "ogg" | "mp3" | "flac" | "m4a") => Kind::Audio,
        _ => Kind::Other,
    }
}

/// Accent colour per file kind (icons, extension badges).
fn kind_color(kind: Kind) -> Color32 {
    match kind {
        Kind::Folder => Color32::from_rgb(230, 190, 90),
        Kind::Mesh => Color32::from_rgb(90, 150, 230),
        Kind::Texture => Color32::from_rgb(110, 190, 120),
        Kind::Shader => Color32::from_rgb(170, 120, 230),
        Kind::Audio => Color32::from_rgb(230, 130, 170),
        Kind::Script => Color32::from_rgb(235, 120, 60),
        Kind::Material => Color32::from_rgb(220, 90, 160),
        Kind::Scene => Color32::from_rgb(90, 190, 200),
        Kind::Other => Color32::from_gray(140),
    }
}

fn folder_for(path: &Path) -> &'static str {
    match kind_of(&path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), false) {
        Kind::Mesh => "models",
        Kind::Texture => "textures",
        Kind::Shader => "shaders",
        Kind::Audio => "audio",
        Kind::Script => "",
        Kind::Material => "materials",
        Kind::Scene => "scenes",
        _ => "",
    }
}

struct Entry {
    name: String,
    is_dir: bool,
}

/// Dragging a script (by name) from the browser onto an object attaches it.
pub struct ScriptDrag(pub String);

pub struct BrowserState {
    /// Name being typed for a new script.
    pub new_script: Option<String>,
    /// Template index and "own file" choice of the New Script dialog.
    pub script_template: usize,
    pub script_separate: bool,
    /// Current folder relative to `assets/` (empty = root).
    pub folder: String,
    pub size: f32,
    pub search: String,
    pub selected: Option<String>,
    entries: Vec<Entry>,
    dirs: Vec<String>,
    stamp: Option<Instant>,
    images: HashMap<String, Option<egui::TextureHandle>>,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self { new_script: None, script_template: 0, script_separate: true, folder: String::new(), size: 92.0, search: String::new(), selected: None, entries: Vec::new(), dirs: Vec::new(), stamp: None, images: HashMap::new() }
    }
}

fn walk_dirs(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut subs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir() && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
    subs.sort();
    for p in subs {
        if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
        walk_dirs(root, &p, out);
    }
}

fn reveal(path: &Path) {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer").arg(format!("/select,{}", path.display())).spawn();
    #[cfg(not(windows))]
    let _ = std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" }).arg(path.parent().unwrap_or(path)).spawn();
}

pub fn open_external(path: &Path) {
    #[cfg(windows)]
    let _ = std::process::Command::new("cmd").args(["/C", "start", ""]).arg(path).spawn();
    #[cfg(not(windows))]
    let _ = std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" }).arg(path).spawn();
}

fn join(folder: &str, name: &str) -> String {
    if folder.is_empty() {
        name.to_string()
    } else {
        format!("{folder}/{name}")
    }
}

impl Editor {
    fn new_folder(&mut self) {
        let base = self.project.assets_dir().join(&self.browser.folder);
        let mut p = base.join("New Folder");
        let mut n = 1;
        while p.exists() {
            p = base.join(format!("New Folder {n}"));
            n += 1;
        }
        let _ = std::fs::create_dir_all(&p);
        self.browser.stamp = None;
    }

    /// Creates `<folder>/<stem>[_n].<ext>` with the given content (relative to assets) and opens it.
    pub fn new_file(&mut self, default_folder: &str, stem: &str, ext: &str, content: &str) {
        let folder = if self.browser.folder.is_empty() { default_folder.to_string() } else { self.browser.folder.clone() };
        let dir = self.project.assets_dir().join(&folder);
        let _ = std::fs::create_dir_all(&dir);
        let mut path = dir.join(format!("{stem}.{ext}"));
        let mut n = 1;
        while path.exists() {
            n += 1;
            path = dir.join(format!("{stem}_{n}.{ext}"));
        }
        match std::fs::write(&path, content) {
            Ok(()) => {
                self.reload_assets();
                self.browser.stamp = None;
                self.prefs.open(&path);
            }
            Err(e) => self.console.error(e.to_string()),
        }
    }

    fn new_shader(&mut self) {
        let dir = self.project.assets_dir().join(if self.browser.folder.is_empty() { "shaders" } else { &self.browser.folder });
        let _ = std::fs::create_dir_all(&dir);
        let mut path = dir.join("new_shader.wgsl");
        let mut n = 1;
        while path.exists() {
            path = dir.join(format!("new_shader_{n}.wgsl"));
            n += 1;
        }
        match std::fs::write(&path, SHADER_TEMPLATE) {
            Ok(()) => {
                self.reload_assets();
                self.browser.stamp = None;
                self.prefs.open(&path);
            }
            Err(e) => self.console.error(e.to_string()),
        }
    }

    /// Copies files into the project's assets folder (current folder, or a folder by type at the root).
    pub fn import_assets(&mut self, files: &[PathBuf]) {
        for file in files {
            let Some(name) = file.file_name() else { continue };
            let sub = if self.browser.folder.is_empty() { folder_for(file).to_string() } else { self.browser.folder.clone() };
            let dir = self.project.assets_dir().join(&sub);
            let mut dest = dir.join(name);
            let mut n = 1;
            while dest.exists() {
                let stem = file.file_stem().map_or(String::new(), |s| s.to_string_lossy().to_string());
                let ext = file.extension().map_or(String::new(), |e| format!(".{}", e.to_string_lossy()));
                dest = dir.join(format!("{stem}_{n}{ext}"));
                n += 1;
            }
            let result = std::fs::create_dir_all(&dir).and_then(|_| std::fs::copy(file, &dest));
            match result {
                Ok(_) => self.console.info(tf("Imported {}", &[&dest.strip_prefix(self.project.assets_dir()).unwrap_or(&dest).display().to_string()])),
                Err(e) => self.console.error(format!("Import {} failed: {e}", file.display())),
            }
        }
        self.reload_assets();
        self.browser.stamp = None;
    }

    fn refresh_browser(&mut self) {
        let fresh = self.browser.stamp.is_some_and(|t| t.elapsed() < Duration::from_millis(700));
        if fresh {
            return;
        }
        let root = self.project.assets_dir();
        let mut entries: Vec<Entry> = std::fs::read_dir(root.join(&self.browser.folder))
            .map(|rd| rd.flatten().map(|e| Entry { name: e.file_name().to_string_lossy().to_string(), is_dir: e.path().is_dir() }).filter(|e| !e.name.starts_with('.')).collect())
            .unwrap_or_default();
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        self.browser.entries = entries;
        let mut dirs = Vec::new();
        walk_dirs(&root, &root, &mut dirs);
        self.browser.dirs = dirs;
        self.browser.stamp = Some(Instant::now());
        if !self.browser.folder.is_empty() && !root.join(&self.browser.folder).is_dir() {
            self.browser.folder.clear();
        }
    }

    fn texture_thumb(&mut self, ctx: &egui::Context, path: &str) -> Option<egui::TextureId> {
        if !self.browser.images.contains_key(path) {
            let handle = self.assets.read(path).ok().and_then(|bytes| image::load_from_memory(&bytes).ok()).map(|img| {
                let small = img.thumbnail(128, 128).to_rgba8();
                let color = egui::ColorImage::from_rgba_unmultiplied([small.width() as usize, small.height() as usize], &small);
                ctx.load_texture(format!("thumb:{path}"), color, egui::TextureOptions::LINEAR)
            });
            self.browser.images.insert(path.to_string(), handle);
        }
        self.browser.images[path].as_ref().map(|h| h.id())
    }

    fn move_asset(&mut self, from: &str, into_folder: &str) {
        let name = from.rsplit('/').next().unwrap_or(from);
        let (src, dst) = (self.project.assets_dir().join(from), self.project.assets_dir().join(join(into_folder, name)));
        if src == dst || dst.exists() {
            return;
        }
        match std::fs::rename(&src, &dst) {
            Ok(()) => {
                self.console.warn(tf("Moved {} to {}. Scenes that reference it by path need updating.", &[from, &join(into_folder, name)]));
                self.reload_assets();
                self.browser.stamp = None;
            }
            Err(e) => self.console.error(format!("Move failed: {e}")),
        }
    }

    pub fn assets_ui(&mut self, ui: &mut egui::Ui) {
        let dropped: Vec<PathBuf> = ui.ctx().input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        if !dropped.is_empty() {
            self.import_assets(&dropped);
        }
        self.refresh_browser();
        self.browser_toolbar(ui);
        ui.separator();

        egui::Panel::left("asset-tree").resizable(true).default_size(170.0).show(ui, |ui| self.folder_tree(ui));
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| self.asset_grid(ui));
    }

    fn browser_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if icons::button(ui, Icon::Home, t("Assets root"), false).clicked() {
                self.browser.folder.clear();
                self.browser.stamp = None;
            }
            let mut acc = String::new();
            for seg in self.browser.folder.clone().split('/').filter(|s| !s.is_empty()) {
                icons::show(ui, Icon::ChevronRight, 12.0);
                acc = join(&acc, seg);
                if ui.link(seg).clicked() {
                    self.browser.folder = acc.clone();
                    self.browser.stamp = None;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(egui::Slider::new(&mut self.browser.size, 56.0..=180.0).show_value(false)).on_hover_text(t("Thumbnail size"));
                ui.add(egui::TextEdit::singleline(&mut self.browser.search).hint_text(t("Search")).desired_width(120.0));
                if icons::button(ui, Icon::FolderPlus, t("New folder"), false).clicked() {
                    self.new_folder();
                }
                if icons::button(ui, Icon::FilePlus, t("New script"), false).clicked() {
                    self.browser.new_script = Some(String::new());
                }
                ui.menu_button(egui::RichText::new("◈ +").strong(), |ui| {
                    if ui.button(t("New material (nodes)")).clicked() {
                        self.new_graph(kvas_engine::material_graph::GraphMode::Material);
                        ui.close();
                    }
                    if ui.button(t("New shader (nodes)")).clicked() {
                        self.new_graph(kvas_engine::material_graph::GraphMode::Shader);
                        ui.close();
                    }
                    if ui.button(t("New camera effect (nodes)")).clicked() {
                        self.new_graph(kvas_engine::material_graph::GraphMode::Effect);
                        ui.close();
                    }
                }).response.on_hover_text(t("New node material, shader or camera effect with live preview"));
                if icons::button(ui, Icon::FileCode, t("New shader from template"), false).clicked() {
                    self.new_shader();
                }
                if icons::button(ui, Icon::Upload, t("Import files (or drop them on the window)"), false).clicked() {
                    if let Some(files) = rfd::FileDialog::new().add_filter("Assets", &["obj", "gltf", "glb", "fbx", "png", "jpg", "jpeg", "wgsl", "wav", "ogg", "mp3", "flac", "m4a"]).pick_files() {
                        self.import_assets(&files);
                    }
                }
            });
        });
    }

    fn folder_tree(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            let mut go: Option<String> = None;
            let mut moved: Option<(String, String)> = None;
            let rows: Vec<(String, String, usize)> = std::iter::once((String::new(), t("assets").to_string(), 0))
                .chain(self.browser.dirs.iter().map(|d| (d.clone(), d.rsplit('/').next().unwrap_or(d).to_string(), d.matches('/').count() + 1)))
                .collect();
            for (path, name, depth) in rows {
                let selected = self.browser.folder == path;
                ui.horizontal(|ui| {
                    ui.add_space(depth as f32 * 12.0);
                    icons::show(ui, if selected { Icon::FolderOpen } else { Icon::Folder }, 14.0);
                    let resp = ui.selectable_label(selected, name);
                    if resp.clicked() {
                        go = Some(path.clone());
                    }
                    if let Some(asset) = resp.dnd_release_payload::<AssetDrag>() {
                        moved = Some((asset.0.clone(), path.clone()));
                    }
                });
            }
            if let Some(f) = go {
                self.browser.folder = f;
                self.browser.stamp = None;
            }
            if let Some((from, to)) = moved {
                self.move_asset(&from, &to);
            }
        });
    }

    fn asset_grid(&mut self, ui: &mut egui::Ui) {
        let size = self.browser.size;
        let search = self.browser.search.to_lowercase();
        let mut entries: Vec<(String, bool, bool)> = self.browser.entries.iter().filter(|e| search.is_empty() || e.name.to_lowercase().contains(&search)).map(|e| (e.name.clone(), e.is_dir, false)).collect();
        // The project's scripts appear as ordinary files at the assets root.
        if self.browser.folder.is_empty() {
            entries.extend(self.script_names().into_iter().filter(|n| search.is_empty() || n.to_lowercase().contains(&search)).map(|n| (n, false, true)));
        }
        let mut new_thumbs = 2; // limit GPU previews rendered per frame
        let mut open_folder: Option<String> = None;
        let mut moved: Option<(String, String)> = None;
        let ctx = ui.ctx().clone();

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            if entries.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| ui.weak(t("Empty folder. Drop models, textures or shaders here.")));
            }
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
            let grid = ui.horizontal_wrapped(|ui| {
                for (name, is_dir, is_script) in entries {
                    let path = if is_script { format!("script:{name}") } else { join(&self.browser.folder, &name) };
                    let kind = if is_script { Kind::Script } else { kind_of(&name, is_dir) };
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size + 22.0), Sense::click_and_drag());
                    let selected = self.browser.selected.as_deref() == Some(path.as_str());
                    let hover_drop = is_dir && resp.dnd_hover_payload::<AssetDrag>().is_some();
                    let lift = crate::ui_kit::animate(ui, resp.id.with("lift"), resp.hovered() || hover_drop);
                    crate::ui_kit::tile(ui, rect, resp.id, resp.hovered() || hover_drop, selected);
                    if hover_drop {
                        ui.painter().rect_stroke(rect, 6.0, egui::Stroke::new(1.5, crate::theme::ACCENT), egui::StrokeKind::Inside);
                    }
                    let thumb_rect = egui::Rect::from_min_size(rect.min + egui::vec2(5.0, 5.0), egui::vec2(size - 10.0, size - 10.0)).expand(lift * 2.5);

                    let texture = match kind {
                        Kind::Mesh | Kind::Shader | Kind::Material => {
                            let (mesh, shader) = if kind == Kind::Mesh { (path.as_str(), "") } else { ("primitive:sphere", path.as_str()) };
                            let cached = self.renderer.has_thumbnail(mesh, shader, 128);
                            if cached || new_thumbs > 0 {
                                if !cached {
                                    new_thumbs -= 1;
                                    ctx.request_repaint();
                                }
                                self.renderer.thumbnail(mesh, shader, 128)
                            } else {
                                ctx.request_repaint();
                                None
                            }
                        }
                        Kind::Texture => self.texture_thumb(&ctx, &path),
                        _ => None,
                    };
                    match texture {
                        Some(tex) => {
                            ui.painter().image(tex, thumb_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                        }
                        None => {
                            let icon = match kind {
                                Kind::Folder => Icon::Folder,
                                Kind::Mesh => Icon::FileBox,
                                Kind::Texture => Icon::Image,
                                Kind::Shader => Icon::FileCode,
                                Kind::Audio => Icon::Music,
                                Kind::Script => Icon::FileCode,
                                Kind::Material => Icon::Palette,
                                Kind::Scene => Icon::Layers,
                                Kind::Other => Icon::File,
                            };
                            let tint = kind_color(kind);
                            icon.image(size * 0.45, tint).paint_at(ui, egui::Rect::from_center_size(thumb_rect.center(), egui::Vec2::splat(size * 0.45)));
                        }
                    }
                    if !is_dir {
                        let ext = if is_script { "RS".to_string() } else { name.rsplit('.').next().filter(|e| e.len() <= 5 && *e != name).unwrap_or("").to_uppercase() };
                        if !ext.is_empty() {
                            let g = ui.painter().layout_no_wrap(ext, egui::FontId::proportional(9.5), Color32::WHITE);
                            let pill = egui::Rect::from_min_size(egui::pos2(thumb_rect.right() - g.size().x - 10.0, thumb_rect.bottom() - 16.0), g.size() + egui::vec2(8.0, 3.0));
                            ui.painter().rect_filled(pill, 4.0, kind_color(kind).gamma_multiply(0.9));
                            ui.painter().galley(pill.min + egui::vec2(4.0, 1.5), g, Color32::WHITE);
                        }
                    }
                    let shown = if is_script { name.clone() } else { name.clone() };
                    let galley = ui.painter().layout(shown, egui::FontId::proportional(12.0), crate::theme::pal().text, size - 8.0);
                    let text_pos = egui::pos2(rect.center().x - galley.size().x.min(size - 8.0) / 2.0, rect.max.y - 20.0);
                    ui.painter().with_clip_rect(rect).galley(text_pos, galley, crate::theme::pal().text);

                    if resp.clicked() {
                        self.browser.selected = Some(path.clone());
                    }
                    if resp.drag_started() && !is_dir {
                        if is_script {
                            resp.dnd_set_drag_payload(ScriptDrag(name.clone()));
                        } else {
                            resp.dnd_set_drag_payload(AssetDrag(path.clone()));
                        }
                    }
                    if is_dir {
                        if let Some(asset) = resp.dnd_release_payload::<AssetDrag>() {
                            moved = Some((asset.0.clone(), path.clone()));
                        }
                    }
                    if resp.double_clicked() {
                        match kind {
                            Kind::Folder => open_folder = Some(path.clone()),
                            Kind::Scene => self.open_scene(&path),
                            Kind::Shader => self.prefs.open(&self.project.assets_dir().join(&path)),
                            Kind::Audio => self.preview_sound(&path),
                            Kind::Material => self.open_material(&path),
                            Kind::Script => self.open_script(&name),
                            _ => open_external(&self.project.assets_dir().join(&path)),
                        }
                    }
                    let tip = if is_script { format!("{name}  (script)") } else { path.clone() };
                    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand).context_menu(|ui| {
                        if is_script {
                            if ui.button(t("Open in code editor")).clicked() {
                                self.open_script(&name);
                                ui.close();
                            }
                            if ui.add_enabled(self.selection.is_some(), egui::Button::new(t("Attach to selected object"))).clicked() {
                                if let Some(e) = self.selection {
                                    self.attach_script(e, &name);
                                }
                                ui.close();
                            }
                            if ui.button(t("Show in folder")).clicked() {
                                reveal(&self.script_source(&name));
                                ui.close();
                            }
                            return;
                        }
                        let full = self.project.assets_dir().join(&path);
                        if kind == Kind::Folder && ui.button(t("Open")).clicked() {
                            open_folder = Some(path.clone());
                            ui.close();
                        }
                        if kind == Kind::Scene && ui.button(t("Open scene")).clicked() {
                            self.open_scene(&path);
                            ui.close();
                        }
                        if matches!(kind, Kind::Mesh | Kind::Texture) && ui.button(if kind == Kind::Mesh { t("Add to scene") } else { t("Add as sprite") }).clicked() {
                            let at = self.camera.pos + self.camera.forward() * 6.0;
                            self.create_from_asset(&path, at);
                            ui.close();
                        }
                        if matches!(kind, Kind::Shader) && ui.button(t("Edit with preview")).clicked() {
                            self.open_material(&path);
                            ui.close();
                        }
                        if matches!(kind, Kind::Shader) && ui.button(t("Open in code editor")).clicked() {
                            self.prefs.open(&full);
                            ui.close();
                        }
                        if kind != Kind::Folder && ui.button(t("Open in default app")).clicked() {
                            open_external(&full);
                            ui.close();
                        }
                        if ui.button(t("Show in folder")).clicked() {
                            reveal(&full);
                            ui.close();
                        }
                        if ui.button(t("Delete")).clicked() {
                            let confirmed = rfd::MessageDialog::new()
                                .set_title(t("Delete"))
                                .set_description(tf("Delete {} permanently?", &[&path]))
                                .set_buttons(rfd::MessageButtons::YesNo)
                                .show()
                                == rfd::MessageDialogResult::Yes;
                            if confirmed {
                                let _ = if is_dir { std::fs::remove_dir_all(&full) } else { std::fs::remove_file(&full) };
                                self.reload_assets();
                                self.browser.stamp = None;
                            }
                            ui.close();
                        }
                    });
                }
            });
            let _ = grid;
            // Empty space: right-click for the create menu.
            let rest = ui.allocate_response(ui.available_size().max(egui::vec2(10.0, 80.0)), Sense::click());
            rest.context_menu(|ui| self.browser_create_menu(ui));
            if rest.clicked() {
                self.browser.selected = None;
            }
        });

        self.new_script_window(&ctx);
        if let Some(f) = open_folder {
            self.browser.folder = f;
            self.browser.stamp = None;
        }
        if let Some((from, to)) = moved {
            self.move_asset(&from, &to);
        }
    }

    fn browser_create_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button(t("New Folder")).clicked() {
            self.new_folder();
            ui.close();
        }
        if ui.button(t("New Script...")).clicked() {
            self.browser.new_script = Some(String::new());
            ui.close();
        }
        if ui.button(t("New Shader")).clicked() {
            self.new_shader();
            ui.close();
        }
        if ui.button(t("New Material (nodes)")).clicked() {
            self.new_graph(kvas_engine::material_graph::GraphMode::Material);
            ui.close();
        }
        if ui.button(t("New Shader (nodes)")).clicked() {
            self.new_graph(kvas_engine::material_graph::GraphMode::Shader);
            ui.close();
        }
        if ui.button(t("New Camera Effect (nodes)")).clicked() {
            self.new_graph(kvas_engine::material_graph::GraphMode::Effect);
            ui.close();
        }
        if ui.button(t("New Camera Shader")).clicked() {
            self.new_file("shaders/fx", "effect", "wgsl", crate::script_templates::POST_EFFECT_TEMPLATE);
            ui.close();
        }
        if ui.button(t("New Text File")).clicked() {
            self.new_file("", "notes", "txt", "");
            ui.close();
        }
        ui.separator();
        if ui.button(t("Import Files...")).clicked() {
            if let Some(files) = rfd::FileDialog::new().add_filter("Assets", &["obj", "gltf", "glb", "fbx", "png", "jpg", "jpeg", "wgsl", "wav", "ogg", "mp3", "flac", "m4a"]).pick_files() {
                self.import_assets(&files);
            }
            ui.close();
        }
        if ui.button(t("Show in Folder")).clicked() {
            reveal(&self.project.assets_dir().join(&self.browser.folder));
            ui.close();
        }
        if ui.button(t("Open Project in Code Editor")).clicked() {
            self.prefs.open(&self.project.root);
            ui.close();
        }
    }

    fn new_script_window(&mut self, ctx: &egui::Context) {
        let Some(mut name) = self.browser.new_script.clone() else { return };
        let mut open = true;
        let mut create = false;
        let (mut template, mut separate) = (self.browser.script_template, self.browser.script_separate);
        egui::Window::new(t("New Script")).open(&mut open).collapsible(false).resizable(false).default_pos([260.0, 120.0]).show(ctx, |ui| {
            ui.label(t("Name (CamelCase):"));
            let resp = ui.add(egui::TextEdit::singleline(&mut name).desired_width(280.0));
            resp.request_focus();
            ui.add_space(6.0);
            ui.label(t("Template"));
            egui::ComboBox::from_id_salt("script-template").selected_text(crate::script_templates::label(template)).width(280.0).show_ui(ui, |ui| {
                for i in 0..crate::script_templates::TEMPLATES.len() {
                    ui.selectable_value(&mut template, i, crate::script_templates::label(i));
                }
            });
            ui.weak(crate::script_templates::hint(template));
            ui.add_space(4.0);
            ui.checkbox(&mut separate, t("Create as a separate file (scripts/src/name.rs)"));
            ui.add_space(6.0);
            create = ui.add(egui::Button::new(egui::RichText::new(t("Create")).color(Color32::WHITE)).fill(crate::theme::ACCENT)).clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
        });
        self.browser.script_template = template;
        self.browser.script_separate = separate;
        if create && self.new_script(&name, template, separate) {
            open = false;
        }
        self.browser.new_script = if open { Some(name) } else { None };
    }
}
