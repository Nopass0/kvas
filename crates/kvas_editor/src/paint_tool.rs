//! Texture paint brush for the scene view: paint dirt, sand or stone onto a surface that has a
//! `TexturePaint` component, with soft edges so the layers blend instead of ending in a hard line.

use crate::editor::Editor;
use crate::i18n::t;
use crate::icons::{self, Icon};
use crate::theme::{self, pal};
use crate::viewport::to_ndc;
use eframe::egui::{self, Color32, PointerButton, RichText};
use glam::{Mat4, Vec2, Vec3};
use kvas_engine::paint::{BrushMode, PaintMaps, TexturePaint};
use kvas_engine::prelude::*;
use kvas_render::debug::DebugDraw;
use kvas_render::View;

/// Brush settings and mode. Saved with the editor session, not with the project.
pub struct PaintTool {
    pub active: bool,
    /// 0 = base layer (erase), 1..=3 = the painted layers.
    pub layer: usize,
    pub smooth: bool,
    /// Brush radius in metres.
    pub radius: f32,
    /// Strength per second, 0..1.
    pub strength: f32,
    /// 0 soft, 1 hard edge.
    pub hardness: f32,
}

impl Default for PaintTool {
    fn default() -> Self {
        Self { active: false, layer: 1, smooth: false, radius: 1.6, strength: 0.7, hardness: 0.15 }
    }
}

struct Hit {
    point: Vec3,
    normal: Vec3,
    uv: Vec2,
    /// uv units per metre of surface, to turn the brush radius into uv space.
    uv_per_metre: f32,
}

