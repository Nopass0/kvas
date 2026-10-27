//! Move / rotate / scale handles. Drag maths is done in screen space, which stays predictable
//! at any camera angle: the mouse is projected onto the on-screen direction of the handle.

use crate::editor::{Editor, Tool};
use crate::snap::round_to;
use crate::viewport::{to_ndc, to_screen};
use eframe::egui::{self, PointerButton, Pos2};
use glam::{Mat4, Quat, Vec3};
use kvas_engine::prelude::*;
use kvas_render::debug::DebugDraw;
use kvas_render::View;

const AXIS_COLORS: [[f32; 4]; 3] = [[0.92, 0.28, 0.28, 1.0], [0.4, 0.82, 0.35, 1.0], [0.32, 0.5, 0.95, 1.0]];
const HOT: [f32; 4] = [1.0, 0.9, 0.2, 1.0];
const PICK_RADIUS: f32 = 9.0;

struct Drag {
    axis: usize,
    start_mouse: Pos2,
    start_local: Transform,
    start_global: Mat4,
}

#[derive(Default)]
pub struct GizmoState {
    drag: Option<Drag>,
    /// Handle under the mouse when the button went down (the pointer may move before a drag starts).
    press_hot: Option<usize>,
}

fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let (ab, ap) = (b - a, p - a);
    let t = if ab.length_sq() < 1e-6 { 0.0 } else { (ap.dot(ab) / ab.length_sq()).clamp(0.0, 1.0) };
    (a + ab * t - p).length()
}

fn ring_points(center: Vec3, axis: Vec3, radius: f32) -> Vec<Vec3> {
    let u = axis.any_orthonormal_vector();
    let v = axis.cross(u);
    (0..=48).map(|i| {
        let a = i as f32 / 48.0 * std::f32::consts::TAU;
        center + (u * a.cos() + v * a.sin()) * radius
    })
    .collect()
}

