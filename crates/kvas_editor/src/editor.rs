//! Editor state and every action on it. Panels (hierarchy, inspector, viewport, assets) are
//! `impl Editor` blocks in their own files and only call the methods defined here.

use crate::console::{Console, Level};
use crate::export;
use crate::gizmo::GizmoState;
use crate::scripting::{lib_path, ScriptState};
use crate::viewport::EditorCamera;
use eframe::egui;
use eframe::egui_wgpu::RenderState;
use glam::Vec3;
use kvas_engine::assets::Assets;
use kvas_engine::prelude::*;
use kvas_engine::project::Project;
use kvas_engine::registry::Registry;
use kvas_engine::runtime::Runtime;
use kvas_engine::scene::{self, Scene};
use kvas_engine::script::ScriptHost;
use kvas_render::{shell, Renderer};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Move,
    Rotate,
    Scale,
}

pub struct Play {
    pub runtime: Runtime,
    pub paused: bool,
    pub step: bool,
    /// Mouse captured by the game (Escape frees it, a click on the view takes it back).
    pub mouse: shell::MouseCapture,
}

#[derive(Default)]
struct Undo {
    undo: Vec<String>,
    redo: Vec<String>,
    /// Scene state captured when an interaction started; pushed once it turns out to be an edit.
    pending: Option<String>,
}

pub struct NewProject {
    pub name: String,
    pub folder: String,
}

pub struct Editor {
    pub project: Project,
    pub registry: Registry,
    pub assets: Assets,
    pub asset_list: Vec<String>,
    pub scene: Scene,
    pub scene_path: Option<String>,
    pub dirty: bool,
    pub play: Option<Play>,
    pub selection: Option<Entity>,
    pub multi_selection: Vec<Entity>,
    pub animation_open: bool,
    pub gui_open: bool,
    pub help: crate::help::HelpState,
    pub api: crate::api_docs::ApiState,
    pub mesh_edit: crate::mesh_editor::MeshEditState,
    pub image_editor: crate::image_editor::ImageEditorState,
    pub training: crate::neural_editor::NeuralTraining,
    pub console: Console,
    pub scripts: ScriptState,
    pub prefs: crate::prefs::Prefs,
    pub renderer: Renderer,
    pub camera: EditorCamera,
    pub tool: Tool,
    pub local_space: bool,
    pub show_colliders: bool,
    pub show_grid: bool,
    /// Scene view mode: 0 lit, 1 albedo, 2 normals, 3 depth, 4 UV.
    pub debug_view: u32,
    pub gizmo: GizmoState,
    /// Node material editor window.
    pub mat: crate::node_editor::MaterialEditor,
    /// Built-in code editor window.
    pub code: crate::code_editor::CodeEditor,
    /// Build Game window.
    pub build_dialog: crate::build_dialog::BuildDialog,
    /// Texture paint brush.
    pub paint: crate::paint_tool::PaintTool,
    /// State of the Sound panel.
    pub sound: crate::audio_panel::AudioPanel,
    /// Snapping modes and steps for moving, rotating and scaling.
    pub snap: crate::snap::SnapSettings,
    /// CPU copies of meshes for vertex / face snapping.
    pub mesh_cache: std::collections::HashMap<String, Option<std::sync::Arc<kvas_engine::mesh::MeshData>>>,
    pub new_project: Option<NewProject>,
    pub filter: String,
    pub settings: crate::settings::SettingsState,
    pub browser: crate::assets_panel::BrowserState,
    pub status: String,
    pub fps: f32,
    pub build: Option<Receiver<Result<PathBuf, String>>>,
    /// Sound output while a game runs; `None` without an audio device.
    pub audio: Option<kvas_audio::AudioEngine>,
    undo: Undo,
    started: Instant,
    pub play_center: Option<egui::Pos2>,
    /// The cursor was hidden by a running game.
    pub cursor_hidden: bool,
    /// A script asked to quit; Play stops after the frame.
    pub pending_stop: bool,
    /// Viewing the scene through this camera (Inspector "Look through"), with its shaders.
    pub look_through: Option<Entity>,
    /// Last valid viewport camera/physical dimensions, used for clean PNG export.
    pub last_view: Option<(kvas_render::View, [u32; 2])>,
    last_frame: Instant,
    last_refresh: Instant,
}

