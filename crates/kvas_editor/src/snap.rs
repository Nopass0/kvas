//! Snapping and quick transforms for the scene view: grid steps, magnetic snap to vertices,
//! placing objects on faces of other objects, and one-click rotate / flip / drop actions.

use crate::editor::Editor;
use crate::viewport::to_screen;
use eframe::egui;
use glam::{Mat4, Quat, Vec3};
use kvas_engine::mesh::MeshData;
use kvas_engine::prelude::*;
use kvas_render::View;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// What dragging snaps to, and the step sizes. Saved with the editor preferences.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapSettings {
    /// Round moves, rotations and scales to the steps below (hold Ctrl to flip this while dragging).
    pub grid: bool,
    /// Magnetic snap of the pivot to the nearest vertex of another object.
    pub vertex: bool,
    /// Free move places the object on the faces of other objects.
    pub surface: bool,
    /// With surface snap, also tilt the object to match the face.
    pub align_normal: bool,
    /// Move step in metres.
    pub move_step: f32,
    /// Rotation step in degrees.
    pub rotate_step: f32,
    /// Scale step.
    pub scale_step: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self { grid: false, vertex: false, surface: false, align_normal: false, move_step: 0.5, rotate_step: 15.0, scale_step: 0.1 }
    }
}

/// Pixel radius within which a vertex attracts the pivot.
const VERTEX_RADIUS: f32 = 22.0;

/// Rounds to a multiple of `step` (no-op for non-positive steps).
pub fn round_to(value: f32, step: f32) -> f32 {
    if step > 1e-6 { (value / step).round() * step } else { value }
}

fn ray_triangle(o: Vec3, d: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let (e1, e2) = (b - a, c - a);
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let t0 = o - a;
    let u = t0.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = t0.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 1e-4).then_some(t)
}

fn world_corners(m: Mat4, min: Vec3, max: Vec3) -> [Vec3; 8] {
    std::array::from_fn(|i| m.transform_point3(Vec3::new(if i & 1 == 0 { min.x } else { max.x }, if i & 2 == 0 { min.y } else { max.y }, if i & 4 == 0 { min.z } else { max.z })))
}

impl Editor {
    /// CPU triangles of an object, cached by mesh key (generated and modified meshes come straight from the world).
    pub fn cpu_mesh(&mut self, e: Entity) -> Option<Arc<MeshData>> {
        let world = &self.active().world;
        if let Some(d) = world.get::<DynamicMesh>(e) {
            return Some(d.data.clone());
        }
        let key = mesh_key(world, e).filter(|k| !k.is_empty())?;
        if let Some(hit) = self.mesh_cache.get(&key) {
            return hit.clone();
        }
        let mesh = MeshData::load(&self.assets, &key).ok().map(Arc::new);
        self.mesh_cache.insert(key, mesh.clone());
        mesh
    }

    /// Objects that snapping may target: everything with geometry except `exclude` and its children.
    fn snap_targets(&mut self, exclude: Option<Entity>) -> Vec<(Mat4, Arc<MeshData>)> {
        let skip: Vec<Entity> = exclude.map(|s| std::iter::once(s).chain(hierarchy::descendants(&self.active().world, s)).collect()).unwrap_or_default();
        let list: Vec<(Entity, Mat4)> = self.active().world.iter::<GlobalTransform>().map(|(e, g)| (e, g.0)).filter(|(e, _)| !skip.contains(e)).collect();
        list.into_iter().filter_map(|(e, m)| Some((m, self.cpu_mesh(e)?))).collect()
    }

