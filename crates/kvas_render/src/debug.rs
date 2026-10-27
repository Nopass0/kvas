//! Immediate-mode line drawing for grids, gizmos and collider outlines.

use glam::{Mat4, Vec3};
use kvas_engine::components::ShapeKind;
use std::f32::consts::TAU;

/// World-space coloured line segment.
#[derive(Clone, Copy)]
pub struct Line {
    /// Segment start in world metres.
    pub a: Vec3,
    /// Segment end in world metres.
    pub b: Vec3,
    /// RGBA line colour.
    pub color: [f32; 4],
}

/// `depth` lines are hidden behind geometry, `overlay` lines are always visible (gizmo handles).
#[derive(Default)]
pub struct DebugDraw {
    /// Depth-tested segments, occluded by scene geometry.
    pub depth: Vec<Line>,
    /// Segments drawn over scene geometry.
    pub overlay: Vec<Line>,
}

impl DebugDraw {
    /// Appends a depth-tested line segment.
    pub fn line(&mut self, a: Vec3, b: Vec3, color: [f32; 4]) {
        self.depth.push(Line { a, b, color });
    }

    /// Appends an always-visible line segment.
    pub fn overlay_line(&mut self, a: Vec3, b: Vec3, color: [f32; 4]) {
        self.overlay.push(Line { a, b, color });
    }

    /// Draws a depth-tested ellipse spanned by world-space radius vectors `u` and `v`.
    pub fn circle(&mut self, center: Vec3, u: Vec3, v: Vec3, color: [f32; 4]) {
        circle_into(&mut self.depth, center, u, v, color);
    }

    /// Draws an always-visible ellipse spanned by world-space radius vectors.
    pub fn overlay_circle(&mut self, center: Vec3, u: Vec3, v: Vec3, color: [f32; 4]) {
        circle_into(&mut self.overlay, center, u, v, color);
    }

    /// Wireframe of the box `min..max` in the space of `m`.
    pub fn wire_box(&mut self, m: Mat4, min: Vec3, max: Vec3, color: [f32; 4]) {
        box_into(&mut self.depth, m, min, max, color);
    }

    /// Draws an always-visible transformed local box.
    pub fn overlay_box(&mut self, m: Mat4, min: Vec3, max: Vec3, color: [f32; 4]) {
        box_into(&mut self.overlay, m, min, max, color);
    }

    /// Outline of a collider. `pose` is rigid (no scale); `size` is already multiplied by world scale.
    pub fn wire_shape(&mut self, pose: Mat4, shape: ShapeKind, size: Vec3, color: [f32; 4]) {
        let size = size.abs();
        let (x, y, z) = (pose.x_axis.truncate().normalize(), pose.y_axis.truncate().normalize(), pose.z_axis.truncate().normalize());
        let c = pose.w_axis.truncate();
        let radius = size.x.max(size.z) * 0.5;
        match shape {
            ShapeKind::Box => self.wire_box(pose, -size * 0.5, size * 0.5, color),
            ShapeKind::Sphere => {
                let r = size.max_element() * 0.5;
                self.circle(c, x * r, y * r, color);
                self.circle(c, y * r, z * r, color);
                self.circle(c, x * r, z * r, color);
            }
            ShapeKind::Cylinder | ShapeKind::Capsule => {
                let half = size.y * 0.5;
                let straight = if shape == ShapeKind::Capsule { (half - radius).max(0.0) } else { half };
                let (top, bottom) = (c + y * straight, c - y * straight);
                self.circle(top, x * radius, z * radius, color);
                self.circle(bottom, x * radius, z * radius, color);
                for d in [x, -x, z, -z] {
                    self.line(top + d * radius, bottom + d * radius, color);
                }
                if shape == ShapeKind::Capsule {
                    for (end, s) in [(top, 1.0), (bottom, -1.0)] {
                        self.circle(end, x * radius, y * radius * s, color);
                        self.circle(end, z * radius, y * radius * s, color);
                    }
                }
            }
        }
    }

    /// Ground grid on the XZ plane with highlighted axes.
    pub fn grid(&mut self, half_extent: i32, step: f32) {
        if half_extent < 0 || !step.is_finite() || step <= 0.0 {
            return;
        }
        let half_extent = half_extent.min(10_000);
        let minor = [0.5, 0.5, 0.5, 0.18];
        let major = [0.5, 0.5, 0.5, 0.35];
        let e = half_extent as f32 * step;
        for i in -half_extent..=half_extent {
            let t = i as f32 * step;
            let color = if i % 10 == 0 { major } else { minor };
            if i == 0 {
                self.line(Vec3::new(-e, 0.0, 0.0), Vec3::new(e, 0.0, 0.0), [0.85, 0.25, 0.25, 0.7]);
                self.line(Vec3::new(0.0, 0.0, -e), Vec3::new(0.0, 0.0, e), [0.3, 0.45, 0.9, 0.7]);
            } else {
                self.line(Vec3::new(-e, 0.0, t), Vec3::new(e, 0.0, t), color);
                self.line(Vec3::new(t, 0.0, -e), Vec3::new(t, 0.0, e), color);
            }
        }
    }