impl Editor {
    pub fn new(rs: &RenderState, project: Project) -> Editor {
        let assets = Assets::Dir(project.assets_dir());
        kvas_engine::meshmod::set_assets(assets.clone());
        let prefs = crate::prefs::Prefs::load();
        let (show_grid, show_colliders, local_space) = (prefs.show_grid, prefs.show_colliders, prefs.local_space);
        let snap = prefs.snap.clone();
        let mut ed = Editor {
            registry: Registry::default(),
            asset_list: assets.list(),
            renderer: Renderer::new(rs, assets.clone()),
            assets,
            scene: Scene::default(),
            scene_path: None,
            dirty: false,
            play: None,
            selection: None,
            multi_selection: Vec::new(),
            animation_open: false,
            gui_open: false,
            help: Default::default(),
            api: Default::default(),
            mesh_edit: Default::default(),
            image_editor: Default::default(),
            training: Default::default(),
            console: Console::default(),
            scripts: ScriptState::new(),
            browser: Default::default(),
            prefs,
            camera: EditorCamera::default(),
            tool: Tool::Move,
            local_space,
            show_colliders,
            show_grid,
            debug_view: 0,
            gizmo: GizmoState::default(),
            mat: Default::default(),
            code: Default::default(),
            build_dialog: Default::default(),
            paint: Default::default(),
            sound: Default::default(),
            snap,
            mesh_cache: Default::default(),
            new_project: None,
            filter: String::new(),
            settings: Default::default(),
            status: String::new(),
            fps: 0.0,
            build: None,
            audio: None,
            undo: Undo::default(),
            started: Instant::now(),
            play_center: None,
            cursor_hidden: false,
            pending_stop: false,
            look_through: None,
            last_view: None,
            last_frame: Instant::now(),
            last_refresh: Instant::now(),
            project,
        };
        ed.scripts.auto_reload = ed.prefs.scripts_auto_reload;
        ed.browser.size = ed.prefs.thumbnail_size.clamp(56.0, 180.0);
        ed.enter_project();
        ed
    }

    /// Copies the view toggles into the saved preferences.
    pub fn sync_prefs(&mut self) {
        self.prefs.show_grid = self.show_grid;
        self.prefs.show_colliders = self.show_colliders;
        self.prefs.local_space = self.local_space;
        self.prefs.scripts_auto_reload = self.scripts.auto_reload;
        self.prefs.thumbnail_size = self.browser.size;
        self.prefs.snap = self.snap.clone();
    }

    /// Loads the start scene and the scripts library of the current project.
    fn enter_project(&mut self) {
        self.last_view = None;
        match crate::library::ensure_assets(&self.project.root) {
            Ok(0) => {}
            Ok(n) => {
                self.console.info(crate::i18n::tf("Built-in materials and shaders added to the project: {}", &[&n.to_string()]));
                self.reload_assets();
            }
            Err(e) => self.console.error(e),
        }
        crate::hub::add_recent(&self.project.root);
        let log = self.console.sender();
        let auto_reload = self.scripts.auto_reload;
        self.scripts = ScriptState::new();
        self.scripts.auto_reload = auto_reload;
        self.scripts.load_existing(&self.project, &log);
        let start = self.project.config.start_scene.clone();
        self.open_scene(&start);
        self.console.info(format!("Opened project '{}' ({})", self.project.config.name, self.project.root.display()));
    }

    pub fn open_project(&mut self, root: &Path) {
        match Project::open(root) {
            Ok(project) => {
                self.stop_play();
                self.assets = Assets::Dir(project.assets_dir());
                kvas_engine::meshmod::set_assets(self.assets.clone());
                self.renderer.set_assets(self.assets.clone());
                if let Err(e) = crate::library::write_api_files(&project.root) {
                    self.console.error(e);
                }
                match crate::script_files::ensure_auto(&project.root) {
                    Ok(true) => self.console.info("Scripts now register automatically: one file per script"),
                    Ok(false) => {}
                    Err(e) => self.console.error(e),
                }
                self.project = project;
                self.reload_assets();
                self.enter_project();
            }
            Err(e) => self.console.error(e),
        }
    }

    pub fn create_project(&mut self, folder: &Path, name: &str, kind: &str) {
        match export::create_project(folder, name, kind) {
            Ok(project) => {
                let root = project.root.clone();
                self.open_project(&root);
                self.scripts.start_build(&self.project, self.console.sender());
            }
            Err(e) => self.console.error(e),
        }
    }

    pub fn reload_assets(&mut self) {
        self.asset_list = self.assets.list();
        self.mesh_cache.clear();
        self.last_refresh = Instant::now();
    }

