use crate::editor::{Editor, Play};
use crate::i18n::{t, td, tf};
use crate::icons::Icon;
use crate::fields::{fields_grid, Lists};
use eframe::egui::{self, Color32};
use kvas_engine::prelude::*;
use kvas_engine::reflect::{apply_props, read_props};
use kvas_engine::scene::Scene;

fn world_of<'a>(play: &'a mut Option<Play>, scene: &'a mut Scene) -> &'a mut World {
    match play {
        Some(p) => &mut p.runtime.scene.world,
        None => &mut scene.world,
    }
}

/// A small caption above a group of controls.
fn group_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(text).size(11.0).family(crate::theme::bold_family()).color(crate::theme::pal().text_dim));
    ui.add_space(2.0);
}

/// Row of preset buttons; the one matching `current` is highlighted. Returns the clicked value.
fn presets(ui: &mut egui::Ui, items: &[(&str, f32)], current: f32) -> Option<f32> {
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
        for (label, value) in items {
            let on = (current - value).abs() < 0.001;
            let text = egui::RichText::new(*label).size(12.0).color(if on { egui::Color32::WHITE } else { crate::theme::pal().text });
            let button = egui::Button::new(text).fill(if on { crate::theme::ACCENT } else { crate::theme::pal().button }).min_size(egui::vec2(0.0, 22.0));
            if ui.add(button).clicked() {
                picked = Some(*value);
            }
        }
    });
    picked
}

/// Camera inspector: projection, lens, focus and cinematic settings in groups, with presets.
fn camera_ui(ui: &mut egui::Ui, c: &mut Camera, id: u32) {
    group_label(ui, &t("PROJECTION"));
    egui::Grid::new(("camera-proj", id)).num_columns(2).spacing([10.0, 6.0]).min_col_width(96.0).show(ui, |ui| {
        ui.label(t("primary"));
        ui.checkbox(&mut c.primary, "");
        ui.end_row();
        ui.label(t("orthographic"));
        ui.checkbox(&mut c.orthographic, "");
        ui.end_row();
        if c.orthographic {
            ui.label(t("ortho_size"));
            ui.add(egui::DragValue::new(&mut c.ortho_size).speed(0.1).range(0.5..=500.0));
            ui.end_row();
        } else {
            ui.label(t("fov"));
            ui.add_enabled(c.focal_length <= 1.0, egui::Slider::new(&mut c.fov, 10.0..=120.0).suffix("°"));
            ui.end_row();
        }
        ui.label(t("near / far"));
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut c.near).speed(0.01).range(0.01..=100.0));
            ui.add(egui::DragValue::new(&mut c.far).speed(1.0).range(1.0..=10000.0));
        });
        ui.end_row();
    });
    if c.orthographic {
        return;
    }
    group_label(ui, &t("LENS"));
    ui.horizontal(|ui| {
        ui.label(t("focal_length"));
        ui.add(egui::DragValue::new(&mut c.focal_length).speed(0.5).range(0.0..=400.0).suffix(" mm"));
        ui.weak(if c.focal_length > 1.0 { format!("= {:.0}° FOV", c.vertical_fov()) } else { t("(0 = use FOV)").to_string() });
    });
    if let Some(v) = presets(ui, &[("14", 14.0), ("24", 24.0), ("35", 35.0), ("50", 50.0), ("85", 85.0), ("135", 135.0), ("200", 200.0)], c.focal_length) {
        c.focal_length = v;
    }
    group_label(ui, &t("FOCUS"));
    ui.checkbox(&mut c.dof, t("Depth of field"));
    ui.add_enabled_ui(c.dof, |ui| {
        egui::Grid::new(("camera-focus", id)).num_columns(2).spacing([10.0, 6.0]).min_col_width(96.0).show(ui, |ui| {
            ui.label(t("aperture"));
            ui.add(egui::Slider::new(&mut c.aperture, 0.95..=22.0).logarithmic(true).prefix("f/"));
            ui.end_row();
            ui.label(t("autofocus"));
            ui.checkbox(&mut c.autofocus, "").on_hover_text(t("Focus on what is in the middle of the screen while the game runs"));
            ui.end_row();
            ui.label(t("focus_distance"));
            ui.add_enabled(!c.autofocus, egui::DragValue::new(&mut c.focus_distance).speed(0.1).range(0.1..=1000.0).suffix(" m"));
            ui.end_row();
            ui.label(t("focus_speed"));
            ui.add_enabled(c.autofocus, egui::Slider::new(&mut c.focus_speed, 0.5..=30.0));
            ui.end_row();
        });
        if let Some(v) = presets(ui, &[("f/1.4", 1.4), ("f/2.8", 2.8), ("f/4", 4.0), ("f/8", 8.0), ("f/16", 16.0)], c.aperture) {
            c.aperture = v;
        }
    });
    group_label(ui, &t("CINEMATIC"));
    egui::Grid::new(("camera-cine", id)).num_columns(2).spacing([10.0, 6.0]).min_col_width(96.0).show(ui, |ui| {
        ui.label(t("exposure_comp"));
        ui.add(egui::Slider::new(&mut c.exposure_comp, -3.0..=3.0).suffix(" EV"));
        ui.end_row();
    });
    ui.horizontal(|ui| {
        ui.label(t("letterbox"));
    });
    if let Some(v) = presets(ui, &[("Off", 0.0), ("16:9", 1.778), ("1.85", 1.85), ("2.39", 2.39), ("4:3", 1.333)], c.letterbox) {
        c.letterbox = v;
    }
    ui.weak(t("Depth of field, bars and exposure show in the Game view."));
}

