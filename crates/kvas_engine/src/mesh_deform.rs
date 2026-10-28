//! Vertex access and deformation for [`EditableMesh`]: the building blocks for procedural
//! animation, destruction, terrain tools and "squash and stretch" effects from scripts.
//!
//! Every method works in the mesh's **local space** (before the object's transform), marks the
//! mesh as changed so the GPU copy is refreshed, and keeps the normals as they are. Call
//! [`EditableMesh::recalculate_normals`] after a deformation when lighting matters.
//!
//! ```
//! use kvas_engine::{mesh::MeshData, mesh_edit::EditableMesh};
//! use glam::Vec3;
//! let mut mesh = EditableMesh::from_mesh(&MeshData::primitive("cube").unwrap()).unwrap();
//! let (min, max) = mesh.bounds();
//! mesh.twist(Vec3::Y, 45.0);          // 45 degrees per metre along Y
//! mesh.wave(Vec3::X, Vec3::Y, 0.1, 1.0, 0.0);
//! mesh.recalculate_normals().unwrap();
//! assert!(mesh.bounds().1.y >= max.y - 0.001 && min.y <= mesh.bounds().0.y + 0.001);
//! ```

use crate::mesh_edit::EditableMesh;
use crate::noise::{Noise, NoiseKind};
use glam::{Mat4, Quat, Vec3};

impl EditableMesh {
    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Local position of vertex `i`.
    pub fn position(&self, i: usize) -> Option<Vec3> {
        self.vertices.get(i).map(|v| Vec3::from(v.pos))
    }

    /// Moves one vertex. Returns `false` for a bad index or a non-finite position.
    pub fn set_position(&mut self, i: usize, p: Vec3) -> bool {
        match self.vertices.get_mut(i) {
            Some(v) if p.is_finite() => {
                v.pos = p.to_array();
                self.changed();
                true
            }
            _ => false,
        }
    }