    /// Total segment count across depth-tested and overlay layers.
    pub fn len(&self) -> usize {
        self.depth.len() + self.overlay.len()
    }
    /// Whether both layers contain no segments.
    pub fn is_empty(&self) -> bool {
        self.depth.is_empty() && self.overlay.is_empty()
    }
    /// Clears both layers while retaining their allocations for the next frame.
    pub fn clear(&mut self) {
        self.depth.clear();
        self.overlay.clear();
    }

    /// Draws a sphere as three perpendicular great circles.
    /// Non-finite or non-positive radii are ignored.
    pub fn sphere(&mut self, center: Vec3, radius: f32, color: [f32; 4]) {
        if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
            return;
        }
        self.circle(center, Vec3::X * radius, Vec3::Y * radius, color);
        self.circle(center, Vec3::Y * radius, Vec3::Z * radius, color);
        self.circle(center, Vec3::Z * radius, Vec3::X * radius, color);
    }

    /// Draws a depth-tested arrow with a four-segment head, scaled to its length.
    /// Degenerate or non-finite endpoints are ignored.
    pub fn arrow(&mut self, start: Vec3, end: Vec3, color: [f32; 4]) {
        if !start.is_finite() || !end.is_finite() {
            return;
        }
        let delta = end - start;
        let Some(direction) = delta.try_normalize() else {
            return;
        };
        let length = delta.length();
        if !length.is_finite() {
            return;
        }
        let axis = if direction.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let u = direction.cross(axis).normalize();
        let v = direction.cross(u);
        let head = length * 0.2;
        self.line(start, end, color);
        for offset in [u, -u, v, -v] {
            self.line(end, end - direction * head + offset * head * 0.5, color);
        }
    }

    /// Draws local +X/+Y/+Z axes in red/green/blue, transformed into world space.
    pub fn axes(&mut self, transform: Mat4, length: f32) {
        if !transform.is_finite() || !length.is_finite() || length <= 0.0 {
            return;
        }
        let origin = transform.transform_point3(Vec3::ZERO);
        for (axis, color) in [(Vec3::X, [1.0, 0.2, 0.2, 1.0]), (Vec3::Y, [0.2, 1.0, 0.2, 1.0]), (Vec3::Z, [0.2, 0.4, 1.0, 1.0])] {
            self.arrow(origin, transform.transform_point3(axis * length), color);
        }
    }
}

fn circle_into(out: &mut Vec<Line>, center: Vec3, u: Vec3, v: Vec3, color: [f32; 4]) {
    const SEGMENTS: usize = 40;
    let point = |i: usize| {
        let (s, c) = (TAU * i as f32 / SEGMENTS as f32).sin_cos();
        center + u * c + v * s
    };
    for i in 0..SEGMENTS {
        out.push(Line { a: point(i), b: point(i + 1), color });
    }
}

fn box_into(out: &mut Vec<Line>, m: Mat4, min: Vec3, max: Vec3, color: [f32; 4]) {
    let corner = |i: usize| {
        m.transform_point3(Vec3::new(
            if i & 1 == 0 { min.x } else { max.x },
            if i & 2 == 0 { min.y } else { max.y },
            if i & 4 == 0 { min.z } else { max.z },
        ))
    };
    for i in 0..8usize {
        for bit in [1usize, 2, 4] {
            if i & bit == 0 {
                out.push(Line { a: corner(i), b: corner(i | bit), color });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn helpers_produce_finite_geometry_and_clear_both_layers() {
        let mut draw = DebugDraw::default();
        draw.arrow(Vec3::ZERO, Vec3::Y, [1.0; 4]);
        assert_eq!(draw.len(), 5);
        draw.sphere(Vec3::ZERO, 1.0, [1.0; 4]);
        draw.axes(Mat4::IDENTITY, 1.0);
        draw.overlay_box(Mat4::IDENTITY, -Vec3::ONE, Vec3::ONE, [1.0; 4]);
        assert!(draw.depth.iter().chain(draw.overlay.iter()).all(|line| line.a.is_finite() && line.b.is_finite()));
        let count = draw.len();
        draw.arrow(Vec3::ZERO, Vec3::ZERO, [1.0; 4]);
        draw.sphere(Vec3::ZERO, -1.0, [1.0; 4]);
        assert_eq!(draw.len(), count);
        draw.clear();
        assert!(draw.is_empty());
    }
}