/// Icon shown in a component's header.
fn component_icon(name: &str) -> Option<Icon> {
    Some(match name {
        "Transform" => Icon::Move3d,
        "MeshRenderer" => Icon::Box,
        "Shape" => Icon::Shapes,
        "Camera" => Icon::Camera,
        "Light" => Icon::Lightbulb,
        "RigidBody" => Icon::Package,
        "Collider" => Icon::Scan,
        "Joint" => Icon::Link,
        "Scripts" => Icon::FileCode,
        "AudioSource" => Icon::Volume,
        "MeshModifiers" => Icon::Layers,
        "EditableMesh" => Icon::VertexSnap,
        "TexturePaint" => Icon::Brush,
        "Cloth" => Icon::Shirt,
        "Grass" => Icon::Wind,
        "VoxelWater" => Icon::Waves,
        "VoxelSmoke" => Icon::Flame,
        "VoxelClouds" => Icon::Cloud,
        "Tags" => Icon::Tag,
        "UiElement" => Icon::LayoutGrid,
        "NeuralAgent" => Icon::Sparkles,
        "ParticleEmitter" => Icon::Sparkles,
        "Spin" | "Bob" | "Pulse" | "Wobble" | "ColorCycle" | "MoveTween" | "TransformAnimation" => Icon::Sliders,
        _ => Icon::Sliders,
    })
}

fn category(name: &str) -> &'static str {
    match name {
        "MeshRenderer" | "Shape" | "Camera" | "Light" | "Visibility" => "Rendering",
        "RigidBody" | "Collider" | "Joint" => "Physics",
        "Spin" | "Bob" | "Pulse" | "Wobble" | "ColorCycle" | "MoveTween" | "TransformAnimation" => "Modifiers",
        "MeshModifiers" | "EditableMesh" => "Mesh",
        "Cloth" | "Grass" | "VoxelSmoke" | "VoxelClouds" | "VoxelWater" | "AudioSource" | "ParticleEmitter" => "Effects",
        "Switch" | "Slider" | "Tags" | "Lifetime" | "NeuralAgent" | "UiElement" => "Gameplay",
        _ => "Scripting",
    }
}