    /// The vertex of another object nearest to the mouse (within a few pixels), in world space.
    pub fn snap_vertex(&mut self, view: &View, rect: egui::Rect, mouse: egui::Pos2, exclude: Option<Entity>) -> Option<Vec3> {
        let mut best: Option<(f32, Vec3)> = None;
        for (m, mesh) in self.snap_targets(exclude) {
            let (min, max) = mesh.bounds();
            let screen: Vec<egui::Pos2> = world_corners(m, min, max).iter().filter_map(|c| to_screen(view, rect, *c)).collect();
            if screen.len() < 8 {
                continue;
            }
            let hull = egui::Rect::from_points(&screen).expand(VERTEX_RADIUS);
            if !hull.contains(mouse) {
                continue;
            }
            let stride = (mesh.vertices.len() / 30_000).max(1);
            for v in mesh.vertices.iter().step_by(stride) {
                let p = m.transform_point3(Vec3::from(v.pos));
                let Some(s) = to_screen(view, rect, p) else { continue };
                let d = (s - mouse).length();
                if d < VERTEX_RADIUS && best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, p));
                }
            }
        }
        best.map(|(_, p)| p)
    }

    /// First face hit by a ray: world point and unit normal.
    pub fn raycast_faces(&mut self, origin: Vec3, dir: Vec3, exclude: Option<Entity>) -> Option<(Vec3, Vec3)> {
        let mut best: Option<(f32, Vec3, Vec3)> = None;
        for (m, mesh) in self.snap_targets(exclude) {
            let inv = m.inverse();
            let (lo, ld) = (inv.transform_point3(origin), inv.transform_vector3(dir));
            let (min, max) = mesh.bounds();
            let Some(aabb) = Aabb::new(min - Vec3::splat(0.01), max + Vec3::splat(0.01)) else { continue };
            let Some(ray) = Ray::new(lo, ld) else { continue };
            if aabb.ray_distance(ray, f32::MAX).is_none() {
                continue;
            }
            let normal_matrix = inv.transpose();
            for tri in mesh.indices.chunks_exact(3) {
                let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| Vec3::from(mesh.vertices[i as usize].pos));
                if let Some(t) = ray_triangle(lo, ld, a, b, c) {
                    // `t` is in units of the (possibly scaled) local direction, which equals world units because dir is unchanged.
                    if best.is_none_or(|(bt, ..)| t < bt) {
                        let n = normal_matrix.transform_vector3((b - a).cross(c - a)).normalize_or_zero();
                        let n = if n.dot(dir) > 0.0 { -n } else { n };
                        best = Some((t, origin + dir * t, n));
                    }
                }
            }
        }
        best.map(|(_, p, n)| (p, n))
    }

    /// How far the selected object extends against `normal` from its pivot (so it can rest on a face).
    pub fn support_below(&mut self, e: Entity, normal: Vec3) -> f32 {
        let Some(mesh) = self.cpu_mesh(e) else { return 0.0 };
        let Some(g) = self.active().world.get::<GlobalTransform>(e).map(|g| g.0) else { return 0.0 };
        let (min, max) = mesh.bounds();
        let pivot = g.w_axis.truncate();
        world_corners(g, min, max).iter().map(|c| (-normal).dot(*c - pivot)).fold(0.0, f32::max)
    }

    /// Applies a world-space matrix change to the selected object's local transform.
    fn set_global(&mut self, e: Entity, global: Mat4) {
        let parent_inv = hierarchy::parent_of(&self.active().world, e).map_or(Mat4::IDENTITY, |p| hierarchy::global_matrix(&self.active().world, p)).inverse();
        if let Some(t) = self.active_mut().world.get_mut::<Transform>(e) {
            *t = Transform::from_matrix(parent_inv * global);
        }
        hierarchy::update_globals(&mut self.active_mut().world);
    }

    /// Rotates the selection around a world axis through its pivot.
    pub fn rotate_selected(&mut self, axis: Vec3, degrees: f32) {
        let Some(e) = self.selection else { return };
        let Some(g) = self.active().world.get::<GlobalTransform>(e).map(|g| g.0) else { return };
        self.begin_edit();
        let (s, r, p) = g.to_scale_rotation_translation();
        self.set_global(e, Mat4::from_scale_rotation_translation(s, Quat::from_axis_angle(axis, degrees.to_radians()) * r, p));
    }

    /// Mirrors the selection across one of its local axes (scale -1 on that axis).
    pub fn flip_selected(&mut self, axis: usize) {
        let Some(e) = self.selection else { return };
        self.begin_edit();
        if let Some(t) = self.active_mut().world.get_mut::<Transform>(e) {
            t.scale[axis] = -t.scale[axis];
        }
    }

    /// Rounds the selection's position to the move step.
    pub fn snap_selected_to_grid(&mut self) {
        let Some(e) = self.selection else { return };
        let Some(g) = self.active().world.get::<GlobalTransform>(e).map(|g| g.0) else { return };
        self.begin_edit();
        let (s, r, p) = g.to_scale_rotation_translation();
        let step = self.snap.move_step;
        self.set_global(e, Mat4::from_scale_rotation_translation(s, r, Vec3::new(round_to(p.x, step), round_to(p.y, step), round_to(p.z, step))));
    }

    /// Lets the selection fall straight down onto the first face below it.
    pub fn drop_selected(&mut self) {
        let Some(e) = self.selection else { return };
        let Some(g) = self.active().world.get::<GlobalTransform>(e).map(|g| g.0) else { return };
        let (s, r, p) = g.to_scale_rotation_translation();
        let low = self.support_below(e, Vec3::Y);
        let origin = p + Vec3::Y * 0.02;
        if let Some((hit, _)) = self.raycast_faces(origin, Vec3::NEG_Y, Some(e)) {
            self.begin_edit();
            self.set_global(e, Mat4::from_scale_rotation_translation(s, r, Vec3::new(p.x, hit.y + low, p.z)));
            self.status = crate::i18n::t("Dropped onto the surface below").into();
        } else {
            self.status = crate::i18n::t("Nothing below to drop onto").into();
        }
    }

    /// Resets rotation of the selection.
    pub fn reset_rotation(&mut self) {
        let Some(e) = self.selection else { return };
        self.begin_edit();
        if let Some(t) = self.active_mut().world.get_mut::<Transform>(e) {
            t.rotation = Vec3::ZERO;
        }
    }
}