/// Möller–Trumbore with barycentric coordinates: `(t, u, v)`.
fn ray_triangle_bary(o: Vec3, d: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<(f32, f32, f32)> {
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
    (t > 1e-4).then_some((t, u, v))
}

impl Editor {
    /// Mask name and component of the selected painted surface.
    fn painted_selection(&self) -> Option<(Entity, String)> {
        let e = self.selection?;
        let w = &self.active().world;
        let tp = w.get::<TexturePaint>(e)?;
        Some((e, tp.mask_name(&w.get::<Name>(e).map(|n| n.0.clone()).unwrap_or_default())))
    }

    pub fn has_paint_target(&self) -> bool {
        self.painted_selection().is_some()
    }

    fn raycast_uv(&mut self, e: Entity, origin: Vec3, dir: Vec3) -> Option<Hit> {
        let mesh = self.cpu_mesh(e)?;
        let m: Mat4 = self.active().world.get::<GlobalTransform>(e)?.0;
        let inv = m.inverse();
        let (lo, ld) = (inv.transform_point3(origin), inv.transform_vector3(dir));
        let normal_matrix = inv.transpose();
        let mut best: Option<(f32, usize, f32, f32)> = None;
        for (i, tri) in mesh.indices.chunks_exact(3).enumerate() {
            let [a, b, c] = [tri[0], tri[1], tri[2]].map(|k| Vec3::from(mesh.vertices[k as usize].pos));
            if let Some((t, u, v)) = ray_triangle_bary(lo, ld, a, b, c) {
                if best.is_none_or(|(bt, ..)| t < bt) {
                    best = Some((t, i, u, v));
                }
            }
        }
        let (t, i, u, v) = best?;
        let tri = &mesh.indices[i * 3..i * 3 + 3];
        let vs = [mesh.vertices[tri[0] as usize], mesh.vertices[tri[1] as usize], mesh.vertices[tri[2] as usize]];
        let uv = |k: usize| Vec2::from(vs[k].uv);
        let hit_uv = uv(0) * (1.0 - u - v) + uv(1) * u + uv(2) * v;
        let world = |k: usize| m.transform_point3(Vec3::from(vs[k].pos));
        let (w0, w1, w2) = (world(0), world(1), world(2));
        let edge_m = ((w1 - w0).length() + (w2 - w0).length()).max(1e-4);
        let edge_uv = (uv(1) - uv(0)).length() + (uv(2) - uv(0)).length();
        let n = normal_matrix.transform_vector3((Vec3::from(vs[1].pos) - Vec3::from(vs[0].pos)).cross(Vec3::from(vs[2].pos) - Vec3::from(vs[0].pos))).normalize_or_zero();
        Some(Hit { point: origin + dir * t, normal: if n.dot(dir) > 0.0 { -n } else { n }, uv: hit_uv, uv_per_metre: edge_uv / edge_m })
    }

    /// Paints while the primary button is held over the selected surface, and draws the brush ring.
    pub fn paint_in_viewport(&mut self, ui: &egui::Ui, resp: &egui::Response, rect: egui::Rect, view: &View, draw: &mut DebugDraw) {
        if !self.paint.active {
            return;
        }
        let Some((e, mask)) = self.painted_selection() else { return };
        let Some(pos) = ui.input(|i| i.pointer.hover_pos()).filter(|p| rect.contains(*p) && resp.hovered()) else { return };
        let (origin, dir) = view.ray(to_ndc(rect, pos));
        let Some(hit) = self.raycast_uv(e, origin, dir) else { return };
        // Brush ring lying on the surface.
        let (tangent, bitangent) = (hit.normal.any_orthonormal_vector(), hit.normal.cross(hit.normal.any_orthonormal_vector()));
        let color = if self.paint.smooth { [0.6, 0.9, 1.0, 1.0] } else { [1.0, 0.85, 0.3, 1.0] };
        let mut last = None;
        for k in 0..=40 {
            let a = k as f32 / 40.0 * std::f32::consts::TAU;
            let p = hit.point + hit.normal * 0.02 + (tangent * a.cos() + bitangent * a.sin()) * self.paint.radius;
            if let Some(l) = last {
                draw.overlay_line(l, p, color);
            }
            last = Some(p);
        }
        let alt = ui.input(|i| i.modifiers.alt);
        if resp.dragged_by(PointerButton::Primary) || (resp.hovered() && ui.input(|i| i.pointer.primary_down()) && !alt) {
            if alt {
                return;
            }
            let dt = ui.input(|i| i.stable_dt).min(0.1);
            let mode = if self.paint.smooth { BrushMode::Smooth } else { BrushMode::Layer(self.paint.layer) };
            PaintMaps::global().brush(&mask, [hit.uv.x, hit.uv.y], self.paint.radius * hit.uv_per_metre, (self.paint.strength * dt * 8.0).min(1.0), self.paint.hardness, mode);
            ui.ctx().request_repaint();
        }
    }

    /// Writes every edited mask to the project's `paint` folder.
    pub fn save_paint(&mut self) {
        let n = PaintMaps::global().save_all(&self.project.assets_dir());
        if n > 0 {
            self.reload_assets();
            self.status = crate::i18n::tf("Saved {} paint mask(s)", &[&n.to_string()]);
        }
    }

    /// Strip with the brush controls, shown while a painted surface is selected.
    pub fn paint_overlay(&mut self, ui: &mut egui::Ui, top_left: egui::Pos2) {
        let Some((_, mask)) = self.painted_selection() else { return };
        let bar = egui::Rect::from_min_size(top_left, egui::vec2(560.0, 34.0));
        ui.painter().add(crate::ui_kit::float_panel(bar, 10.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(bar.shrink2(egui::vec2(6.0, 3.0))).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            if icons::button(ui, Icon::Brush, t("Texture paint brush (hold the mouse over the surface)"), self.paint.active).clicked() {
                self.paint.active = !self.paint.active;
            }
            ui.separator();
            for (layer, label) in [(0usize, "1"), (1, "2"), (2, "3"), (3, "4")] {
                let on = !self.paint.smooth && self.paint.layer == layer;
                let tip = if layer == 0 { t("Layer 1 (base): erases the others") } else { t("Paint this layer") };
                let text = RichText::new(label).color(if on { Color32::WHITE } else { pal().text });
                if ui.add(egui::Button::new(text).fill(if on { theme::ACCENT } else { pal().button }).min_size(egui::vec2(24.0, 22.0))).on_hover_text(tip).clicked() {
                    self.paint.layer = layer;
                    self.paint.smooth = false;
                }
            }
            if ui.add(egui::Button::new(t("Soften")).fill(if self.paint.smooth { theme::ACCENT } else { pal().button })).on_hover_text(t("Blur the transitions under the brush")).clicked() {
                self.paint.smooth = !self.paint.smooth;
            }
            ui.separator();
            ui.add(egui::DragValue::new(&mut self.paint.radius).speed(0.05).range(0.1..=20.0).prefix(format!("{} ", t("size"))).suffix(" m"));
            ui.add(egui::DragValue::new(&mut self.paint.strength).speed(0.01).range(0.02..=1.0).prefix(format!("{} ", t("strength"))));
            ui.add(egui::DragValue::new(&mut self.paint.hardness).speed(0.01).range(0.0..=0.95).prefix(format!("{} ", t("hard"))));
            ui.separator();
            if ui.button(t("Fill")).on_hover_text(t("Fill the whole mask with the chosen layer")).clicked() {
                PaintMaps::global().fill(&mask, self.paint.layer);
            }
            if ui.button(t("Save mask")).clicked() {
                self.save_paint();
            }
        });
    }
}