    /// Exports the last scene/game camera as a clean PNG without editor overlays or HUD.
    /// A native save dialog chooses the destination. Oversized viewports are scaled
    /// to at most 4096 pixels per edge while preserving their aspect ratio.
    pub fn export_view_png(&mut self) {
        let Some((view, mut size)) = self.last_view else {
            self.console.error("No valid viewport camera is available for export"); return;
        };
        let Some(path) = rfd::FileDialog::new().set_title("Export View PNG").set_directory(&self.project.root)
            .add_filter("PNG image", &["png"]).set_file_name("scene.png").save_file() else { return; };
        let largest = size[0].max(size[1]);
        if largest > 4096 { size = size.map(|edge| ((edge as f64 * 4096.0 / largest as f64) as u32).max(1)); }
        let debug = kvas_render::debug::DebugDraw::default();
        let time = self.play.as_ref().map_or_else(|| self.elapsed(), |play| play.runtime.time.elapsed);
        let frame = kvas_render::Frame { view, size, time, dt: 0.0, debug: &debug, shadows: true, debug_view: self.debug_view, cinematic: self.is_playing() };
        let scene = self.play.as_ref().map_or(&self.scene, |play| &play.runtime.scene);
        match self.renderer.capture(scene, &frame).and_then(|image| image.save_with_format(&path, image::ImageFormat::Png).map_err(|e| e.to_string())) {
            Ok(()) => {
                self.status = crate::i18n::tf("Exported PNG: {}", &[&path.display().to_string()]);
                self.console.info(format!("Exported {}x{} PNG: {}", size[0], size[1], path.display()));
            }
            Err(error) => self.console.error(format!("PNG export failed: {error}")),
        }
    }

    // ---------- scene access ----------

    /// The scene being shown: the running game's copy while playing, otherwise the edited one.
    pub fn active(&self) -> &Scene {
        self.play.as_ref().map_or(&self.scene, |p| &p.runtime.scene)
    }