impl Editor {
    /// Rotate / flip / drop actions, shared by the scene-view toolbar and the context menu.
    pub fn transform_menu(&mut self, ui: &mut egui::Ui) {
        use crate::i18n::t;
        let has = self.selection.is_some();
        ui.add_enabled_ui(has, |ui| {
            ui.menu_button(t("Rotate 90°"), |ui| {
                for (label, axis) in [("X", Vec3::X), ("Y", Vec3::Y), ("Z", Vec3::Z)] {
                    if ui.button(format!("{label}  +90°")).clicked() {
                        self.rotate_selected(axis, 90.0);
                        ui.close();
                    }
                    if ui.button(format!("{label}  -90°")).clicked() {
                        self.rotate_selected(axis, -90.0);
                        ui.close();
                    }
                }
                if ui.button(t("Turn around (Y 180°)")).clicked() {
                    self.rotate_selected(Vec3::Y, 180.0);
                    ui.close();
                }
            });
            ui.menu_button(t("Flip (mirror)"), |ui| {
                for (i, label) in ["X", "Y", "Z"].into_iter().enumerate() {
                    if ui.button(label).clicked() {
                        self.flip_selected(i);
                        ui.close();
                    }
                }
            });
            if ui.button(t("Drop to surface")).clicked() {
                self.drop_selected();
                ui.close();
            }
            if ui.button(t("Snap position to grid")).clicked() {
                self.snap_selected_to_grid();
                ui.close();
            }
            if ui.button(t("Reset rotation")).clicked() {
                self.reset_rotation();
                ui.close();
            }
        });
    }

    /// Second floating strip of the scene view: snapping modes, the step and the transform menu.
    pub fn snap_overlay(&mut self, ui: &mut egui::Ui, top_left: egui::Pos2) {
        use crate::i18n::t;
        use crate::icons::{self, Icon};
        let bar = egui::Rect::from_min_size(top_left, egui::vec2(if self.snap.surface { 400.0 } else { 366.0 }, 34.0));
        ui.painter().add(crate::ui_kit::float_panel(bar, 10.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(bar.shrink2(egui::vec2(6.0, 3.0))).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            if icons::button(ui, Icon::Magnet, t("Snap to grid steps (Ctrl while dragging flips it)"), self.snap.grid).clicked() {
                self.snap.grid = !self.snap.grid;
            }
            if icons::button(ui, Icon::VertexSnap, t("Magnetic snap to vertices of other objects"), self.snap.vertex).clicked() {
                self.snap.vertex = !self.snap.vertex;
            }
            if icons::button(ui, Icon::SurfaceSnap, t("Place on faces of other objects (drag the centre square)"), self.snap.surface).clicked() {
                self.snap.surface = !self.snap.surface;
            }
            if self.snap.surface && icons::button(ui, Icon::Target, t("Tilt the object to match the face"), self.snap.align_normal).clicked() {
                self.snap.align_normal = !self.snap.align_normal;
            }
            ui.separator();
            let (presets, step, unit, id): (&[f32], &mut f32, &str, &str) = match self.tool {
                crate::editor::Tool::Move => (&[0.1, 0.25, 0.5, 1.0, 2.0, 5.0], &mut self.snap.move_step, "m", "snap-move"),
                crate::editor::Tool::Rotate => (&[5.0, 15.0, 30.0, 45.0, 90.0], &mut self.snap.rotate_step, "°", "snap-rotate"),
                crate::editor::Tool::Scale => (&[0.05, 0.1, 0.25, 0.5, 1.0], &mut self.snap.scale_step, "", "snap-scale"),
            };
            ui.label(egui::RichText::new(t("Step")).color(crate::theme::pal().text_dim));
            ui.add(egui::DragValue::new(step).speed(0.01).range(0.01..=360.0).suffix(unit).max_decimals(2));
            egui::ComboBox::from_id_salt(id).selected_text("").width(14.0).show_ui(ui, |ui| {
                for p in presets {
                    if ui.selectable_label((*step - p).abs() < 1e-4, format!("{p}{unit}")).clicked() {
                        *step = *p;
                    }
                }
            });
            ui.separator();
            ui.menu_button(t("Transform"), |ui| self.transform_menu(ui));
        });
    }
}
