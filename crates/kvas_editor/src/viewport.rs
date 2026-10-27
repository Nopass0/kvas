//! The Scene/Game view: editor camera, picking, overlays and the render image.

use crate::assets_panel::AssetDrag;
use crate::editor::{Editor, Tool};
use crate::icons::{self, Icon};
use eframe::egui::{self, Color32, PointerButton, Sense};
use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};
use kvas_engine::prelude::*;
use kvas_render::debug::DebugDraw;
use kvas_render::{game_view, Frame, View};

pub struct EditorCamera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    /// Distance to the point Alt+drag orbits around.
    pub pivot_dist: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        let pos = Vec3::new(9.0, 7.0, 12.0);
        let f = (-pos).normalize();
        Self { pos, yaw: (-f.x).atan2(-f.z), pitch: f.y.asin(), pivot_dist: pos.length() }
    }
}

impl EditorCamera {
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }
    pub fn forward(&self) -> Vec3 {
        self.rotation() * Vec3::NEG_Z
    }
    pub fn view(&self, aspect: f32) -> View {
        View::new(Mat4::from_rotation_translation(self.rotation(), self.pos), 60.0, aspect, 0.05, 1000.0)
    }
    pub fn focus(&mut self, center: Vec3, radius: f32) {
        self.pivot_dist = (radius * 2.5).max(2.0);
        self.pos = center - self.forward() * self.pivot_dist;
    }
}

fn ray_box(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    Aabb::new(min, max)?.ray_distance(Ray::new(origin, dir)?, f32::MAX)
}

fn ray_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    Ray::new(origin, dir)?.sphere_distance(center, radius)
}

pub fn to_screen(view: &View, rect: egui::Rect, p: Vec3) -> Option<egui::Pos2> {
    let ndc = view.project(p)?;
    Some(egui::pos2(rect.min.x + (ndc.x * 0.5 + 0.5) * rect.width(), rect.min.y + (0.5 - ndc.y * 0.5) * rect.height()))
}

pub fn to_ndc(rect: egui::Rect, p: egui::Pos2) -> Vec2 {
    Vec2::new((p.x - rect.min.x) / rect.width() * 2.0 - 1.0, 1.0 - (p.y - rect.min.y) / rect.height() * 2.0)
}

const SELECT_COLOR: [f32; 4] = [1.0, 0.65, 0.1, 1.0];

impl Editor {
    pub fn viewport_ui(&mut self, ui: &mut egui::Ui) {
        let (rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        if rect.width() < 16.0 || rect.height() < 16.0 {
            return;
        }
        let playing = self.is_playing();
        if !playing {
            self.camera_controls(ui, &resp);
        }

        let aspect = rect.width() / rect.height();
        let view = if playing { game_view(self.active(), aspect) } else { Some(self.camera.view(aspect)) };
        let Some(view) = view else {
            self.last_view = None;
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(25));
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, crate::i18n::t("No Camera in the scene"), egui::FontId::proportional(16.0), Color32::GRAY);
            return;
        };

        let mut draw = DebugDraw::default();
        self.build_debug(&view, &mut draw, playing);
        let painting = !playing && self.paint.active && self.has_paint_target();
        let gizmo_busy = if playing || painting { false } else { self.gizmo_ui(ui, &resp, rect, &view, &mut draw) };
        if painting {
            self.paint_in_viewport(ui, &resp, rect, &view, &mut draw);
        }