    pub fn active_mut(&mut self) -> &mut Scene {
        match &mut self.play {
            Some(p) => &mut p.runtime.scene,
            None => &mut self.scene,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.play.is_some()
    }

    pub fn elapsed(&self) -> f32 {
        self.started.elapsed().as_secs_f32()
    }

    pub fn entity_names(&self) -> Vec<String> {
        let w = &self.active().world;
        let mut names: Vec<String> = w.iter::<Name>().map(|(_, n)| n.0.clone()).filter(|n| !n.is_empty()).collect();
        names.sort();
        names.dedup();
        names
    }

    fn position_of(&self, e: Entity) -> Option<usize> {
        self.active().world.entities().iter().position(|&x| x == e)
    }

    fn entity_at(&self, pos: usize) -> Option<Entity> {
        self.active().world.entities().get(pos).copied()
    }

    // ---------- creating and changing entities ----------

    fn spawn_point(&self) -> Vec3 {
        let (o, d) = (self.camera.pos, self.camera.forward());
        if d.y < -0.05 {
            let t = -o.y / d.y;
            if (1.0..80.0).contains(&t) {
                return o + d * t;
            }
        }
        o + d * 8.0
    }

    pub fn create_entity(&mut self, name: &str, setup: impl FnOnce(&mut kvas_ecs::World, Entity)) -> Entity {
        self.begin_edit();
        let at = self.spawn_point();
        let parent = self.selection.filter(|_| false); // new objects go to the scene root
        let w = &mut self.active_mut().world;
        let e = hierarchy::spawn_named(w, name);
        w.insert(e, Transform::at(at));
        setup(w, e);
        if let Some(p) = parent {
            hierarchy::set_parent(w, e, Some(p));
        }
        self.selection = Some(e);
        e
    }

    pub fn create_empty(&mut self) {
        self.create_entity("Empty", |_, _| {});
    }

    pub fn create_primitive(&mut self, primitive: &str, physics: bool) {
        let kind = match primitive {
            "sphere" => ShapeType::Sphere,
            "plane" => ShapeType::Plane,
            "cylinder" => ShapeType::Cylinder,
            "capsule" => ShapeType::Capsule,
            "cone" => ShapeType::Cone,
            "torus" => ShapeType::Torus,
            "rock" => ShapeType::Rock,
            _ => ShapeType::Cube,
        };
        let name = primitive[..1].to_uppercase() + &primitive[1..];
        let shape = Shape::of(kind);
        let collider = match kind {
            ShapeType::Plane => None,
            ShapeType::Sphere => Some(Collider { shape: ShapeKind::Sphere, size: Vec3::splat(shape.radius * 2.0), ..Default::default() }),
            ShapeType::Capsule => Some(Collider {
                shape: ShapeKind::Capsule,
                size: Vec3::new(shape.radius * 2.0, shape.height, shape.radius * 2.0),
                ..Default::default()
            }),
            ShapeType::Cylinder | ShapeType::Cone => Some(Collider {
                shape: ShapeKind::Cylinder,
                size: Vec3::new(shape.radius * 2.0, shape.height, shape.radius * 2.0),
                ..Default::default()
            }),
            ShapeType::Torus => Some(Collider {
                size: Vec3::new((shape.radius + shape.radius2) * 2.0, shape.radius2 * 2.0, (shape.radius + shape.radius2) * 2.0),
                ..Default::default()
            }),
            ShapeType::Rock => Some(Collider { size: shape.size, ..Default::default() }),
            ShapeType::Cube => Some(Collider::default()),
        };
        self.create_entity(&name, |w, e| {
            w.insert(e, MeshRenderer { mesh: String::new(), ..Default::default() });
            w.insert(e, shape);
            if let Some(c) = collider {
                w.insert(e, c);
            }
            if physics {
                w.insert(e, RigidBody::default());
            }
        });
    }

    /// A sphere with a ready-made material: clear glass or polished chrome.
    pub fn create_material_sphere(&mut self, glass: bool) {
        self.create_primitive("sphere", false);
        let Some(e) = self.selection else { return };
        let world = &mut self.active_mut().world;
        if let Some(name) = world.get_mut::<Name>(e) {
            name.0 = if glass { "Glass Sphere".into() } else { "Chrome Sphere".into() };
        }
        if let Some(m) = world.get_mut::<MeshRenderer>(e) {
            if glass {
                (m.color, m.roughness, m.reflectivity, m.cast_shadows) = ([0.78, 0.92, 1.0, 0.16], 0.03, 1.0, false);
            } else {
                (m.color, m.roughness, m.metallic, m.reflectivity) = ([0.92, 0.92, 0.96, 1.0], 0.08, 1.0, 1.0);
            }
        }
    }

    /// Procedural effects: grass patch, cloth, voxel smoke, voxel clouds.
    pub fn create_procedural(&mut self, kind: &str) {
        let (name, lift) = match kind {
            "grass" => ("Grass", 0.0),
            "tree" => ("Tree", 0.0),
            "tree_pine" => ("Pine", 0.0),
            "tree_palm" => ("Palm", 0.0),
            "bush" => ("Bush", 0.0),
            k if k.starts_with("decal") => ("Decal", 0.0),
            "cloth" => ("Cloth", 4.0),
            "smoke" => ("Voxel Smoke", 0.0),
            "water_voxel" => ("Voxel Water", 0.0),
            "water" => ("Water", 0.0),
            "water_real" => ("Realistic Water", 0.0),
            "paint_plane" => ("Painted Surface", 0.0),
            _ => ("Voxel Clouds", 28.0),
        };
        let e = self.create_entity(name, |w, e| {
            if let Some(p) = kind.strip_prefix("decal") {
                w.insert(e, kvas_engine::decal::preset(p.trim_start_matches(':')));
                if let Some(t) = w.get_mut::<Transform>(e) {
                    t.scale = Vec3::new(2.0, 1.0, 2.0);
                }
                return;
            }
            if kind.starts_with("tree") || kind == "bush" {
                let tree = match kind {
                    "tree_pine" => Tree { style: TreeStyle::Conifer, height: 7.5, trunk_radius: 0.2, levels: 2, branches: 9, branch_angle: 78.0, branch_length: 0.5, leaf_density: 10, leaf_size: 0.6, crookedness: 0.12, leaf_color: [0.02, 0.07, 0.03, 1.0], leaf_color_2: [0.04, 0.11, 0.04, 1.0], ..Default::default() },
                    "tree_palm" => Tree { style: TreeStyle::Palm, height: 6.0, trunk_radius: 0.16, crookedness: 0.6, leaf_density: 6, leaf_size: 0.9, bark_color: [0.45, 0.36, 0.25, 1.0], ..Default::default() },
                    "bush" => Tree { style: TreeStyle::Bush, height: 1.5, trunk_radius: 0.07, levels: 2, branches: 3, leaf_density: 12, leaf_size: 0.55, ..Default::default() },
                    _ => Tree::default(),
                };
                w.insert(e, tree);
                return;
            }
            w.insert(e, MeshRenderer::default());
            match kind {
                "grass" => w.insert(e, Grass::default()),
                "cloth" => w.insert(e, Cloth::default()),
                "smoke" => w.insert(e, VoxelSmoke::default()),
                "water_voxel" => w.insert(e, VoxelWater { style: WaterStyle::Voxel, cell: 0.6, ..Default::default() }),
                "water" => w.insert(e, VoxelWater::default()),
                "water_real" => w.insert(e, VoxelWater { style: WaterStyle::Realistic, wave_height: 0.25, ..Default::default() }),
                "paint_plane" => {
                    w.insert(e, Shape { kind: ShapeType::Plane, size: Vec3::new(24.0, 1.0, 24.0), rings: 4, ..Shape::of(ShapeType::Plane) });
                    w.insert(e, TexturePaint::default())
                }
                _ => w.insert(e, VoxelClouds::default()),
            }
        });
        if let Some(t) = self.active_mut().world.get_mut::<Transform>(e) {
            t.position.y += lift;
        }
    }

    /// A particle emitter from a preset ("fire", "smoke", ...).
    pub fn create_particles(&mut self, key: &str, label: &str) {
        let key = key.to_string();
        let e = self.create_entity(label, move |w, e| w.insert(e, ParticleEmitter::preset(&key)));
        if let Some(t) = self.active_mut().world.get_mut::<Transform>(e) {
            t.position.y += 0.2;
        }
    }

    pub fn create_light(&mut self, kind: LightKind) {
        let name = match kind {
            LightKind::Directional => "Directional Light",
            LightKind::Point => "Point Light",
            LightKind::Spot => "Spot Light",
        };
        let e = self.create_entity(name, |w, e| w.insert(e, Light { kind, ..Default::default() }));
        let mut up = Vec3::new(0.0, 3.0, 0.0);
        if let Some(t) = self.active_mut().world.get_mut::<Transform>(e) {
            up += t.position;
            t.position = up;
            match kind {
                LightKind::Directional => t.rotation = Vec3::new(-50.0, -30.0, 0.0),
                LightKind::Spot => t.rotation = Vec3::new(-75.0, 0.0, 0.0),
                LightKind::Point => {}
            }
        }
    }

    pub fn create_camera(&mut self) {
        let (pos, yaw, pitch) = (self.camera.pos, self.camera.yaw, self.camera.pitch);
        self.create_entity("Camera", |w, e| {
            w.insert(e, Camera { primary: false, ..Default::default() });
            if let Some(t) = w.get_mut::<Transform>(e) {
                t.position = pos;
                t.rotation = Vec3::new(pitch.to_degrees(), yaw.to_degrees(), 0.0);
            }
        });
    }

    /// Places a model as an object, or an image as a flat sprite facing +Z.
    pub fn create_from_asset(&mut self, path: &str, at: Vec3) {
        let name = Path::new(path).file_stem().map_or("Object".into(), |s| s.to_string_lossy().to_string());
        let lower = path.to_ascii_lowercase();
        let is_image = [".png", ".jpg", ".jpeg"].iter().any(|e| lower.ends_with(e));
        let asset = path.to_string();
        let aspect = if is_image {
            self.assets
                .read(path)
                .ok()
                .and_then(|b| image::load_from_memory(&b).ok())
                .map_or(1.0, |i| i.width() as f32 / i.height().max(1) as f32)
        } else {
            1.0
        };
        // FBX / separate-map models: pick up albedo, normal and metal/rough maps that sit next to the file.
        let pbr = if is_image { Default::default() } else { kvas_engine::materials::discover(&self.assets, path) };
        let e = self.create_entity(&name, |w, e| {
            if is_image {
                w.insert(e, Shape { size: Vec3::new(2.0 * aspect, 1.0, 2.0), rings: 2, ..Shape::of(ShapeType::Plane) });
                w.insert(e, MeshRenderer { mesh: String::new(), texture: asset, roughness: 1.0, emissive: 0.85, ..Default::default() });
            } else {
                let pbr_on = !pbr.normal.is_empty() || !pbr.orm.is_empty();
                w.insert(
                    e,
                    MeshRenderer {
                        mesh: asset,
                        texture: pbr.albedo,
                        normal_map: pbr.normal,
                        orm_map: pbr.orm,
                        metallic: if pbr_on { 1.0 } else { 0.0 },
                        roughness: if pbr_on { 1.0 } else { 0.5 },
                        ..Default::default()
                    },
                );
            }
        });
        if !is_image {
            self.fit_collider(e);
        }
        if let Some(t) = self.active_mut().world.get_mut::<Transform>(e) {
            t.position = at;
            if is_image {
                t.rotation = Vec3::new(90.0, 0.0, 0.0);
            }
        }
    }

    /// Gives a model object a box collider that matches the model bounds (a child, so the object can
    /// still get a RigidBody and the pivot does not matter).
    pub fn fit_collider(&mut self, e: Entity) -> bool {
        let data = if let Some(dynamic) = self.active().world.get::<DynamicMesh>(e) { dynamic.data.clone() }
            else {
                let Some(mesh) = kvas_engine::components::mesh_key(&self.active().world, e) else { return false };
                let Ok(data) = kvas_engine::mesh::MeshData::load(&self.assets, &mesh) else { return false };
                std::sync::Arc::new(data)
            };
        let (min, max) = data.bounds();
        if !min.is_finite() || !max.is_finite() { return false; }
        let size = (max - min).max(Vec3::splat(0.02));
        self.begin_edit();
        let existing = hierarchy::children(&self.active().world, e).into_iter().find(|&c| hierarchy::name_of(&self.active().world, c) == "Auto Collider" && self.active().world.has::<Collider>(c));
        let child = existing.unwrap_or_else(|| hierarchy::spawn_named(&mut self.active_mut().world, "Auto Collider"));
        let world = &mut self.active_mut().world;
        world.insert(child, Transform::at((min + max) * 0.5));
        world.insert(child, Collider { size, ..Default::default() });
        world.insert(child, Parent(e));
        true
    }

    /// Dropping a shader (.wgsl) or an image on an object assigns it as the material shader / texture.
    /// Returns false for other asset types.
    pub fn apply_asset_to(&mut self, e: Entity, path: &str) -> bool {
        let lower = path.to_ascii_lowercase();
        let is_shader = lower.ends_with(".wgsl") || lower.ends_with(".kmat") || lower.ends_with(".kshader");
        let is_image = [".png", ".jpg", ".jpeg"].iter().any(|x| lower.ends_with(x));
        if !is_shader && !is_image {
            return false;
        }
        self.begin_edit();
        let world = &mut self.active_mut().world;
        if !world.has::<MeshRenderer>(e) {
            world.insert(e, MeshRenderer::default());
        }
        let m = world.get_mut::<MeshRenderer>(e).unwrap();
        if is_shader {
            m.shader = path.to_string();
        } else {
            m.texture = path.to_string();
        }
        self.selection = Some(e);
        self.status = format!("{} {path}", if is_shader { crate::i18n::t("Shader set:") } else { crate::i18n::t("Texture set:") });
        true
    }

    /// True for assets that can be dropped onto an existing object.
    pub fn apply_asset_to_ok(&self, path: &str) -> bool {
        let l = path.to_ascii_lowercase();
        l.ends_with(".wgsl") || l.ends_with(".kmat") || l.ends_with(".kshader") || [".png", ".jpg", ".jpeg"].iter().any(|x| l.ends_with(x))
    }

    pub fn add_component(&mut self, e: Entity, name: &str) {
        self.begin_edit();
        if let Some(info) = self.registry.get(name) {
            (info.add)(&mut self.active_mut().world, e);
        }
    }

    pub fn remove_component(&mut self, e: Entity, name: &str) {
        self.begin_edit();
        if let Some(info) = self.registry.get(name) {
            (info.remove)(&mut self.active_mut().world, e);
        }
        if name == "MeshModifiers" {
            let w = &mut self.active_mut().world;
            w.remove::<DynamicMesh>(e);
            w.remove::<ModifierStateCell>(e);
        }
    }

    pub fn delete_entity(&mut self, e: Entity) {
        self.begin_edit();
        hierarchy::despawn_recursive(&mut self.active_mut().world, e);
        if self.selection.is_some_and(|s| !self.active().world.is_alive(s)) {
            self.selection = None;
        }
    }

    pub fn duplicate_entity(&mut self, e: Entity) {
        self.begin_edit();
        let reg = std::mem::take(&mut self.registry);
        let copy = scene::duplicate(&mut self.active_mut().world, &reg, e);
        self.registry = reg;
        self.selection = Some(copy);
    }

    pub fn reparent(&mut self, child: Entity, parent: Option<Entity>) {
        self.begin_edit();
        if !hierarchy::set_parent(&mut self.active_mut().world, child, parent) {
            self.status = crate::i18n::t("Cannot make an object a child of itself or its descendants").into();
        }
    }

    // ---------- scenes ----------

    pub fn new_scene(&mut self) {
        self.stop_play();
        self.scene = Scene::default();
        self.scene_path = None;
        self.selection = None;
        self.undo = Undo::default();
        self.dirty = false;
        self.create_camera_and_light();
    }

    fn create_camera_and_light(&mut self) {
        let w = &mut self.scene.world;
        let cam = hierarchy::spawn_named(w, "Main Camera");
        w.insert(cam, Transform { position: Vec3::new(0.0, 3.0, 8.0), rotation: Vec3::new(-15.0, 0.0, 0.0), ..Default::default() });
        w.insert(cam, Camera::default());
        let sun = hierarchy::spawn_named(w, "Sun");
        w.insert(sun, Transform { position: Vec3::new(0.0, 10.0, 0.0), rotation: Vec3::new(-50.0, -30.0, 0.0), ..Default::default() });
        w.insert(sun, Light::default());
    }

    pub fn open_scene(&mut self, path: &str) {
        self.stop_play();
        let result = self.assets.read_string(path).map_err(|e| format!("{path}: {e}")).and_then(|t| Scene::from_json(&self.registry, &t));
        match result {
            Ok((scene, warnings)) => {
                self.scene = scene;
                self.scene_path = Some(path.to_string());
                self.selection = None;
                self.undo = Undo::default();
                self.dirty = false;
                for w in warnings {
                    self.console.warn(w);
                }
                self.status = crate::i18n::tf("Opened {}", &[path]);
            }
            Err(e) => {
                self.console.error(e);
                if self.scene_path.is_none() {
                    self.new_scene();
                }
            }
        }
    }

    pub fn save_scene(&mut self, save_as: bool) {
        self.save_paint();
        if self.is_playing() {
            self.status = crate::i18n::t("Stop the game before saving").into();
            return;
        }
        let rel = match (&self.scene_path, save_as) {
            (Some(p), false) => p.clone(),
            _ => {
                let dir = self.project.assets_dir().join("scenes");
                let _ = std::fs::create_dir_all(&dir);
                let Some(file) =
                    rfd::FileDialog::new().set_directory(&dir).add_filter(crate::i18n::t("Scene"), &["json"]).set_file_name("scene.scene.json").save_file()
                else {
                    return;
                };
                match file.strip_prefix(self.project.assets_dir()) {
                    Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
                    Err(_) => {
                        self.console.error(crate::i18n::t("Scenes must be saved inside the project's assets folder"));
                        return;
                    }
                }
            }
        };
        match self.scene.save(&self.registry, &self.project.assets_dir().join(&rel)) {
            Ok(()) => {
                self.scene_path = Some(rel.clone());
                self.dirty = false;
                self.status = crate::i18n::tf("Saved {}", &[&rel]);
                self.reload_assets();
            }
            Err(e) => self.console.error(format!("save failed: {e}")),
        }
    }

    // ---------- undo / redo ----------

    /// Remembers the scene before an edit that is about to happen.
    pub fn begin_edit(&mut self) {
        if self.is_playing() || self.undo.pending.is_some() {
            return;
        }
        self.undo.pending = Some(self.scene.to_json(&self.registry));
    }

    /// Commits the remembered state as an undo step if the scene actually changed.
    fn finish_edit(&mut self) {
        let Some(before) = self.undo.pending.take() else { return };
        if self.scene.to_json(&self.registry) != before {
            self.undo.undo.push(before);
            if self.undo.undo.len() > 100 {
                self.undo.undo.remove(0);
            }
            self.undo.redo.clear();
            self.dirty = true;
        }
    }

    fn restore(&mut self, json: &str) {
        let selected = self.selection.and_then(|e| self.position_of(e));
        if let Ok((scene, _)) = Scene::from_json(&self.registry, json) {
            self.scene = scene;
            self.selection = selected.and_then(|p| self.entity_at(p));
            self.dirty = true;
        }
    }

    pub fn undo(&mut self) {
        if self.is_playing() {
            return;
        }
        self.finish_edit();
        if let Some(prev) = self.undo.undo.pop() {
            self.undo.redo.push(self.scene.to_json(&self.registry));
            self.restore(&prev);
        }
    }

    pub fn redo(&mut self) {
        if self.is_playing() {
            return;
        }
        self.finish_edit();
        if let Some(next) = self.undo.redo.pop() {
            self.undo.undo.push(self.scene.to_json(&self.registry));
            self.restore(&next);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.undo.redo.is_empty()
    }

    // ---------- play mode ----------

    fn load_runtime_host(&mut self) -> Option<ScriptHost> {
        self.scripts.host.as_ref()?;
        match ScriptHost::load(&lib_path(&self.project)) {
            Ok(h) => Some(h),
            Err(e) => {
                self.console.error(e);
                None
            }
        }
    }

    pub fn start_play(&mut self) {
        if self.is_playing() {
            return;
        }
        self.finish_edit();
        let json = self.scene.to_json(&self.registry);
        match Scene::from_json(&self.registry, &json) {
            Ok((scene, warnings)) => {
                let selected = self.selection.and_then(|e| self.position_of(e));
                let host = self.load_runtime_host();
                let mut runtime = match Runtime::with_config(scene, host, self.project.config.timing) {
                    Ok(rt) => rt,
                    Err(e) => { self.console.error(e); return; }
                };
                if let Err(e) = runtime.set_time_scale(self.project.config.time_scale) { self.console.error(e); return; }
                runtime.scene.audio.master_volume = self.project.config.master_volume;
                runtime.set_bindings(&self.project.config.binding_defs());
                runtime.log.extend(warnings);
                runtime.input.set_storage(kvas_engine::storage::Storage::new(&self.project.root.join(".kvas/data")));
                runtime.input.set_scene_path(self.scene_path.as_deref().unwrap_or(""));
                self.play = Some(Play { runtime, paused: false, step: false, mouse: shell::MouseCapture::default() });
                self.cursor_hidden = true;
                self.selection = selected.and_then(|p| self.entity_at(p));
            }
            Err(e) => self.console.error(e),
        }
    }

    /// Builds the game (assets packed, player copied, icon embedded) on a worker thread.
    pub fn start_game_build(&mut self) {
        if self.build.is_some() {
            return;
        }
        self.stop_play();
        self.save_paint();
        let (project, log) = (self.project.clone(), self.console.sender());
        let (tx, rx) = std::sync::mpsc::channel();
        self.build = Some(rx);
        self.console.info(crate::i18n::t("Building game..."));
        std::thread::spawn(move || {
            let _ = tx.send(crate::export::build_game(&project, &log));
        });
    }

    /// Plays a clip once from the project panel.
    pub fn preview_sound(&mut self, path: &str) {
        if self.audio.is_none() {
            self.audio = kvas_audio::AudioEngine::new();
        }
        if let Some(a) = &mut self.audio {
            a.stop_all();
            self.scene.audio.play(path, 1.0);
            a.update_requests(&mut self.scene, &self.assets);
        }
    }

    pub fn stop_play(&mut self) {
        if let Some(play) = self.play.take() {
            let name = self.selection.and_then(|e| play.runtime.scene.world.get::<Name>(e)).map(|n| n.0.clone());
            drop(play);
            if let Some(a) = &mut self.audio {
                a.stop_all();
            }
            self.selection = name.and_then(|n| hierarchy::find_by_name(&self.scene.world, &n));
        }
    }

    // ---------- per frame ----------

    /// Non-UI work: threads, game ticking, hot reload, undo bookkeeping.
    pub fn update(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.fps = self.fps * 0.9 + (1.0 / dt.max(1e-4)) * 0.1;

        self.console.pump();
        let log = self.console.sender();
        let scene = &self.scene;
        self.scripts.poll(&self.project, &log, || {
            scene.world.iter::<kvas_engine::components::Scripts>().flat_map(|(_, s)| s.0.iter().map(|slot| slot.name.clone())).collect()
        });
        if self.scripts.is_building() || self.build.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if std::mem::take(&mut self.scripts.reloaded) && self.is_playing() {
            let host = self.load_runtime_host();
            if let Some(p) = &mut self.play {
                p.runtime.scripts.set_host(host);
                self.console.info("Scripts reloaded in the running game (script state was reset)");
            }
        }
        if let Some(rx) = &self.build {
            if let Ok(result) = rx.try_recv() {
                self.build = None;
                match result {
                    Ok(dir) => {
                        self.console.info(format!("Build finished: {}", dir.display()));
                        self.status = crate::i18n::tf("Game built: {}", &[&dir.display().to_string()]);
                        crate::assets_panel::open_external(&dir);
                    }
                    Err(e) => self.console.error(format!("Build failed: {e}")),
                }
            }
        }

        // The game hid the cursor; give it back once play stops.
        if self.play.is_none() && self.cursor_hidden {
            self.cursor_hidden = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
        }
        let typing = ctx.egui_wants_keyboard_input();
        match &mut self.play {
            Some(p) => {
                if !typing {
                    ctx.input(|i| shell::feed_input(&mut p.runtime.input, i, self.play_center));
                } else {
                    p.runtime.input.reset();
                }
                p.runtime.set_paused(p.paused);
                if p.step {
                    p.runtime.step_once();
                    p.step = false;
                } else {
                    p.runtime.tick(dt);
                }
                let assets = self.assets.clone();
                match p.runtime.apply_scene_request(&self.registry, |path| assets.read_string(path).map_err(|e| e.to_string())) {
                    Some(Ok(path)) => self.console.info(format!("{}: {path}", crate::i18n::t("Scene loaded"))),
                    Some(Err(e)) => self.console.error(e),
                    None => {}
                }
                if self.audio.is_none() {
                    self.audio = kvas_audio::AudioEngine::new();
                }
                if let Some(a) = &mut self.audio {
                    if !p.paused {
                        a.update(&mut p.runtime.scene, &self.assets);
                    }
                    for w in a.warnings.drain(..) {
                        self.console.log(Level::Warn, w);
                    }
                }
                let lines: Vec<String> = p.runtime.log.drain(..).collect();
                for line in lines {
                    // `ctx.warn` / `ctx.error` prefix their text; show those in the matching colour.
                    match line.strip_prefix("warning: ").map(|m| (Level::Warn, m)).or_else(|| line.strip_prefix("error: ").map(|m| (Level::Error, m))) {
                        Some((level, text)) => self.console.log(level, text.to_string()),
                        None => self.console.log(Level::Info, line),
                    }
                }
            }
            None => {
                if let Some(a) = &mut self.audio {
                    a.update_requests(&mut self.scene, &self.assets);
                }
                hierarchy::update_globals(&mut self.scene.world);
                // Procedural objects show their initial state while editing.
                let t = self.started.elapsed().as_secs_f32();
                kvas_engine::procedural::update(&mut self.scene.world, t, 0.0, false);
                // Animated characters play their clip as a preview (IK looks at targets; feet
                // and ragdolls need physics, so they act in Play).
                kvas_engine::skeletal::update(&mut self.scene.world, t, 0.0, false);
            }
        }

        if self.last_refresh.elapsed() > Duration::from_millis(1000) {
            self.renderer.refresh_changed();
            let list = self.assets.list();
            if list != self.asset_list {
                self.asset_list = list;
            }
            self.last_refresh = Instant::now();
        }
        for e in self.renderer.take_errors() {
            self.console.error(e);
        }

        // Undo bookkeeping: snapshot when an interaction starts, commit when it is over.
        let (pressed, down) = ctx.input(|i| (i.pointer.any_pressed(), i.pointer.any_down()));
        if pressed {
            self.begin_edit();
        }
        if !down && !typing {
            self.finish_edit();
        }
    }
}
