//! UV unwrapping and normal smoothing for [`EditableMesh`]: projections, UV transforms, flat shading
//! and crease-angle smoothing. All methods work on local-space geometry and mark the mesh as changed.
//!
//! ```
//! use kvas_engine::{mesh::MeshData, mesh_edit::EditableMesh};
//! use glam::Vec3;
//! let mut mesh = EditableMesh::from_mesh(&MeshData::primitive("cube").unwrap()).unwrap();
//! mesh.project_planar(Vec3::Y, 2.0);   // 2 metres per texture repeat, seen from above
//! mesh.normalize_uv(0.02);             // fit into 0..1 with a small border
//! mesh.smooth_normals(40.0);           // edges sharper than 40 degrees stay hard
//! ```

use crate::mesh_edit::EditableMesh;
use glam::Vec3;
use std::collections::HashMap;

fn key(p: [f32; 3]) -> [i64; 3] {
    p.map(|v| (v as f64 * 10_000.0).round() as i64)
}

impl EditableMesh {
    /// Planar projection: UVs are the position on the plane perpendicular to `axis`, divided by `scale`
    /// (metres per UV unit). Best for floors, walls and decals. Does nothing for a zero axis or `scale <= 0`.
    pub fn project_planar(&mut self, axis: Vec3, scale: f32) {
        let n = axis.normalize_or_zero();
        if n == Vec3::ZERO || scale <= 0.0 {
            return;
        }
        let u = n.any_orthonormal_vector();
        let v = n.cross(u);
        for vert in &mut self.vertices {
            let p = Vec3::from(vert.pos);
            vert.uv = [p.dot(u) / scale, p.dot(v) / scale];
        }
        self.changed();
    }

    /// Cylindrical projection around the Y axis: U is the angle (0..1 around), V the height divided by `scale`.
    /// Good for trunks, bottles and columns. The seam sits at +X.
    pub fn project_cylindrical(&mut self, scale: f32) {
        if scale <= 0.0 {
            return;
        }
        for vert in &mut self.vertices {
            let p = Vec3::from(vert.pos);
            vert.uv = [p.z.atan2(p.x) / std::f32::consts::TAU + 0.5, p.y / scale];
        }
        self.changed();
    }

    /// Spherical projection around the origin: U is longitude, V latitude (both 0..1). Good for planets and balls.
    pub fn project_spherical(&mut self) {
        for vert in &mut self.vertices {
            let p = Vec3::from(vert.pos).normalize_or_zero();
            vert.uv = [p.z.atan2(p.x) / std::f32::consts::TAU + 0.5, p.y.clamp(-1.0, 1.0).asin() / std::f32::consts::PI + 0.5];
        }
        self.changed();
    }