impl Editor {
    /// "Add Component" submenus grouped by category. `comps` is `(name, already present)`.
    /// Returns the component the user picked (the caller adds it, because the registry may be borrowed).
    pub fn component_menu(&mut self, ui: &mut egui::Ui, comps: &[(&'static str, bool)]) -> Option<&'static str> {
        let mut picked = None;
        for cat in ["Rendering", "Mesh", "Physics", "Effects", "Gameplay", "Modifiers", "Scripting"] {
            ui.menu_button(t(cat), |ui| {
                for (name, present) in comps.iter().filter(|(n, _)| category(n) == cat) {
                    if ui.add_enabled(!present, egui::Button::new(td(name))).clicked() {
                        picked = Some(*name);
                        ui.close();
                    }
                }
            });
        }
        picked
    }

    pub fn inspector_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| match self.selection.filter(|&e| self.active().world.is_alive(e)) {
            Some(e) => self.entity_inspector(ui, e),
            None => self.scene_settings_ui(ui),
        });
        self.inspector_drop(ui, rect);
    }

    /// Dropping a script, shader, texture, model or sound anywhere on the inspector applies it to
    /// the selected object (or, with nothing selected, adds a camera shader to the scene).
    fn inspector_drop(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        use crate::assets_panel::{AssetDrag, ScriptDrag};
        let ctx = ui.ctx().clone();
        let over = ctx.input(|i| i.pointer.hover_pos()).is_some_and(|p| rect.contains(p));
        let script = egui::DragAndDrop::payload::<ScriptDrag>(&ctx);
        let asset = egui::DragAndDrop::payload::<AssetDrag>(&ctx);
        if !over || (script.is_none() && asset.is_none()) {
            return;
        }
        ui.painter().rect_stroke(rect.shrink(2.0), 8.0, egui::Stroke::new(2.0, crate::theme::ACCENT), egui::StrokeKind::Inside);
        let label = match (&script, &asset) {
            (Some(s), _) => format!("{}  {}", t("Add script"), s.0),
            (_, Some(a)) => format!("{}  {}", t("Apply"), a.0.rsplit('/').next().unwrap_or(&a.0)),
            _ => String::new(),
        };
        let pill = egui::Align2::CENTER_BOTTOM.anchor_size(rect.center_bottom() - egui::vec2(0.0, 14.0), egui::vec2(260.0, 28.0));
        ui.painter().rect_filled(pill, 14.0, crate::theme::ACCENT);
        ui.painter().text(pill.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(12.5), egui::Color32::WHITE);
        if !ctx.input(|i| i.pointer.any_released()) {
            return;
        }
        let selected = self.selection.filter(|&e| self.active().world.is_alive(e));
        if let (Some(s), Some(e)) = (&script, selected) {
            self.attach_script(e, &s.0);
        } else if let Some(a) = &asset {
            self.drop_asset(selected, &a.0);
        }
        egui::DragAndDrop::clear_payload(&ctx);
    }

    /// Applies a project asset to an object (or to the scene when nothing is selected).
    pub fn drop_asset(&mut self, target: Option<Entity>, path: &str) {
        let lower = path.to_ascii_lowercase();
        if self.is_camera_effect(path) {
            self.active_mut().settings.post.effects.push(PostEffect { shader: path.to_string(), ..Default::default() });
            self.status = tf("Camera shader added: {}", &[path]);
            return;
        }
        let Some(e) = target else {
            self.status = t("Select an object first").into();
            return;
        };
        if lower.ends_with(".wgsl") || lower.ends_with(".kmat") || lower.ends_with(".kshader") || [".png", ".jpg", ".jpeg"].iter().any(|x| lower.ends_with(x)) {
            self.apply_asset_to(e, path);
        } else if ["obj", "gltf", "glb", "fbx"].iter().any(|x| lower.ends_with(x)) {
            self.begin_edit();
            let world = &mut self.active_mut().world;
            if !world.has::<MeshRenderer>(e) {
                world.insert(e, MeshRenderer::default());
            }
            world.remove::<Shape>(e);
            if let Some(m) = world.get_mut::<MeshRenderer>(e) {
                m.mesh = path.to_string();
            }
            self.status = tf("Model set: {}", &[path]);
        } else if ["wav", "ogg", "mp3", "flac", "m4a"].iter().any(|x| lower.ends_with(x)) {
            self.begin_edit();
            self.active_mut().world.insert(e, AudioSource { clip: path.to_string(), ..Default::default() });
            self.status = tf("Attached {}", &[path]);
        }
    }

    /// Preview and "open in the node editor" for a node material on the selected object.
    fn material_preview(&mut self, ui: &mut egui::Ui, e: Entity) {
        let shader = self.active().world.get::<MeshRenderer>(e).map(|m| m.shader.clone()).unwrap_or_default();
        if !(shader.ends_with(".kmat") || shader.ends_with(".kshader")) {
            return;
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(84.0, 84.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 10.0, crate::theme::pal().field);
            if let Some(tex) = self.renderer.thumbnail("primitive:sphere", &shader, 128) {
                ui.painter().image(tex, rect.shrink(3.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
            }
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(shader.rsplit('/').next().unwrap_or(&shader)).family(crate::theme::bold_family()));
                if ui.button(t("Open in node editor")).clicked() {
                    self.open_material(&shader);
                }
                if ui.button(t("Remove material")).clicked() {
                    if let Some(m) = self.active_mut().world.get_mut::<MeshRenderer>(e) {
                        m.shader.clear();
                    }
                }
            });
        });
    }

    /// A WGSL file that defines `fn effect(` is a camera shader, not a mesh shader.
    pub fn is_camera_effect(&self, path: &str) -> bool {
        let lower = path.to_ascii_lowercase();
        lower.ends_with(".kfx") || (lower.ends_with(".wgsl") && self.assets.read_string(path).is_ok_and(|s| s.contains("fn effect(")))
    }

    pub fn scene_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading(t("Scene Settings"));
        ui.weak(t("Select an object to edit it."));
        ui.separator();
        let shaders: Vec<String> = self.asset_list.iter().filter(|a| a.ends_with(".wgsl") || a.ends_with(".kfx")).cloned().collect();
        let environments: Vec<String> = self.asset_list.iter().filter(|p| [".hdr", ".png", ".jpg", ".jpeg"].iter().any(|ext| p.to_ascii_lowercase().ends_with(ext))).cloned().collect();
        let mut new_fx = false;
        let (dlss_ok, fg_ok, is_rtx, gpu) = (self.renderer.dlss_available(), self.renderer.frame_generation_available(), self.renderer.is_rtx(), self.renderer.gpu_name().to_string());
        let s = &mut self.active_mut().settings;
        egui::Grid::new("scene-settings").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(t("ambient"));
            ui.color_edit_button_rgb(&mut s.ambient);
            ui.end_row();
            ui.label(t("sky color"));
            ui.color_edit_button_rgb(&mut s.sky);
            ui.end_row();
            ui.label(t("gravity"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut s.gravity.y).speed(0.1));
            });
            ui.end_row();
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t("Look")).family(crate::theme::bold_family()));
        ui.horizontal_wrapped(|ui| {
            for (name, bands, sat, con, exp) in [
                ("Realistic", 0, 1.0, 1.0, 1.0),
                ("Semi-real", 0, 1.1, 1.08, 1.0),
                ("Anime", 3, 1.2, 1.05, 1.05),
                ("Cartoon", 4, 1.45, 1.12, 1.1),
                ("Noir", 3, 0.0, 1.3, 1.0),
            ] {
                if ui.button(t(name)).clicked() {
                    (s.toon_bands, s.saturation, s.contrast, s.exposure) = (bands, sat, con, exp);
                }
            }
        });
        egui::Grid::new("scene-look").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(t("toon bands"));
            let mut bands = s.toon_bands as i32;
            if ui.add(egui::DragValue::new(&mut bands).range(0..=8)).changed() {
                s.toon_bands = bands.max(0) as u32;
            }
            ui.end_row();
            ui.label(t("saturation"));
            ui.add(egui::DragValue::new(&mut s.saturation).speed(0.01).range(0.0..=3.0));
            ui.end_row();
            ui.label(t("contrast"));
            ui.add(egui::DragValue::new(&mut s.contrast).speed(0.01).range(0.3..=2.5));
            ui.end_row();
            ui.label(t("exposure"));
            ui.add(egui::DragValue::new(&mut s.exposure).speed(0.01).range(0.1..=4.0));
            ui.end_row();
            ui.label(t("PSX look"));
            ui.checkbox(&mut s.psx, t("snapped vertices, 15-bit dither, point texels"));
            ui.end_row();
            ui.label(t("render scale"));
            ui.add(egui::Slider::new(&mut s.render_scale, 0.2..=1.0));
            ui.end_row();
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t("Sky")).family(crate::theme::bold_family()));
        let a = &mut s.atmosphere;
        ui.checkbox(&mut a.procedural, t("Procedural sky (also lights the scene)"));
        ui.add_enabled_ui(a.procedural, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (name, hour) in [("Dawn", 6.0), ("Noon", 12.0), ("Sunset", 18.0), ("Midnight", 0.0)] {
                    if ui.button(t(name)).clicked() {
                        a.time = hour;
                    }
                }
            });
            egui::Grid::new("scene-sky").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(t("time of day"));
                ui.add(
                    egui::Slider::new(&mut a.time, 0.0..=24.0)
                        .custom_formatter(|h, _| format!("{:02}:{:02}", h as i32 % 24, ((h % 1.0) * 60.0) as i32)),
                );
                ui.end_row();
                ui.label(t("day speed"));
                ui.add(egui::DragValue::new(&mut a.speed).speed(0.01).range(0.0..=24.0)).on_hover_text(t("Game hours per real second"));
                ui.end_row();
                ui.label(t("clouds"));
                ui.add(egui::Slider::new(&mut a.clouds, 0.0..=1.0));
                ui.end_row();
                ui.label(t("cloud speed"));
                ui.add(egui::DragValue::new(&mut a.cloud_speed).speed(0.002).range(0.0..=1.0));
                ui.end_row();
                ui.label(t("stars"));
                ui.add(egui::Slider::new(&mut a.stars, 0.0..=1.0));
                ui.end_row();
                ui.label(t("haze"));
                ui.add(egui::Slider::new(&mut a.haze, 0.0..=1.0));
                ui.end_row();
                ui.label(t("milky way"));
                ui.add(egui::Slider::new(&mut a.milky_way, 0.0..=1.0));
                ui.end_row();
                ui.label(t("moon phase"));
                ui.add(egui::Slider::new(&mut a.moon_phase, 0.0..=1.0)).on_hover_text(t("0 new, 0.5 full"));
                ui.end_row();
                ui.label(t("aurora"));
                ui.add(egui::Slider::new(&mut a.aurora, 0.0..=1.0));
                ui.end_row();
            });
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t("Performance")).family(crate::theme::bold_family()));
        ui.collapsing("Окружение HDRI / панорама", |ui| {
            egui::ComboBox::from_id_salt("environment-map").selected_text(if s.environment_map.is_empty() { "Процедурное небо / цвет" } else { &s.environment_map }).show_ui(ui, |ui| {
                ui.selectable_value(&mut s.environment_map, String::new(), "Процедурное небо / цвет");
                for path in &environments { ui.selectable_value(&mut s.environment_map, path.clone(), path); }
            });
            ui.add(egui::Slider::new(&mut s.environment_rotation, -180.0..=180.0).text("Поворот"));
            ui.add(egui::Slider::new(&mut s.environment_intensity, 0.0..=8.0).text("Интенсивность"));
            ui.weak("Панорамы 2:1: Radiance HDR (линейный), PNG/JPEG (sRGB). Фон, зеркальные отражения и усреднённый рассеянный свет. До 2048×1024; EXR и prefiltered IBL пока не поддержаны.");
        });
        ui.horizontal_wrapped(|ui| {
            for (label, scale) in [("Native", 1.0), ("Quality", 0.77), ("Balanced", 0.67), ("Performance", 0.5), ("Ultra", 0.33)] {
                if ui.selectable_label((s.render_scale - scale).abs() < 0.01, t(label)).clicked() {
                    s.render_scale = scale;
                }
            }
        });
        egui::Grid::new("scene-perf").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(t("upscaler"));
            let name = |u: Upscaler| match u {
                Upscaler::Fsr => "FSR",
                Upscaler::Dlss => "DLSS",
                Upscaler::Bilinear => "Bilinear",
            };
            egui::ComboBox::from_id_salt("upscaler").selected_text(t(name(s.upscaler))).show_ui(ui, |ui| {
                for u in [Upscaler::Fsr, Upscaler::Dlss, Upscaler::Bilinear] {
                    ui.selectable_value(&mut s.upscaler, u, t(name(u)));
                }
            });
            ui.end_row();
            ui.label(t("frame generation"));
            ui.checkbox(&mut s.frame_generation, t("Enable"));
            ui.end_row();
            ui.label(t("view distance"));
            ui.add(egui::DragValue::new(&mut s.view_distance).speed(1.0).range(0.0..=2000.0).suffix(" m")).on_hover_text(t("Objects farther away are not loaded or drawn (0 = unlimited)"));
            ui.end_row();
            ui.label(t("filmic tone"));
            ui.add(egui::Slider::new(&mut s.filmic, 0.0..=1.0)).on_hover_text(t("Soft highlight roll-off like film"));
            ui.end_row();
            ui.label(t("fog"));
            ui.add(egui::Slider::new(&mut s.fog, 0.0..=0.03).custom_formatter(|v, _| format!("{:.1}", v * 1000.0)).custom_parser(|t| t.parse::<f64>().ok().map(|v| v / 1000.0))).on_hover_text(t("Distance haze that takes the sky colour"));
            ui.end_row();
            ui.label(t("sharpness"));
            ui.add(egui::Slider::new(&mut s.sharpness, 0.0..=1.0));
            ui.end_row();
        });
        // Say plainly what actually runs.
        let mut notes = Vec::new();
        if s.upscaler == Upscaler::Dlss && !dlss_ok {
            notes.push(if is_rtx { "DLSS: нативный бэкенд Streamline ещё не подключён; используется собственный пространственный апскейлер." } else { t("DLSS needs an NVIDIA RTX card; the FSR-style upscaler runs instead.") });
        }
        if s.frame_generation && !fg_ok {
            notes.push(t("Frame generation is saved in the scene but cannot run yet (needs NVIDIA Streamline / FSR 3 integration)."));
        }
        if s.frame_generation && fg_ok {
            notes.push("Экспериментальная интерполяция optical flow ×2 в Game. Добавляет задержку; возможны артефакты на быстром движении. Это не NVIDIA DLSS FG.");
        }
        for n in notes {
            ui.label(egui::RichText::new(n).size(11.0).color(crate::theme::pal().text_dim));
        }
        ui.weak(format!("GPU: {}", gpu));
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t("Shadows")).family(crate::theme::bold_family()));
        let sh = &mut s.shadows;
        ui.checkbox(&mut sh.enabled, t("Sun shadows"));
        ui.add_enabled_ui(sh.enabled, |ui| {
            egui::Grid::new("scene-shadows").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(t("softness"));
                ui.add(egui::Slider::new(&mut sh.softness, 0.25..=6.0)).on_hover_text(t("Edge softness: low is crisp, high is very soft"));
                ui.end_row();
                ui.label(t("strength"));
                ui.add(egui::Slider::new(&mut sh.strength, 0.0..=1.0));
                ui.end_row();
                ui.label(t("bias"));
                ui.add(egui::DragValue::new(&mut sh.bias).speed(0.00005).range(0.0..=0.01).max_decimals(5)).on_hover_text(t("Raise it if surfaces show stripes"));
                ui.end_row();
                ui.label(t("range"));
                ui.add(egui::DragValue::new(&mut sh.range).speed(0.5).range(5.0..=200.0)).on_hover_text(t("Metres around the camera that receive shadows"));
                ui.end_row();
            });
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t("Post effects")).family(crate::theme::bold_family()));
        ui.horizontal_wrapped(|ui| {
            for (label, key) in [("None", ""), ("Cinematic", "cinematic"), ("Dreamy", "dreamy"), ("Crisp", "crisp"), ("Retro", "retro")] {
                if ui.button(t(label)).clicked() {
                    s.post = PostSettings::preset(key);
                }
            }
        });
        let p = &mut s.post;
        egui::Grid::new("scene-post").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            for (label, value, max) in [
                ("bloom", &mut p.bloom, 2.0),
                ("lens flare", &mut p.lens_flare, 1.5),
                ("blur", &mut p.blur, 1.0),
                ("motion blur", &mut p.motion_blur, 2.0),
                ("vignette", &mut p.vignette, 1.0),
                ("chromatic aberration", &mut p.chromatic_aberration, 1.5),
                ("film grain", &mut p.film_grain, 1.0),
                ("sharpen", &mut p.sharpen, 1.5),
            ] {
                ui.label(t(label));
                ui.add(egui::Slider::new(value, 0.0..=max));
                ui.end_row();
            }
            ui.label(t("bloom threshold"));
            ui.add(egui::Slider::new(&mut p.bloom_threshold, 0.0..=1.5));
            ui.end_row();
            ui.label(t("bloom radius"));
            ui.add(egui::Slider::new(&mut p.bloom_radius, 0.2..=6.0));
            ui.end_row();
            ui.label(t("tint"));
            ui.color_edit_button_rgb(&mut p.tint);
            ui.end_row();
        });
        // Custom camera shaders: WGSL files with `fn effect(uv, color, params, time)`.
        ui.add_space(6.0);
        ui.label(egui::RichText::new(t("Camera shaders")).family(crate::theme::bold_family()));
        let mut remove_fx: Option<usize> = None;
        for (n, fx) in p.effects.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut fx.enabled, "");
                egui::ComboBox::from_id_salt(("camera-fx", n)).selected_text(if fx.shader.is_empty() { t("(choose a shader)").to_string() } else { fx.shader.clone() }).width(170.0).show_ui(ui, |ui| {
                    for sh in &shaders {
                        ui.selectable_value(&mut fx.shader, sh.clone(), sh);
                    }
                });
                if crate::icons::button(ui, crate::icons::Icon::X, t("Remove"), false).clicked() {
                    remove_fx = Some(n);
                }
            });
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                for v in fx.params.iter_mut() {
                    ui.add(egui::DragValue::new(v).speed(0.01));
                }
            });
        }
        if let Some(n) = remove_fx {
            p.effects.remove(n);
        }
        ui.horizontal(|ui| {
            if ui.button(t("+ Add shader")).clicked() {
                p.effects.push(PostEffect::default());
            }
            if ui.button(t("New shader file")).on_hover_text(t("Creates shaders/fx/effect.wgsl from a template and adds it")).clicked() {
                new_fx = true;
            }
        });
        ui.separator();
        ui.label(format!("{} {}", self.active().world.entities().len(), t("objects")));
        if let Some(path) = &self.scene_path {
            ui.weak(path);
        }
        if new_fx {
            self.create_post_effect_file();
        }
    }

    /// Writes a starter camera shader into the project and adds it to the scene's effect list.
    fn create_post_effect_file(&mut self) {
        let dir = self.project.assets_dir().join("shaders/fx");
        let _ = std::fs::create_dir_all(&dir);
        let mut n = 1;
        let mut path = dir.join("effect.wgsl");
        while path.exists() {
            n += 1;
            path = dir.join(format!("effect_{n}.wgsl"));
        }
        match std::fs::write(&path, crate::script_templates::POST_EFFECT_TEMPLATE) {
            Ok(()) => {
                self.reload_assets();
                let rel = format!("shaders/fx/{}", path.file_name().unwrap_or_default().to_string_lossy());
                self.active_mut().settings.post.effects.push(PostEffect { shader: rel, ..Default::default() });
                self.prefs.open(&path);
            }
            Err(e) => self.console.error(e.to_string()),
        }
    }

    fn entity_inspector(&mut self, ui: &mut egui::Ui, e: Entity) {
        let assets = self.asset_list.clone();
        let names = self.entity_names();
        let lists = Lists { assets: &assets, entities: &names };
        let registry = std::mem::replace(&mut self.registry, kvas_engine::registry::Registry { components: Vec::new() });

        // Name.
        let mut name = hierarchy::name_of(&self.active().world, e).to_string();
        ui.horizontal(|ui| {
            ui.label(t("Name"));
            if ui.add(egui::TextEdit::singleline(&mut name).desired_width(f32::INFINITY)).changed() {
                self.active_mut().world.insert(e, Name(name.clone()));
            }
        });
        ui.separator();

        let mut remove: Option<&'static str> = None;
        let mut tags = self.active().world.get::<Tags>(e).cloned().unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("Теги");
            if ui.add(egui::TextEdit::singleline(&mut tags.value).hint_text("enemy damageable").desired_width(f32::INFINITY)).on_hover_text("Через пробел; ctx.tagged(\"enemy\"), ctx.has_tag(entity, \"enemy\")").changed() { self.active_mut().world.insert(e, tags); }
        });
        let count = self.selected_entities().len();
        if count > 1 { ui.weak(format!("Выделено: {count}. Инспектор — активный объект; gizmo и Ctrl+D/Delete — группа.")); }
        let mut reset: Option<&'static str> = None;
        for info in &registry.components {
            if matches!(info.name, "Name" | "Tags") || !(info.has)(&self.active().world, e) {
                continue;
            }
            let header = crate::ui_kit::section(ui, ui.make_persistent_id(("component", info.name)), component_icon(info.name), &td(info.name), true, |ui| {
                if info.name == "Scripts" {
                    self.scripts_ui(ui, e, &lists);
                } else if info.name == "MeshModifiers" {
                    self.modifiers_ui(ui, e, &lists);
                } else if info.name == "TransformAnimation" {
                    self.animation_clip_ui(ui, e);
                    if ui.button("Открыть таймлайн").clicked() { self.animation_open = true; }
                } else if info.name == "Camera" {
                    let world = world_of(&mut self.play, &mut self.scene);
                    if let Some(camera) = world.get_mut::<Camera>(e) {
                        camera_ui(ui, camera, e.index());
                    }
                } else {
                    let id = format!("{}-{}", info.name, e.index());
                    let world = world_of(&mut self.play, &mut self.scene);
                    (info.edit)(world, e, &mut |mut fields| {
                        fields_grid(ui, &id, &mut fields, &lists);
                    });
                    if info.name == "MeshRenderer" {
                        self.material_preview(ui, e);
                        if ui.button("Редактировать геометрию / Sculpt").clicked() { self.start_mesh_edit(); }
                    }
                    if info.name == "NeuralAgent" {
                        if ui.button("Обучение AI / веса").clicked() { self.training.open = true; }
                        ui.weak("CPU steering: задайте target, обучите/примените веса и включите enabled. Без навигации и RigidBody.");
                    }
                }
            });
            header.context_menu(|ui| {
                if ui.button(t("Reset to defaults")).clicked() {
                    reset = Some(info.name);
                    ui.close();
                }
                if info.removable && ui.button(t("Remove component")).clicked() {
                    remove = Some(info.name);
                    ui.close();
                }
            });
            ui.add_space(6.0);
        }

        ui.separator();
        let comps: Vec<(&'static str, bool)> = registry
            .components
            .iter()
            .filter(|c| !matches!(c.name, "Name" | "Transform"))
            .map(|c| (c.name, (c.has)(&self.active().world, e)))
            .collect();
        let mut add_new = None;
        ui.menu_button(t("Add Component"), |ui| add_new = self.component_menu(ui, &comps));
        ui.weak(t("Right-click a component header for more."));
        self.registry = registry;
        if let Some(name) = add_new {
            self.add_component(e, name);
        }
        if let Some(name) = remove {
            self.remove_component(e, name);
        }
        if let Some(name) = reset {
            self.add_component(e, name);
        }
    }

    /// The Blender-style modifier stack: reorderable, individually switchable.
    fn modifiers_ui(&mut self, ui: &mut egui::Ui, e: Entity, lists: &Lists) {
        use kvas_engine::meshmod::Modifier;
        let world = world_of(&mut self.play, &mut self.scene);
        let mut remove: Option<usize> = None;
        let mut swap: Option<(usize, usize)> = None;
        let mut add: Option<&'static str> = None;
        if let Some(stack) = world.get_mut::<MeshModifiers>(e) {
            let count = stack.0.len();
            for (i, m) in stack.0.iter_mut().enumerate() {
                let title = td(m.name());
                let id = format!("modifier-{}-{i}", e.index());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{}. {title}", i + 1)).family(crate::theme::bold_family()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if crate::icons::button(ui, crate::icons::Icon::X, t("Remove"), false).clicked() {
                            remove = Some(i);
                        }
                        if i + 1 < count && crate::icons::button(ui, crate::icons::Icon::ChevronDown, t("Move down"), false).clicked() {
                            swap = Some((i, i + 1));
                        }
                        if i > 0 && crate::icons::button(ui, crate::icons::Icon::ChevronUp, t("Move up"), false).clicked() {
                            swap = Some((i, i - 1));
                        }
                    });
                });
                let mut fields = m.fields();
                fields_grid(ui, &id, &mut fields, lists);
                ui.separator();
            }
            if count == 0 {
                ui.weak(t("No modifiers yet. They reshape the mesh without changing the model."));
            }
        }
        ui.menu_button(t("Add Modifier"), |ui| {
            for name in Modifier::NAMES {
                if ui.button(td(name)).clicked() {
                    add = Some(name);
                    ui.close();
                }
            }
        });
        if let Some(stack) = world.get_mut::<MeshModifiers>(e) {
            if let Some((a, b)) = swap {
                stack.0.swap(a, b);
            }
            if let Some(i) = remove {
                stack.0.remove(i);
            }
            if let Some(m) = add.and_then(Modifier::named) {
                stack.0.push(m);
            }
        }
    }

    fn scripts_ui(&mut self, ui: &mut egui::Ui, e: Entity, lists: &Lists) {
        let available = self.scripts.names();
        let mut remove: Option<usize> = None;
        let mut add: Option<String> = None;
        let world = world_of(&mut self.play, &mut self.scene);
        let host = self.scripts.host.as_ref();
        if let Some(scripts) = world.get_mut::<Scripts>(e) {
            for (i, slot) in scripts.0.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(("script", e.index(), i)).selected_text(&slot.name).show_ui(ui, |ui| {
                        for n in &available {
                            if ui.selectable_label(slot.name == *n, n).clicked() {
                                slot.name = n.clone();
                                slot.props.clear();
                            }
                        }
                    });
                    if ui.small_button(t("Remove")).clicked() {
                        remove = Some(i);
                    }
                });
                match host.and_then(|h| h.create(&slot.name)) {
                    Some(mut instance) => {
                        let mut fields = instance.fields();
                        apply_props(&mut fields, &slot.props);
                        if fields_grid(ui, &format!("script-{}-{i}", e.index()), &mut fields, lists) {
                            slot.props = read_props(&fields);
                        }
                    }
                    None => {
                        ui.colored_label(Color32::from_rgb(230, 160, 80), tf("'{}' is not in the built scripts library", &[&slot.name]));
                    }
                }
                ui.separator();
            }
        }
        ui.menu_button(t("Add Script"), |ui| {
            if available.is_empty() {
                ui.label(t("No scripts built yet (see the Console)"));
            }
            for n in &available {
                if ui.button(n).clicked() {
                    add = Some(n.clone());
                    ui.close();
                }
            }
        });
        if let Some(i) = remove {
            if let Some(s) = world.get_mut::<Scripts>(e) {
                s.0.remove(i);
            }
        }
        if let Some(name) = add {
            if let Some(s) = world.get_mut::<Scripts>(e) {
                s.0.push(ScriptSlot { name, ..Default::default() });
            }
        }
    }
}