impl Editor {
    /// Draws the gizmo for the selected object and handles dragging.
    /// Returns true while the mouse is over a handle or dragging one (so clicks must not pick).
    pub fn gizmo_ui(&mut self, ui: &mut egui::Ui, resp: &egui::Response, rect: egui::Rect, view: &View, draw: &mut DebugDraw) -> bool {
        let ctx = ui.ctx().clone();
        if resp.hovered() && !ctx.egui_wants_keyboard_input() && !ctx.input(|i| i.pointer.secondary_down()) {
            ctx.input(|i| {
                for (key, tool) in [(egui::Key::W, Tool::Move), (egui::Key::E, Tool::Rotate), (egui::Key::R, Tool::Scale)] {
                    if i.key_pressed(key) {
                        self.tool = tool;
                    }
                }
            });
        }
        let Some(sel) = self.selection else {
            self.gizmo.drag = None;
            return false;
        };
        let Some(global) = self.active().world.get::<GlobalTransform>(sel).map(|g| g.0) else { return false };
        let (_, rot, pos) = global.to_scale_rotation_translation();
        let size = ((pos - view.eye).length() * 0.17).max(0.05);
        let axes: [Vec3; 3] = if self.local_space || self.tool == Tool::Scale { [rot * Vec3::X, rot * Vec3::Y, rot * Vec3::Z] } else { [Vec3::X, Vec3::Y, Vec3::Z] };
        let mouse = ctx.input(|i| i.pointer.hover_pos());
        let project = |p: Vec3| to_screen(view, rect, p);

        // Which handle is under the mouse?
        let mut hot = self.gizmo.drag.as_ref().map(|d| d.axis);
        if hot.is_none() && resp.hovered() {
            if let Some(m) = mouse {
                let mut best = PICK_RADIUS;
                for (i, &axis) in axes.iter().enumerate() {
                    let d = match self.tool {
                        Tool::Move | Tool::Scale => match (project(pos), project(pos + axis * size)) {
                            (Some(a), Some(b)) => dist_to_segment(m, a, b),
                            _ => f32::MAX,
                        },
                        Tool::Rotate => ring_points(pos, axis, size)
                            .windows(2)
                            .filter_map(|w| Some(dist_to_segment(m, project(w[0])?, project(w[1])?)))
                            .fold(f32::MAX, f32::min),
                    };
                    if d < best {
                        best = d;
                        hot = Some(i);
                    }
                }
                // The centre square moves freely in the view plane (with vertex / surface snapping).
                if self.tool == Tool::Move && project(pos).is_some_and(|c| (c - m).length() < 11.0) {
                    hot = Some(3);
                }
            }
        }

        if self.gizmo.drag.is_some() {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if hot.is_some() {
            ctx.set_cursor_icon(egui::CursorIcon::Grab);
        }

        // Draw.
        for (i, &axis) in axes.iter().enumerate() {
            let color = if hot == Some(i) { HOT } else { AXIS_COLORS[i] };
            match self.tool {
                Tool::Move => {
                    let tip = pos + axis * size;
                    draw.overlay_line(pos, tip, color);
                    let (u, v) = (axis.any_orthonormal_vector(), axis.cross(axis.any_orthonormal_vector()));
                    let base = tip - axis * size * 0.2;
                    for off in [u, -u, v, -v] {
                        draw.overlay_line(tip, base + off * size * 0.06, color);
                    }
                }
                Tool::Scale => {
                    let tip = pos + axis * size;
                    draw.overlay_line(pos, tip, color);
                    let h = Vec3::splat(size * 0.06);
                    draw.overlay_box(Mat4::from_rotation_translation(rot, tip), -h, h, color);
                }
                Tool::Rotate => {
                    let pts = ring_points(pos, axis, size);
                    for w in pts.windows(2) {
                        draw.overlay_line(w[0], w[1], color);
                    }
                }
            }
        }

        if self.tool == Tool::Move {
            let h = Vec3::splat(size * 0.08);
            draw.overlay_box(Mat4::from_translation(pos), -h, h, if hot == Some(3) { HOT } else { [0.95, 0.95, 0.95, 1.0] });
        }

        // Interact.
        if ctx.input(|i| i.pointer.primary_pressed()) {
            self.gizmo.press_hot = if resp.hovered() { hot } else { None };
        }
        let alt = ctx.input(|i| i.modifiers.alt);
        if resp.drag_started_by(PointerButton::Primary) && !alt {
            if let (Some(axis), Some(m)) = (self.gizmo.press_hot.take(), ctx.input(|i| i.pointer.press_origin())) {
                if let Some(t) = self.active().world.get::<Transform>(sel).copied() {
                    self.begin_edit();
                    self.gizmo.drag = Some(Drag { axis, start_mouse: m, start_local: t, start_global: global });
                }
            }
        }
        let current = self.gizmo.drag.as_ref().map(|d| (d.axis, d.start_mouse, d.start_local, d.start_global));
        if let Some((daxis, start_mouse, start_local, start_global)) = current {
            let (ctrl, shift) = ctx.input(|i| (i.modifiers.ctrl, i.modifiers.shift));
            let mouse = mouse.unwrap_or(start_mouse);
            // Ctrl flips the grid snapping setting while dragging.
            let snapping = self.snap.grid != ctrl;
            if daxis == 3 {
                self.free_move(view, rect, mouse, sel, start_local, start_global, snapping, draw);
                if resp.drag_stopped() || !ctx.input(|i| i.pointer.primary_down()) {
                    self.gizmo.drag = None;
                }
                return true;
            }
            let axis = axes[daxis];
            // Pixels the mouse travelled along the on-screen direction of this axis, in world units.
            let along = |unit: f32| -> f32 {
                match (project(pos), project(pos + axis * unit)) {
                    (Some(a), Some(b)) if (b - a).length_sq() > 1e-4 => (mouse - start_mouse).dot(b - a) / (b - a).length_sq() * unit,
                    _ => 0.0,
                }
            };
            let (s_scale, s_rot, s_pos) = start_global.to_scale_rotation_translation();
            let parent_inv = hierarchy::parent_of(&self.active().world, sel).map_or(Mat4::IDENTITY, |p| hierarchy::global_matrix(&self.active().world, p)).inverse();
            let mut new_t = start_local;
            match self.tool {
                Tool::Move => {
                    let mut amount = along(1.0);
                    if snapping {
                        let step = self.snap.move_step;
                        if self.local_space {
                            amount = round_to(amount, step);
                        } else {
                            // World axes: land on the world grid, not on multiples of the start offset.
                            let base = s_pos.dot(axis);
                            amount = round_to(base + amount, step) - base;
                        }
                    }
                    if self.snap.vertex {
                        if let Some(v) = self.snap_vertex(view, rect, mouse, Some(sel)) {
                            amount = (v - s_pos).dot(axis);
                            draw.overlay_box(Mat4::from_translation(v), Vec3::splat(-size * 0.1), Vec3::splat(size * 0.1), HOT);
                        }
                    }
                    let world = Mat4::from_scale_rotation_translation(s_scale, s_rot, s_pos + axis * amount);
                    new_t.position = (parent_inv * world).w_axis.truncate();
                }
                Tool::Rotate => {
                    let Some(center) = project(pos) else { return true };
                    let (a, b) = (start_mouse - center, mouse - center);
                    let mut angle = (a.x * b.y - a.y * b.x).atan2(a.dot(b));
                    if axis.dot(view.eye - pos) > 0.0 {
                        angle = -angle;
                    }
                    if snapping {
                        angle = round_to(angle, self.snap.rotate_step.to_radians());
                    }
                    let world = Mat4::from_scale_rotation_translation(s_scale, Quat::from_axis_angle(axis, angle) * s_rot, s_pos);
                    new_t.rotation = Transform::from_matrix(parent_inv * world).rotation;
                }
                Tool::Scale => {
                    let mut factor = 1.0 + along(size) / size;
                    if snapping {
                        factor = round_to(factor, self.snap.scale_step);
                    }
                    let factor = factor.max(0.01);
                    if shift {
                        new_t.scale = start_local.scale * factor;
                    } else {
                        new_t.scale[daxis] = start_local.scale[daxis] * factor;
                    }
                }
            }
            if let Some(t) = self.active_mut().world.get_mut::<Transform>(sel) {
                *t = new_t;
            }
            if resp.drag_stopped() || !ctx.input(|i| i.pointer.primary_down()) {
                self.gizmo.drag = None;
            }
            return true;
        }
        hot.is_some()
    }

    /// Dragging the centre square: moves in the view plane, or glides over other objects' faces,
    /// or sticks to their vertices, depending on the snap settings.
    #[allow(clippy::too_many_arguments)]
    fn free_move(&mut self, view: &View, rect: egui::Rect, mouse: Pos2, sel: Entity, start_local: Transform, start_global: Mat4, snapping: bool, draw: &mut DebugDraw) {
        let (origin, dir) = view.ray(to_ndc(rect, mouse));
        let (s_scale, s_rot, s_pos) = start_global.to_scale_rotation_translation();
        let plane_n = (view.eye - s_pos).normalize_or_zero();
        let denom = dir.dot(plane_n);
        let mut new_pos = if denom.abs() > 1e-5 { origin + dir * ((s_pos - origin).dot(plane_n) / denom) } else { s_pos };
        let mut new_rot = s_rot;
        let mut marker: Option<Vec3> = None;
        if self.snap.surface {
            if let Some((p, n)) = self.raycast_faces(origin, dir, Some(sel)) {
                let low = self.support_below(sel, n);
                new_pos = p + n * low;
                if self.snap.align_normal {
                    new_rot = Quat::from_rotation_arc(Vec3::Y, n);
                }
                marker = Some(p);
            }
        }
        if self.snap.vertex {
            if let Some(v) = self.snap_vertex(view, rect, mouse, Some(sel)) {
                new_pos = v;
                marker = Some(v);
            }
        } else if snapping && marker.is_none() {
            let step = self.snap.move_step;
            new_pos = Vec3::new(round_to(new_pos.x, step), round_to(new_pos.y, step), round_to(new_pos.z, step));
        }
        if let Some(p) = marker {
            let h = Vec3::splat(((p - view.eye).length() * 0.012).max(0.02));
            draw.overlay_box(Mat4::from_translation(p), -h, h, HOT);
        }
        let parent_inv = hierarchy::parent_of(&self.active().world, sel).map_or(Mat4::IDENTITY, |p| hierarchy::global_matrix(&self.active().world, p)).inverse();
        let world = Mat4::from_scale_rotation_translation(s_scale, new_rot, new_pos);
        let mut t = start_local;
        let local = Transform::from_matrix(parent_inv * world);
        t.position = local.position;
        t.rotation = local.rotation;
        if let Some(target) = self.active_mut().world.get_mut::<Transform>(sel) {
            *target = t;
        }
    }
}