        if !playing && !gizmo_busy && !painting && resp.clicked_by(PointerButton::Primary) && !ui.input(|i| i.modifiers.alt) {
            if let Some(pos) = resp.interact_pointer_pos() {
                let picked = self.pick(&view, to_ndc(rect, pos));
                self.select_entity(picked, ui.input(|i| i.modifiers), &[]);
            }
        }
        // While something is dragged over the scene, its edge pulses so the drop target is obvious.
        if resp.contains_pointer() && (egui::DragAndDrop::has_payload_of_type::<AssetDrag>(ui.ctx()) || egui::DragAndDrop::has_payload_of_type::<crate::assets_panel::ScriptDrag>(ui.ctx())) {
            let pulse = 0.5 + 0.5 * (ui.input(|i| i.time) as f32 * 6.0).sin();
            let a = crate::theme::ACCENT;
            ui.painter().rect_stroke(rect.shrink(2.0), 6.0, egui::Stroke::new(2.0 + pulse * 2.0, egui::Color32::from_rgba_unmultiplied(a.r(), a.g(), a.b(), (120.0 + 110.0 * pulse) as u8)), egui::StrokeKind::Inside);
            ui.ctx().request_repaint();
        }
        if let Some(script) = resp.dnd_release_payload::<crate::assets_panel::ScriptDrag>() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()).filter(|p| rect.contains(*p)) {
                match self.pick(&view, to_ndc(rect, pos)) {
                    Some(e) => self.attach_script(e, &script.0),
                    None => self.status = crate::i18n::t("Drop the script on an object").into(),
                }
            }
        }
        if let Some(asset) = resp.dnd_release_payload::<AssetDrag>().filter(|a| self.is_camera_effect(&a.0)) {
            self.drop_asset(None, &asset.0);
        }
        if let Some(asset) = resp.dnd_release_payload::<AssetDrag>() {
            let target = ui.input(|i| i.pointer.interact_pos()).filter(|p| rect.contains(*p)).and_then(|pos| self.pick(&view, to_ndc(rect, pos)));
            if let Some(e) = target.filter(|_| self.apply_asset_to_ok(&asset.0)) {
                self.apply_asset_to(e, &asset.0);
            } else if let Some(pos) = ui.input(|i| i.pointer.interact_pos()).filter(|p| rect.contains(*p)) {
                let (o, d) = view.ray(to_ndc(rect, pos));
                let at = if d.y < -0.01 { o + d * (-o.y / d.y).min(60.0) } else { o + d * 8.0 };
                self.create_from_asset(&asset.0, at);
            }
        }

        let ppp = ui.ctx().pixels_per_point();
        self.last_view = Some((view, [(rect.width() * ppp) as u32, (rect.height() * ppp) as u32]));
        let frame = Frame { view, size: [(rect.width() * ppp) as u32, (rect.height() * ppp) as u32], time: self.elapsed(), dt: 1.0 / 60.0, debug: &draw, shadows: true, debug_view: self.debug_view, cinematic: playing };
        let scene = match &self.play {
            Some(p) => &p.runtime.scene,
            None => &self.scene,
        };
        let texture = self.renderer.render(scene, &frame);
        ui.painter().image(texture, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        if !playing {
            resp.context_menu(|ui| self.viewport_context_menu(ui));
        }
        self.play_center = self.play.as_ref().map(|_| rect.center());
        if let Some(p) = &self.play {
            kvas_render::shell::draw_hud(ui.painter(), rect, &p.runtime.input.hud());
            kvas_render::shell::apply_cursor(ui.ctx(), &p.runtime.input, rect.center());
        }
        self.viewport_overlay(ui, rect);
        if resp.hovered() && ui.input(|i| i.pointer.any_down()) && playing {
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, crate::theme::ACCENT), egui::StrokeKind::Inside);
        }
    }

    fn viewport_context_menu(&mut self, ui: &mut egui::Ui) {
        if let Some(e) = self.selection.filter(|&e| self.active().world.is_alive(e)) {
            let name = hierarchy::name_of(&self.active().world, e).to_string();
            ui.label(egui::RichText::new(name).family(crate::theme::bold_family()));
            ui.separator();
            if ui.button(crate::i18n::t("Focus (F)")).clicked() {
                self.focus_selection();
                ui.close();
            }
            if ui.button(crate::i18n::t("Duplicate (Ctrl+D)")).clicked() {
                self.duplicate_entity(e);
                ui.close();
            }
            if ui.button(crate::i18n::t("Fit collider to model")).clicked() {
                self.begin_edit();
                if !self.fit_collider(e) {
                    self.status = crate::i18n::t("Select an object with a model first").into();
                }
                ui.close();
            }
            if ui.button(crate::i18n::t("Reset transform")).clicked() {
                self.add_component(e, "Transform");
                ui.close();
            }
            let comps: Vec<(&'static str, bool)> = self.registry.components.iter().filter(|c| !matches!(c.name, "Name" | "Transform")).map(|c| (c.name, (c.has)(&self.active().world, e))).collect();
            let mut picked = None;
            ui.menu_button(crate::i18n::t("Add component"), |ui| picked = self.component_menu(ui, &comps));
            if let Some(name) = picked {
                self.add_component(e, name);
            }
            if ui.button(crate::i18n::t("Delete (Del)")).clicked() {
                self.delete_entity(e);
                ui.close();
            }
            ui.menu_button(crate::i18n::t("Transform"), |ui| self.transform_menu(ui));
            ui.separator();
        }
        ui.menu_button(crate::i18n::t("Create object"), |ui| self.create_menu(ui));
    }

    /// Floating glass toolbar (tools) and stats pill on top of the 3D view.
    fn viewport_overlay(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let playing = self.is_playing();
        let width = if playing { 190.0 } else { 478.0 };
        let bar = egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 10.0), egui::vec2(width, 36.0));
        ui.painter().add(crate::ui_kit::float_panel(bar, 10.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(bar.shrink2(egui::vec2(6.0, 4.0))).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            if playing {
                ui.colored_label(crate::theme::green(), crate::i18n::t("Game view"));
            } else {
                for (tool, icon, label) in [(Tool::Move, Icon::Move3d, crate::i18n::t("Move (W)")), (Tool::Rotate, Icon::Rotate3d, crate::i18n::t("Rotate (E)")), (Tool::Scale, Icon::Scale3d, crate::i18n::t("Scale (R)"))] {
                    if icons::button(ui, icon, label, self.tool == tool).clicked() {
                        self.tool = tool;
                    }
                }
                ui.separator();
                if icons::button(ui, Icon::Cuboid, crate::i18n::t("Local space (off = global)"), self.local_space).clicked() {
                    self.local_space = !self.local_space;
                }
                if icons::button(ui, Icon::Grid3x3, crate::i18n::t("Grid"), self.show_grid).clicked() {
                    self.show_grid = !self.show_grid;
                }
            }
            if icons::button(ui, Icon::Scan, crate::i18n::t("Collider outlines"), self.show_colliders).clicked() {
                self.show_colliders = !self.show_colliders;
            }
            ui.separator();
            let names = ["Lit", "Albedo", "Normals", "Depth", "UV", "Lighting", "Clay", "Material", "Creases"];
            egui::ComboBox::from_id_salt("view-mode").selected_text(crate::i18n::t(names[self.debug_view as usize])).width(72.0).show_ui(ui, |ui| {
                for (i, name) in names.iter().enumerate() {
                    ui.selectable_value(&mut self.debug_view, i as u32, crate::i18n::t(name));
                }
            });
        });
        if !playing {
            self.snap_overlay(ui, bar.left_bottom() + egui::vec2(0.0, 6.0));
            if self.has_paint_target() {
                self.paint_overlay(ui, bar.left_bottom() + egui::vec2(0.0, 46.0));
            }
        }
        let stats = format!("{:.0} {}   {} {}", self.fps, crate::i18n::t("fps"), self.active().world.len(), crate::i18n::t("objects"));
        let galley = ui.painter().layout_no_wrap(stats, egui::FontId::proportional(12.0), crate::theme::pal().text);
        let pill = egui::Rect::from_min_size(egui::pos2(rect.right() - galley.size().x - 34.0, rect.top() + 10.0), egui::vec2(galley.size().x + 24.0, 26.0));
        ui.painter().add(crate::ui_kit::float_panel(pill, 13.0));
        ui.painter().galley(pill.min + egui::vec2(12.0, 5.0), galley, crate::theme::pal().text);
        let render = self.renderer.stats();
        ui.interact(pill, ui.scope_id().with("render-statistics"), Sense::hover()).on_hover_text(format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
            crate::i18n::t("Mesh draws"), render.mesh_draw_calls, crate::i18n::t("Triangles"), render.triangles, crate::i18n::t("Hidden"), render.hidden_objects,
            crate::i18n::t("Frustum culled"), render.culled_objects, crate::i18n::t("Shadow draws"), render.shadow_draw_calls,
            crate::i18n::t("Shadow triangles"), render.shadow_triangles, crate::i18n::t("Debug lines"), render.debug_lines
        ));
    }

    fn camera_controls(&mut self, ui: &mut egui::Ui, resp: &egui::Response) {
        let ctx = ui.ctx().clone();
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let (delta, scroll, alt, shift, dt) = ctx.input(|i| (i.pointer.delta(), i.smooth_scroll_delta.y, i.modifiers.alt, i.modifiers.shift, i.stable_dt));
        let cam = &mut self.camera;

        if resp.dragged_by(PointerButton::Secondary) {
            ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
            cam.yaw -= delta.x * 0.004;
            cam.pitch = (cam.pitch - delta.y * 0.004).clamp(-1.55, 1.55);
            let rot = cam.rotation();
            let speed = if shift { 24.0 } else { 8.0 } * dt;
            let mut dir = Vec3::ZERO;
            ctx.input(|i| {
                for (key, v) in [(egui::Key::W, Vec3::NEG_Z), (egui::Key::S, Vec3::Z), (egui::Key::A, Vec3::NEG_X), (egui::Key::D, Vec3::X), (egui::Key::E, Vec3::Y), (egui::Key::Q, Vec3::NEG_Y)] {
                    if i.key_down(key) {
                        dir += v;
                    }
                }
            });
            cam.pos += rot * dir * speed;
            ctx.request_repaint();
        } else if resp.dragged_by(PointerButton::Middle) {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            let k = cam.pivot_dist.max(1.0) * 0.0016;
            let rot = cam.rotation();
            cam.pos += rot * Vec3::new(-delta.x * k, delta.y * k, 0.0);
        } else if alt && resp.dragged_by(PointerButton::Primary) {
            ctx.set_cursor_icon(egui::CursorIcon::Move);
            let pivot = cam.pos + cam.forward() * cam.pivot_dist;
            cam.yaw -= delta.x * 0.005;
            cam.pitch = (cam.pitch - delta.y * 0.005).clamp(-1.55, 1.55);
            cam.pos = pivot - cam.forward() * cam.pivot_dist;
        }
        if resp.hovered() && scroll != 0.0 {
            let step = scroll * 0.004 * cam.pivot_dist.max(1.0);
            let fwd = cam.forward();
            cam.pos += fwd * step;
            cam.pivot_dist = (cam.pivot_dist - step).max(0.5);
        }
        if resp.hovered() && ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.focus_selection();
        }
    }

    pub fn focus_selection(&mut self) {
        let Some(e) = self.selection else { return };
        let Some(g) = self.active().world.get::<GlobalTransform>(e).map(|g| g.0) else { return };
        let mesh = mesh_key(&self.active().world, e);
        let (center, radius) = match mesh.and_then(|m| self.renderer.mesh_bounds(&m)) {
            Some((min, max)) => (g.transform_point3((min + max) * 0.5), ((max - min) * 0.5 * g.to_scale_rotation_translation().0).length()),
            None => (g.w_axis.truncate(), 1.0),
        };
        self.camera.focus(center, radius);
    }

    fn pick(&mut self, view: &View, ndc: Vec2) -> Option<Entity> {
        let (origin, dir) = view.ray(ndc);
        let items: Vec<(Entity, Mat4, Option<String>)> = self
            .active()
            .world
            .iter::<GlobalTransform>()
            .map(|(e, g)| (e, g.0, mesh_key(&self.active().world, e)))
            .collect();
        let mut best: Option<(f32, Entity)> = None;
        for (e, m, mesh) in items {
            let hit = match mesh.and_then(|mesh| self.renderer.mesh_bounds(&mesh)) {
                Some((min, max)) => {
                    let inv = m.inverse();
                    let pad = Vec3::splat(0.01);
                    ray_box(inv.transform_point3(origin), inv.transform_vector3(dir), min - pad, max + pad)
                }
                None => {
                    let pos = m.w_axis.truncate();
                    ray_sphere(origin, dir, pos, (pos - view.eye).length() * 0.04 + 0.2)
                }
            };
            if let Some(t) = hit {
                if best.map_or(true, |(bt, _)| t < bt) {
                    best = Some((t, e));
                }
            }
        }
        // Clicking a system-made piece (a grass chunk) selects the object that owns it.
        best.map(|(_, e)| if self.active().world.has::<Transient>(e) { hierarchy::parent_of(&self.active().world, e).unwrap_or(e) } else { e })
    }

    fn build_debug(&mut self, view: &View, draw: &mut DebugDraw, playing: bool) {
        if self.show_grid && !playing {
            draw.grid(60, 1.0);
        }
        let selected = self.selection;
        let mut outline: Vec<(Mat4, String)> = Vec::new();
        {
            let world = &self.active().world;
            for sel in self.selected_entities() {
                for e in std::iter::once(sel).chain(hierarchy::descendants(world, sel)) {
                    if let (Some(key), Some(g)) = (mesh_key(world, e), world.get::<GlobalTransform>(e)) {
                        outline.push((g.0, key));
                    }
                }
            }
            if self.show_colliders {
                for (e, c) in world.iter::<Collider>() {
                    let Some(g) = world.get::<GlobalTransform>(e) else { continue };
                    let (scale, rot, pos) = g.0.to_scale_rotation_translation();
                    let color = if Some(e) == selected {
                        [1.0, 0.9, 0.3, 1.0]
                    } else if c.is_trigger {
                        [0.3, 0.6, 1.0, 0.8]
                    } else {
                        [0.3, 0.9, 0.4, 0.55]
                    };
                    draw.wire_shape(Mat4::from_rotation_translation(rot, pos), c.shape, c.size * scale, color);
                }
            }
            if !playing {
                for (e, l) in world.iter::<Light>() {
                    let Some(g) = world.get::<GlobalTransform>(e) else { continue };
                    let (pos, fwd) = (g.0.w_axis.truncate(), g.0.transform_vector3(Vec3::NEG_Z).normalize_or_zero());
                    let color = [l.color[0], l.color[1], l.color[2], 1.0];
                    let r = (pos - view.eye).length() * 0.03;
                    let (u, v) = (g.0.x_axis.truncate().normalize_or_zero(), g.0.y_axis.truncate().normalize_or_zero());
                    draw.circle(pos, u * r, v * r, color);
                    match l.kind {
                        LightKind::Directional => {
                            draw.line(pos, pos + fwd * 2.5, color);
                            for off in [u, -u, v, -v] {
                                draw.line(pos + off * r, pos + off * r + fwd * 2.0, color);
                            }
                        }
                        LightKind::Point if Some(e) == selected => {
                            let axes = [Vec3::X, Vec3::Y, Vec3::Z];
                            for i in 0..3 {
                                draw.circle(pos, axes[i] * l.range, axes[(i + 1) % 3] * l.range, [color[0], color[1], color[2], 0.4]);
                            }
                        }
                        LightKind::Spot => {
                            let reach = l.range.min(6.0);
                            let radius = reach * l.angle.clamp(1.0, 89.0).to_radians().tan();
                            let tip = pos + fwd * reach;
                            let a = fwd.any_orthonormal_vector();
                            let b = fwd.cross(a);
                            draw.circle(tip, a * radius, b * radius, color);
                            for off in [a, -a, b, -b] {
                                draw.line(pos, tip + off * radius, color);
                            }
                        }
                        LightKind::Point => {}
                    }
                }
                for (e, c) in world.iter::<Camera>() {
                    let Some(g) = world.get::<GlobalTransform>(e) else { continue };
                    let (_, rot, pos) = g.0.to_scale_rotation_translation();
                    let depth = 1.5;
                    let half_h = (c.fov.to_radians() * 0.5).tan() * depth;
                    let half_w = half_h * 16.0 / 9.0;
                    let corners: Vec<Vec3> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().map(|&(x, y)| pos + rot * Vec3::new(x * half_w, y * half_h, -depth)).collect();
                    let color = if Some(e) == selected { SELECT_COLOR } else { [0.8, 0.8, 0.8, 0.9] };
                    for i in 0..4 {
                        draw.line(pos, corners[i], color);
                        draw.line(corners[i], corners[(i + 1) % 4], color);
                    }
                }
                for (e, j) in world.iter::<Joint>() {
                    let Some(g) = world.get::<GlobalTransform>(e) else { continue };
                    let pivot = g.0.transform_point3(j.anchor);
                    let color = [1.0, 0.35, 0.9, 0.9];
                    let r = (pivot - view.eye).length() * 0.012;
                    for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                        draw.line(pivot - axis * r, pivot + axis * r, color);
                    }
                    draw.line(g.0.w_axis.truncate(), pivot, [1.0, 0.35, 0.9, 0.4]);
                    if let Some(t) = (!j.target.is_empty()).then(|| hierarchy::find_by_name(world, &j.target)).flatten() {
                        if let Some(tg) = world.get::<GlobalTransform>(t) {
                            draw.line(pivot, tg.0.w_axis.truncate(), [1.0, 0.35, 0.9, 0.4]);
                        }
                    }
                }
            }
        }
        for (m, mesh) in outline {
            if let Some((min, max)) = self.renderer.mesh_bounds(&mesh) {
                draw.wire_box(m, min, max, SELECT_COLOR);
            }
        }
    }
}