    /// Axis-aligned local bounds `(min, max)`; both zero for an empty mesh.
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let mut it = self.vertices.iter().map(|v| Vec3::from(v.pos));
        let Some(first) = it.next() else { return (Vec3::ZERO, Vec3::ZERO) };
        it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p)))
    }

    /// Replaces every position with `f(index, position, normal)`. Non-finite results are ignored.
    pub fn map_positions(&mut self, mut f: impl FnMut(usize, Vec3, Vec3) -> Vec3) {
        for (i, v) in self.vertices.iter_mut().enumerate() {
            let p = f(i, Vec3::from(v.pos), Vec3::from(v.normal));
            if p.is_finite() {
                v.pos = p.to_array();
            }
        }
        self.changed();
    }

    /// Indices of the vertices inside a sphere (local space): the selection for brush-like edits.
    pub fn select_sphere(&self, center: Vec3, radius: f32) -> Vec<usize> {
        let r2 = radius * radius;
        self.vertices.iter().enumerate().filter(|(_, v)| Vec3::from(v.pos).distance_squared(center) <= r2).map(|(i, _)| i).collect()
    }

    /// Indices of the vertices on the positive side of a plane through `point` with `normal`.
    pub fn select_half_space(&self, point: Vec3, normal: Vec3) -> Vec<usize> {
        let n = normal.normalize_or_zero();
        self.vertices.iter().enumerate().filter(|(_, v)| (Vec3::from(v.pos) - point).dot(n) >= 0.0).map(|(i, _)| i).collect()
    }

    /// Adds `delta` to the listed vertices.
    pub fn translate_vertices(&mut self, indices: &[usize], delta: Vec3) {
        for &i in indices {
            if let Some(v) = self.vertices.get_mut(i) {
                v.pos = (Vec3::from(v.pos) + delta).to_array();
            }
        }
        self.changed();
    }

    /// Moves the whole mesh.
    pub fn translate(&mut self, delta: Vec3) {
        self.map_positions(|_, p, _| p + delta);
    }

    /// Scales around `center` per axis.
    pub fn scale_about(&mut self, center: Vec3, scale: Vec3) {
        self.map_positions(|_, p, _| center + (p - center) * scale);
    }

    /// Applies an arbitrary matrix to positions and (rotation part only) normals.
    pub fn transform(&mut self, m: Mat4) {
        let (_, rot, _) = m.to_scale_rotation_translation();
        for v in &mut self.vertices {
            v.pos = m.transform_point3(Vec3::from(v.pos)).to_array();
            v.normal = (rot * Vec3::from(v.normal)).normalize_or_zero().to_array();
        }
        self.changed();
    }

    /// Pushes every vertex along its normal (`amount` metres; negative shrinks).
    pub fn inflate(&mut self, amount: f32) {
        self.map_positions(|_, p, n| p + n * amount);
    }

    /// Twists the mesh around `axis` through the origin: `degrees_per_metre` of rotation per metre along the axis.
    pub fn twist(&mut self, axis: Vec3, degrees_per_metre: f32) {
        let axis = axis.normalize_or_zero();
        if axis == Vec3::ZERO {
            return;
        }
        self.map_positions(|_, p, _| Quat::from_axis_angle(axis, (p.dot(axis) * degrees_per_metre).to_radians()) * p);
    }

    /// Scales the cross-section along `axis`: `start` at the lowest point of the mesh, `end` at the highest.
    pub fn taper(&mut self, axis: Vec3, start: f32, end: f32) {
        let axis = axis.normalize_or_zero();
        if axis == Vec3::ZERO {
            return;
        }
        let (lo, hi) = self.bounds();
        let (a, b) = (lo.dot(axis).min(hi.dot(axis)), lo.dot(axis).max(hi.dot(axis)));
        self.map_positions(|_, p, _| {
            let t = if b - a > 1e-6 { (p.dot(axis) - a) / (b - a) } else { 0.0 };
            let along = axis * p.dot(axis);
            along + (p - along) * (start + (end - start) * t)
        });
    }

    /// Bends the mesh around the `bend_axis` direction: points are rotated around it by
    /// `degrees_per_metre` times their distance along `along`.
    pub fn bend(&mut self, along: Vec3, bend_axis: Vec3, degrees_per_metre: f32) {
        let (along, axis) = (along.normalize_or_zero(), bend_axis.normalize_or_zero());
        if along == Vec3::ZERO || axis == Vec3::ZERO {
            return;
        }
        self.map_positions(|_, p, _| Quat::from_axis_angle(axis, (p.dot(along) * degrees_per_metre).to_radians()) * p);
    }

    /// A travelling sine wave: vertices move along `displace` by `amplitude * sin(2pi * (d / wavelength + phase))`,
    /// where `d` is the position along `direction`.
    pub fn wave(&mut self, direction: Vec3, displace: Vec3, amplitude: f32, wavelength: f32, phase: f32) {
        let direction = direction.normalize_or_zero();
        let wavelength = wavelength.max(1e-4);
        self.map_positions(|_, p, _| p + displace * amplitude * ((p.dot(direction) / wavelength + phase) * std::f32::consts::TAU).sin());
    }

    /// Displaces vertices along their normals by 3D noise (rocks, terrain, hand-made wobble).
    /// `time` slides the noise so repeated calls animate.
    pub fn noise_displace(&mut self, noise: &Noise, amplitude: f32, time: f32) {
        self.map_positions(|_, p, n| p + n * amplitude * noise.sample3(p.x, p.y, p.z + time));
    }

    /// Laplacian smoothing: moves each vertex towards the average of its neighbours.
    /// Vertices that share a position stay together (hard-edged meshes do not tear).
    pub fn smooth(&mut self, iterations: u32, factor: f32) {
        let key = |p: [f32; 3]| p.map(|v| (v as f64 * 10_000.0).round() as i64);
        for _ in 0..iterations.min(32) {
            let mut sum = std::collections::HashMap::<[i64; 3], (Vec3, f32)>::new();
            for t in self.indices.chunks_exact(3) {
                for k in 0..3 {
                    let (a, b) = (t[k] as usize, t[(k + 1) % 3] as usize);
                    let (pa, pb) = (self.vertices[a].pos, self.vertices[b].pos);
                    let e = sum.entry(key(pa)).or_default();
                    e.0 += Vec3::from(pb);
                    e.1 += 1.0;
                    let e = sum.entry(key(pb)).or_default();
                    e.0 += Vec3::from(pa);
                    e.1 += 1.0;
                }
            }
            for v in &mut self.vertices {
                if let Some((s, n)) = sum.get(&key(v.pos)) {
                    if *n > 0.0 {
                        let p = Vec3::from(v.pos);
                        v.pos = p.lerp(*s / *n, factor.clamp(0.0, 1.0)).to_array();
                    }
                }
            }
        }
        self.changed();
    }

    /// A ready noise for [`Self::noise_displace`]: smooth Perlin at the given frequency.
    pub fn default_noise(frequency: f32, seed: u32) -> Noise {
        Noise::new(NoiseKind::Perlin).with_seed(seed).with_frequency(frequency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshData;

    fn cube() -> EditableMesh {
        EditableMesh::from_mesh(&MeshData::primitive("cube").unwrap()).unwrap()
    }

    #[test]
    fn deformers_change_geometry_and_bump_the_revision() {
        let mut m = cube();
        let rev = m.revision;
        let before = m.bounds();
        m.taper(Vec3::Y, 1.0, 0.0);
        assert!(m.revision > rev);
        // The top of a fully tapered cube collapses to a point.
        let top: Vec<_> = m.vertices.iter().filter(|v| v.pos[1] > 0.49).collect();
        assert!(top.iter().all(|v| v.pos[0].abs() < 1e-4 && v.pos[2].abs() < 1e-4));
        assert_eq!(m.bounds().1.y, before.1.y);
        let mut m = cube();
        m.bend(Vec3::Y, Vec3::Z, 30.0);
        assert!(m.bounds() != before);
        let mut m = cube();
        m.noise_displace(&EditableMesh::default_noise(2.0, 1), 0.2, 0.0);
        m.smooth(2, 0.5);
        m.recalculate_normals().unwrap();
        assert!(m.vertices.iter().all(|v| Vec3::from(v.pos).is_finite()));
    }

    #[test]
    fn selection_and_access_are_checked() {
        let mut m = cube();
        assert_eq!(m.select_sphere(Vec3::ZERO, 10.0).len(), m.vertex_count());
        assert!(m.select_half_space(Vec3::ZERO, Vec3::Y).len() < m.vertex_count());
        assert!(!m.set_position(9999, Vec3::ZERO));
        assert!(!m.set_position(0, Vec3::NAN));
        assert!(m.set_position(0, Vec3::ONE));
        let sel = m.select_half_space(Vec3::ZERO, Vec3::Y);
        let y0 = m.position(sel[0]).unwrap().y;
        m.translate_vertices(&sel, Vec3::Y);
        assert!((m.position(sel[0]).unwrap().y - y0 - 1.0).abs() < 1e-5);
    }
}
