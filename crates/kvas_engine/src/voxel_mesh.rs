//! Greedy solid-voxel mesher; density discontinuities retain separate faces.
use crate::mesh::{MeshData, Vertex};
use glam::Vec3;

/// Merges coplanar exposed faces with equal density. Each cell is sampled once.
/// A solid cuboid becomes six quads. Invalid parameters or >16M cells return an empty mesh.
pub fn mesh_grid_greedy(dim: [usize; 3], cell: f32, origin: Vec3, threshold: f32, density: impl Fn(usize, usize, usize) -> f32) -> MeshData {
    let mut mesh = MeshData::default();
    let count = dim.iter().try_fold(1usize, |n, &d| n.checked_mul(d));
    if dim.iter().any(|&d| d == 0 || d > 512) || count.is_none_or(|n| n > 16_000_000) || !cell.is_finite() || cell <= 0.0 || !origin.is_finite() || !threshold.is_finite() { return mesh; }
    let mut values = Vec::with_capacity(count.unwrap());
    for z in 0..dim[2] { for y in 0..dim[1] { for x in 0..dim[0] { values.push(density(x, y, z)); } } }
    let sample = |p: [usize; 3]| values[(p[2] * dim[1] + p[1]) * dim[0] + p[0]];
    for axis in 0..3 {
        let u = (axis + 1) % 3; let v = (axis + 2) % 3;
        let mut mask = vec![None::<(bool, u32)>; dim[u] * dim[v]];
        for plane in 0..=dim[axis] {
            for j in 0..dim[v] { for i in 0..dim[u] {
                let mut p = [0; 3]; p[u] = i; p[v] = j;
                let left = if plane > 0 { p[axis] = plane - 1; Some(sample(p)).filter(|d| d.is_finite() && *d > threshold) } else { None };
                let right = if plane < dim[axis] { p[axis] = plane; Some(sample(p)).filter(|d| d.is_finite() && *d > threshold) } else { None };
                mask[j * dim[u] + i] = match (left, right) { (Some(d), None) => Some((true, d.clamp(0.0, 1.0).to_bits())), (None, Some(d)) => Some((false, d.clamp(0.0, 1.0).to_bits())), _ => None };
            } }
            for j in 0..dim[v] {
                let mut i = 0;
                while i < dim[u] {
                    let Some(key) = mask[j * dim[u] + i] else { i += 1; continue };
                    let mut width = 1;
                    while i + width < dim[u] && mask[j * dim[u] + i + width] == Some(key) { width += 1; }
                    let mut height = 1;
                    while j + height < dim[v] && (0..width).all(|x| mask[(j + height) * dim[u] + i + x] == Some(key)) { height += 1; }
                    for y in j..j + height { for x in i..i + width { mask[y * dim[u] + x] = None; } }
                    let mut p = Vec3::ZERO; p[axis] = plane as f32; p[u] = i as f32; p[v] = j as f32;
                    let mut du = Vec3::ZERO; du[u] = width as f32;
                    let mut dv = Vec3::ZERO; dv[v] = height as f32;
                    let mut normal = Vec3::ZERO; normal[axis] = if key.0 { 1.0 } else { -1.0 };
                    let base = mesh.vertices.len() as u32;
                    for q in [p, p + du, p + du + dv, p + dv] { mesh.vertices.push(Vertex { pos: (origin + q * cell).to_array(), normal: normal.to_array(), uv: [f32::from_bits(key.1), 0.5] }); }
                    if key.0 { mesh.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]); }
                    else { mesh.indices.extend([base, base + 2, base + 1, base, base + 3, base + 2]); }
                    i += width;
                }
            }
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    fn area(mesh: &MeshData) -> f32 {
        mesh.indices.chunks_exact(3).map(|t| {
            let a = Vec3::from(mesh.vertices[t[0] as usize].pos); let b = Vec3::from(mesh.vertices[t[1] as usize].pos); let c = Vec3::from(mesh.vertices[t[2] as usize].pos);
            let cross = (b - a).cross(c - a);
            assert!(cross.dot(Vec3::from(mesh.vertices[t[0] as usize].normal)) > 0.0);
            cross.length() * 0.5
        }).sum()
    }
    #[test]
    fn greedy_cavity_density_winding_and_sampling_are_preserved() {
        let calls = std::cell::Cell::new(0usize);
        let mesh = mesh_grid_greedy([3, 3, 3], 1.0, Vec3::ZERO, 0.1, |x, y, z| {
            calls.set(calls.get() + 1);
            if x == 1 && y == 1 && z == 1 { 0.0 } else if x == 0 { 0.5 } else { 1.0 }
        });
        assert_eq!(calls.get(), 27); mesh.validate().unwrap();
        assert_eq!(area(&mesh), 60.0, "outer shell and internal cavity");
        assert!(mesh.vertices.iter().any(|v| v.uv[0] == 0.5));
        let solid = mesh_grid_greedy([32, 32, 32], 1.0, Vec3::ZERO, 0.1, |_, _, _| 1.0);
        assert_eq!(solid.indices.len(), 36); assert_eq!(area(&solid), 6.0 * 32.0 * 32.0);
        assert!(mesh_grid_greedy([usize::MAX, 2, 2], 1.0, Vec3::ZERO, 0.5, |_, _, _| 1.0).vertices.is_empty());
    }
}