    /// Fits the UVs into the 0..1 square keeping their proportions, with a `margin` border (0..0.45).
    pub fn normalize_uv(&mut self, margin: f32) {
        if self.vertices.is_empty() {
            return;
        }
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        for v in &self.vertices {
            for k in 0..2 {
                lo[k] = lo[k].min(v.uv[k]);
                hi[k] = hi[k].max(v.uv[k]);
            }
        }
        let margin = margin.clamp(0.0, 0.45);
        let size = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-6);
        let k = (1.0 - 2.0 * margin) / size;
        for v in &mut self.vertices {
            v.uv = [(v.uv[0] - lo[0]) * k + margin, (v.uv[1] - lo[1]) * k + margin];
        }
        self.changed();
    }

    /// Scales, rotates (degrees) and offsets every UV around (0.5, 0.5).
    pub fn transform_uv(&mut self, scale: [f32; 2], rotation_degrees: f32, offset: [f32; 2]) {
        let (s, c) = rotation_degrees.to_radians().sin_cos();
        for v in &mut self.vertices {
            let (x, y) = ((v.uv[0] - 0.5) * scale[0], (v.uv[1] - 0.5) * scale[1]);
            v.uv = [c * x - s * y + 0.5 + offset[0], s * x + c * y + 0.5 + offset[1]];
        }
        self.changed();
    }

    /// Flat shading: every triangle gets its own face normal (vertices are split per triangle).
    pub fn flat_normals(&mut self) {
        let mut vertices = Vec::with_capacity(self.indices.len());
        for t in self.indices.chunks_exact(3) {
            let (a, b, c) = (self.vertices[t[0] as usize], self.vertices[t[1] as usize], self.vertices[t[2] as usize]);
            let n = (Vec3::from(b.pos) - Vec3::from(a.pos)).cross(Vec3::from(c.pos) - Vec3::from(a.pos)).normalize_or_zero().to_array();
            for mut v in [a, b, c] {
                v.normal = n;
                vertices.push(v);
            }
        }
        self.indices = (0..vertices.len() as u32).collect();
        self.vertices = vertices;
        self.changed();
    }

    /// Smooth shading with a crease angle: neighbouring faces closer than `angle_degrees` share a smooth
    /// normal, sharper edges stay hard. Vertices are split along creases.
    pub fn smooth_normals(&mut self, angle_degrees: f32) {
        let faces: Vec<(Vec3, [usize; 3])> = self
            .indices
            .chunks_exact(3)
            .map(|t| {
                let p = |i: u32| Vec3::from(self.vertices[i as usize].pos);
                ((p(t[1]) - p(t[0])).cross(p(t[2]) - p(t[0])), [t[0] as usize, t[1] as usize, t[2] as usize])
            })
            .collect();
        let mut at_point = HashMap::<[i64; 3], Vec<usize>>::new();
        for (f, (_, ids)) in faces.iter().enumerate() {
            for &i in ids {
                at_point.entry(key(self.vertices[i].pos)).or_default().push(f);
            }
        }
        let cos = angle_degrees.to_radians().cos();
        let mut vertices = Vec::with_capacity(self.indices.len());
        for (f, (normal, ids)) in faces.iter().enumerate() {
            let n0 = normal.normalize_or_zero();
            for &i in ids {
                let mut sum = Vec3::ZERO;
                for &other in &at_point[&key(self.vertices[i].pos)] {
                    if other == f || n0.dot(faces[other].0.normalize_or_zero()) >= cos {
                        sum += faces[other].0;
                    }
                }
                let mut v = self.vertices[i];
                v.normal = sum.normalize_or(if n0 == Vec3::ZERO { Vec3::from(v.normal) } else { n0 }).to_array();
                vertices.push(v);
            }
        }
        self.indices = (0..vertices.len() as u32).collect();
        self.vertices = vertices;
        self.changed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshData;

    fn primitive(name: &str) -> EditableMesh {
        EditableMesh::from_mesh(&MeshData::primitive(name).unwrap()).unwrap()
    }

    #[test]
    fn projections_stay_finite_and_normalize_into_the_unit_square() {
        let mut m = primitive("cube");
        m.project_planar(Vec3::Y, 2.0);
        m.normalize_uv(0.05);
        assert!(m.vertices.iter().all(|v| v.uv.iter().all(|c| (0.0..=1.0).contains(c))));
        m.project_spherical();
        m.project_cylindrical(1.0);
        m.transform_uv([2.0, 2.0], 45.0, [0.1, 0.0]);
        assert!(m.vertices.iter().all(|v| v.uv.iter().all(|c| c.is_finite())));
        m.project_planar(Vec3::ZERO, 1.0);
    }

    #[test]
    fn flat_and_crease_normals() {
        let mut m = primitive("cube");
        let faces = m.indices.len() / 3;
        m.flat_normals();
        assert_eq!(m.vertex_count(), faces * 3);
        let mut sphere = primitive("sphere");
        sphere.smooth_normals(60.0);
        assert!(sphere.vertices.iter().all(|v| (Vec3::from(v.normal).length() - 1.0).abs() < 1e-3));
        let mut hard = primitive("cube");
        hard.smooth_normals(30.0);
        assert!(hard.vertices.iter().all(|v| Vec3::from(v.normal).abs().max_element() > 0.99), "90 degree edges stay hard");
    }
}
