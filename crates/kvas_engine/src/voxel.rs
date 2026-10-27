//! Voxel effects built on one fast mesher: smoke that rises and swirls, and drifting clouds.
//!
//! Both produce a single `DynamicMesh` (one draw call). Hidden faces between touching voxels are
//! never emitted, so a large cloud costs only its visible surface.

use crate::components::*;
use crate::mesh::{MeshData, Vertex};
use crate::noise::{Noise, NoiseKind};
use crate::procedural::signature;
use crate::reflect::Field;
use glam::Vec3;
use kvas_ecs::World;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const FACES: [(Vec3, Vec3, Vec3); 6] = [
    (Vec3::X, Vec3::Y, Vec3::Z),
    (Vec3::NEG_X, Vec3::Z, Vec3::Y),
    (Vec3::Y, Vec3::Z, Vec3::X),
    (Vec3::NEG_Y, Vec3::X, Vec3::Z),
    (Vec3::Z, Vec3::X, Vec3::Y),
    (Vec3::NEG_Z, Vec3::Y, Vec3::X),
];

/// Meshes a voxel grid. `density(x, y, z)` > `threshold` means filled; `uv.x` carries the density.
/// With `shrink` every voxel is scaled by its density (soft puffs, all faces emitted); otherwise
/// full cubes are merged and faces between neighbours are culled.
pub fn mesh_grid(
    dim: [usize; 3],
    cell: f32,
    origin: Vec3,
    threshold: f32,
    shrink: bool,
    density: impl Fn(usize, usize, usize) -> f32,
) -> MeshData {
    let filled = |x: i64, y: i64, z: i64| -> f32 {
        if x < 0 || y < 0 || z < 0 || x >= dim[0] as i64 || y >= dim[1] as i64 || z >= dim[2] as i64 {
            return 0.0;
        }
        density(x as usize, y as usize, z as usize)
    };
    let mut m = MeshData::default();
    for z in 0..dim[2] as i64 {
        for y in 0..dim[1] as i64 {
            for x in 0..dim[0] as i64 {
                let d = filled(x, y, z);
                if d <= threshold {
                    continue;
                }
                let center = origin + (Vec3::new(x as f32, y as f32, z as f32) + 0.5) * cell;
                let half = cell * 0.5 * if shrink { 0.35 + 0.65 * d.clamp(0.0, 1.0) } else { 1.0 };
                for (n, u, v) in FACES {
                    if !shrink {
                        let o = n.as_ivec3();
                        if filled(x + o.x as i64, y + o.y as i64, z + o.z as i64) > threshold {
                            continue;
                        }
                    }
                    let base = m.vertices.len() as u32;
                    for (su, sv, uvc) in [(-1.0, -1.0, 0.0), (1.0, -1.0, 0.0), (1.0, 1.0, 0.0), (-1.0, 1.0, 0.0)] {
                        let _ = uvc;
                        let p = center + n * half + u * (half * su) + v * (half * sv);
                        m.vertices.push(Vertex { pos: p.into(), normal: n.into(), uv: [d.clamp(0.0, 1.0), 0.5] });
                    }
                    m.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
        }
    }
    m
}

// ------------------------------------------------------------------------------------ smoke

/// Voxel smoke emitter and simulation parameters. Runtime density is not serialized.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VoxelSmoke {
    /// Grid cells along each axis.
    pub grid: i32,
    /// Edge length of one cell in metres.
    pub cell: f32,
    /// Upward speed, cells per second.
    pub rise: f32,
    /// Strength of seeded lateral noise in the smoke velocity field.
    pub turbulence: f32,
    /// Density added at the source per second.
    pub emit: f32,
    /// Fraction of density lost per second.
    pub dissipation: f32,
    /// Minimum density at which a voxel contributes visible geometry.
    pub threshold: f32,
    /// Deterministic turbulence seed.
    pub seed: i32,
    #[serde(skip)]
    #[doc(hidden)]
    pub state: Option<SmokeState>,
    #[serde(skip)]
    #[doc(hidden)]
    pub sig: u64,
}

impl Default for VoxelSmoke {
    fn default() -> Self {
        Self {
            grid: 20,
            cell: 0.22,
            rise: 3.0,
            turbulence: 2.2,
            emit: 3.5,
            dissipation: 0.28,
            threshold: 0.07,
            seed: 3,
            state: None,
            sig: 0,
        }
    }
}

impl Component for VoxelSmoke {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::i32("grid", &mut self.grid),
            Field::f32("cell", &mut self.cell),
            Field::f32("rise", &mut self.rise),
            Field::f32("turbulence", &mut self.turbulence),
            Field::f32("emit", &mut self.emit),
            Field::f32("dissipation", &mut self.dissipation),
            Field::f32("threshold", &mut self.threshold),
            Field::i32("seed", &mut self.seed),
        ]
    }
}

/// Simulation state (internal).
#[doc(hidden)]
#[derive(Clone)]
pub struct SmokeState {
    n: usize,
    density: Vec<f32>,
    scratch: Vec<f32>,
    time: f32,
    version: u64,
}

impl SmokeState {
    fn at(&self, x: i64, y: i64, z: i64) -> f32 {
        let n = self.n as i64;
        if x < 0 || y < 0 || z < 0 || x >= n || y >= n || z >= n {
            return 0.0;
        }
        self.density[(z as usize * self.n + y as usize) * self.n + x as usize]
    }

    fn sample(&self, p: Vec3) -> f32 {
        let (f, i) = (p - p.floor(), p.floor());
        let (x, y, z) = (i.x as i64, i.y as i64, i.z as i64);
        let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
        l(
            l(l(self.at(x, y, z), self.at(x + 1, y, z), f.x), l(self.at(x, y + 1, z), self.at(x + 1, y + 1, z), f.x), f.y),
            l(l(self.at(x, y, z + 1), self.at(x + 1, y, z + 1), f.x), l(self.at(x, y + 1, z + 1), self.at(x + 1, y + 1, z + 1), f.x), f.y),
            f.z,
        )
    }

    /// One semi-Lagrangian step: every cell looks back along the flow to see what arrives here.
    fn step(&mut self, s: &VoxelSmoke, dt: f32) {
        let n = self.n;
        let noise = Noise::new(NoiseKind::Perlin).with_seed(s.seed as u32).with_octaves(2);
        self.time += dt;
        let t = self.time * 0.6;
        let source = (n as f32 * 0.5, 1.0, n as f32 * 0.5);
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    let p = Vec3::new(x as f32, y as f32, z as f32);
                    let flow = Vec3::new(
                        noise.sample3(p.x * 0.18, p.y * 0.18 - t, p.z * 0.18 + 5.0),
                        0.0,
                        noise.sample3(p.x * 0.18 + 9.0, p.y * 0.18 - t, p.z * 0.18),
                    ) * s.turbulence
                        + Vec3::Y * s.rise * (0.6 + 0.4 * (y as f32 / n as f32));
                    let back = p + Vec3::splat(0.5) - flow * dt;
                    let mut d = self.sample(back - Vec3::splat(0.5)) * (1.0 - s.dissipation * dt).max(0.0);
                    let dx = p.x + 0.5 - source.0;
                    let dz = p.z + 0.5 - source.2;
                    if y < 2 && dx * dx + dz * dz < (n as f32 * 0.12).powi(2) + 1.0 {
                        d += s.emit * dt * (1.0 - y as f32 * 0.4);
                    }
                    self.scratch[(z * n + y) * n + x] = d.clamp(0.0, 1.0);
                }
            }
        }
        std::mem::swap(&mut self.density, &mut self.scratch);
    }

    fn mesh(&self, s: &VoxelSmoke) -> MeshData {
        let n = self.n;
        let origin = Vec3::new(-(n as f32) * 0.5 * s.cell, 0.0, -(n as f32) * 0.5 * s.cell);
        mesh_grid([n, n, n], s.cell, origin, s.threshold, true, |x, y, z| self.density[(z * n + y) * n + x])
    }
}

/// Creates/rebuilds smoke meshes and optionally advances the density simulation by dt seconds.
pub fn update_smoke(world: &mut World, dt: f32, simulate: bool) {
    for e in world.entities_with::<VoxelSmoke>() {
        let s = world.get_mut::<VoxelSmoke>(e).unwrap();
        let sig = signature(&(s.grid, s.seed));
        if s.state.is_none() || s.sig != sig {
            let n = s.grid.clamp(6, 40) as usize;
            let mut st = SmokeState { n, density: vec![0.0; n * n * n], scratch: vec![0.0; n * n * n], time: 0.0, version: 0 };
            let params = s.clone();
            for _ in 0..90 {
                st.step(&params, 1.0 / 30.0); // pre-warm so the editor shows a plume right away
            }
            s.state = Some(st);
            s.sig = sig;
        }
        let mut st = s.state.take().unwrap();
        if simulate && dt > 0.0 {
            st.step(s, dt.min(0.05));
        }
        st.version += 1;
        let (mesh, version) = (st.mesh(s), st.version);
        s.state = Some(st);
        world.insert(e, DynamicMesh { data: Arc::new(mesh), version });
        if !world.has::<MeshRenderer>(e) {
            world.insert(e, MeshRenderer::default());
        }
        let m = world.get_mut::<MeshRenderer>(e).unwrap();
        m.shader = "builtin:smoke".into();
        m.cast_shadows = false;
        if m.color[3] > 0.95 {
            m.color = [0.78, 0.78, 0.8, 0.9];
        }
    }
}

// ------------------------------------------------------------------------------------ clouds

/// Volumetric clouds: a voxel density grid (generated from fractal noise) that the GPU ray-marches
/// with sun scattering and self-shadowing. The entity draws only an inverted bounding box.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VoxelClouds {
    /// Extent in metres (x, y, z).
    pub size: Vec3,
    /// Horizontal voxel size in metres (smaller = finer detail, bigger texture).
    pub cell: f32,
    /// 0..1, how much of the volume is cloud.
    pub coverage: f32,
    /// Noise features per 100 m.
    pub scale: f32,
    /// Deterministic density-field seed.
    pub seed: i32,
    /// Wind drift in metres per second (the density scrolls through the box).
    pub drift: Vec3,
    /// Opacity multiplier.
    pub density: f32,
    /// Bends light rays along the density gradient: brighter, glassier edges (0 = off).
    pub refraction: f32,
    #[serde(skip)]
    #[doc(hidden)]
    pub sig: u64,
}

impl Default for VoxelClouds {
    fn default() -> Self {
        Self {
            size: Vec3::new(240.0, 36.0, 240.0),
            cell: 3.5,
            coverage: 0.36,
            scale: 3.2,
            seed: 7,
            drift: Vec3::new(2.0, 0.0, 0.6),
            density: 0.8,
            refraction: 0.25,
            sig: 0,
        }
    }
}

impl Component for VoxelClouds {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::vec3("size", &mut self.size),
            Field::f32("cell", &mut self.cell),
            Field::f32("coverage", &mut self.coverage),
            Field::f32("scale", &mut self.scale),
            Field::i32("seed", &mut self.seed),
            Field::vec3("drift", &mut self.drift),
            Field::f32("density", &mut self.density),
            Field::f32("refraction", &mut self.refraction),
        ]
    }
}

/// Prefix for procedural three-dimensional cloud density texture keys.
pub const CLOUD_VOLUME_PREFIX: &str = "cloudvol:";

/// Describes the density grid of a cloud layer; its `key()` goes into `MeshRenderer.texture`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CloudVolume {
    /// Full world-space box extents in metres.
    pub size: Vec3,
    /// Horizontal density sampling interval in metres.
    pub cell: f32,
    /// Cloud occupancy fraction, clamped to 0..1 during generation.
    pub coverage: f32,
    /// Noise feature scale multiplier per 100 metres.
    pub scale: f32,
    /// Deterministic density-field seed.
    pub seed: i32,
}

impl CloudVolume {
    /// Copies density parameters from a cloud component.
    pub fn of(c: &VoxelClouds) -> Self {
        Self { size: c.size, cell: c.cell, coverage: c.coverage, scale: c.scale, seed: c.seed }
    }

    /// Encodes the density parameters as a renderer texture cache key.
    pub fn key(&self) -> String {
        format!(
            "{CLOUD_VOLUME_PREFIX}{},{},{},{},{},{},{}",
            self.size.x, self.size.y, self.size.z, self.cell, self.coverage, self.scale, self.seed
        )
    }

    /// Parses a cloud-volume cache key, or returns None for malformed input.
    pub fn parse(key: &str) -> Option<CloudVolume> {
        let n: Vec<&str> = key.strip_prefix(CLOUD_VOLUME_PREFIX)?.split(',').collect();
        if n.len() != 7 {
            return None;
        }
        let real = |i: usize| n[i].parse::<f32>().ok().filter(|v| v.is_finite());
        Some(CloudVolume {
            size: Vec3::new(real(0)?, real(1)?, real(2)?),
            cell: real(3)?,
            coverage: real(4)?,
            scale: real(5)?,
            seed: n[6].parse().ok()?,
        })
    }

    /// Grid resolution (x, y, z). Vertical resolution is finer than horizontal so cloud tops stay round.
    pub fn dims(&self) -> [u32; 3] {
        let cell = self.cell.max(0.5);
        [
            (self.size.x / cell).ceil().clamp(8.0, 160.0) as u32,
            (self.size.y / (cell * 0.5)).ceil().clamp(6.0, 64.0) as u32,
            (self.size.z / cell).ceil().clamp(8.0, 160.0) as u32,
        ]
    }

    /// 8-bit density per voxel, x fastest, then y, then z.
    pub fn generate(&self) -> ([u32; 3], Vec<u8>) {
        let dim = self.dims();
        let base = Noise::new(NoiseKind::Perlin).with_seed(self.seed as u32).with_octaves(5);
        let erode = Noise::new(NoiseKind::Worley).with_seed((self.seed as u32).wrapping_add(1)).with_octaves(2);
        let freq = self.scale / 100.0;
        let threshold = 1.0 - self.coverage.clamp(0.0, 1.0);
        let mut out = Vec::with_capacity((dim[0] * dim[1] * dim[2]) as usize);
        for z in 0..dim[2] {
            for y in 0..dim[1] {
                for x in 0..dim[0] {
                    let (fx, fy, fz) = (x as f32 / dim[0] as f32, y as f32 / dim[1] as f32, z as f32 / dim[2] as f32);
                    let p = Vec3::new(fx * self.size.x, fy * self.size.y, fz * self.size.z) * freq;
                    // Flat bottoms, billowing tops.
                    let bottom = (fy / 0.18).clamp(0.0, 1.0);
                    let top = (1.0 - (fy - 0.45) / 0.55).clamp(0.0, 1.0).powf(0.8);
                    let shape = bottom * top;
                    let n = base.sample01(p.x, p.y * 1.7, p.z) * 0.8 + erode.sample01(p.x * 3.1, p.y * 3.1, p.z * 3.1) * 0.2;
                    let d = ((n * shape - threshold * 0.55) * 2.6).clamp(0.0, 1.0);
                    out.push((d * 255.0) as u8);
                }
            }
        }
        (dim, out)
    }
}

/// Inverted box mesh: its inside faces face the camera whether it is outside or inside the volume.
fn inverted_box(half: Vec3) -> MeshData {
    let mut m = MeshData::primitive("cube").unwrap_or_default();
    for v in &mut m.vertices {
        v.pos = (Vec3::from(v.pos) * half * 2.0).into();
    }
    for tri in m.indices.as_chunks_mut::<3>().0.iter_mut() {
        tri.swap(1, 2);
    }
    m
}

/// Refreshes cloud proxy geometry and shader parameters; drift advances only while simulating.
pub fn update_clouds(world: &mut World, dt: f32, simulate: bool) {
    let _ = (dt, simulate);
    for e in world.entities_with::<VoxelClouds>() {
        let c = world.get::<VoxelClouds>(e).unwrap().clone();
        let volume = CloudVolume::of(&c);
        let sig = signature(&(c.size.to_array(), c.cell.to_bits(), c.coverage.to_bits(), c.scale.to_bits(), c.seed));
        if world.get::<VoxelClouds>(e).unwrap().sig != sig || !world.has::<DynamicMesh>(e) {
            world.insert(e, DynamicMesh { data: Arc::new(inverted_box(c.size * 0.5)), version: sig });
            world.get_mut::<VoxelClouds>(e).unwrap().sig = sig;
        }
        if !world.has::<MeshRenderer>(e) {
            world.insert(e, MeshRenderer::default());
        }
        let m = world.get_mut::<MeshRenderer>(e).unwrap();
        m.shader = "builtin:clouds".into();
        m.texture = volume.key();
        m.cast_shadows = false;
        m.color = [1.0, 1.0, 1.0, 0.9]; // below 1 so the box is drawn in the transparent pass
        let half = c.size * 0.5;
        m.user = [half.x, half.y, half.z, c.density, c.drift.x, c.drift.y, c.drift.z, c.refraction];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_voxel_is_a_cube_and_touching_voxels_hide_their_shared_faces() {
        let one = mesh_grid([1, 1, 1], 1.0, Vec3::ZERO, 0.5, false, |_, _, _| 1.0);
        assert_eq!(one.indices.len() / 3, 12);
        let two = mesh_grid([2, 1, 1], 1.0, Vec3::ZERO, 0.5, false, |_, _, _| 1.0);
        assert_eq!(two.indices.len() / 3, 20, "10 visible quads, the shared pair is culled");
    }

    #[test]
    fn solid_block_only_pays_for_its_surface() {
        let n = 10;
        let m = mesh_grid([n, n, n], 1.0, Vec3::ZERO, 0.5, false, |_, _, _| 1.0);
        assert_eq!(m.indices.len() / 3, 6 * n * n * 2, "only the outer shell is meshed");
    }

    #[test]
    fn smoke_rises_and_fades() {
        let mut s = VoxelSmoke::default();
        s.dissipation = 0.5;
        let mut st = SmokeState { n: 20, density: vec![0.0; 8000], scratch: vec![0.0; 8000], time: 0.0, version: 0 };
        for _ in 0..180 {
            st.step(&s, 1.0 / 30.0);
        }
        let n = st.n;
        let mass_at = |y0: usize, y1: usize| {
            (0..n)
                .flat_map(|z| (y0..y1).flat_map(move |y| (0..n).map(move |x| (x, y, z))))
                .map(|(x, y, z)| st.density[(z * n + y) * n + x])
                .sum::<f32>()
        };
        assert!(mass_at(0, 4) > 1.0, "smoke exists near the source");
        assert!(mass_at(8, 20) > 0.5, "and has risen into the upper grid");
        assert!(st.density.iter().all(|d| (0.0..=1.0).contains(d)));
    }

    #[test]
    fn cloud_volume_has_clouds_and_gaps_and_roundtrips_its_key() {
        let vol = CloudVolume::of(&VoxelClouds { size: Vec3::new(120.0, 30.0, 120.0), cell: 6.0, ..Default::default() });
        assert_eq!(CloudVolume::parse(&vol.key()), Some(vol));
        let (dim, data) = vol.generate();
        assert_eq!(data.len(), (dim[0] * dim[1] * dim[2]) as usize);
        let filled = data.iter().filter(|&&d| d > 40).count() as f32 / data.len() as f32;
        assert!(filled > 0.03 && filled < 0.7, "a mix of cloud and clear sky: {filled}");
        // Cloud bases are flat: the lowest layer is nearly empty, the middle is denser.
        let layer = |y: u32| {
            (0..dim[2])
                .flat_map(|z| (0..dim[0]).map(move |x| (x, z)))
                .map(|(x, z)| data[((z * dim[1] + y) * dim[0] + x) as usize] as u32)
                .sum::<u32>()
        };
        assert!(layer(0) < layer(dim[1] / 3), "bottom layer is thinner than the body");
    }

    #[test]
    fn cloud_box_faces_inwards() {
        let m = inverted_box(Vec3::new(5.0, 2.0, 5.0));
        let tri = &m.indices[0..3];
        let p = |i: u32| Vec3::from(m.vertices[i as usize].pos);
        let n = (p(tri[1]) - p(tri[0])).cross(p(tri[2]) - p(tri[0]));
        let center_to_face = p(tri[0]);
        assert!(n.dot(center_to_face) < 0.0, "winding faces the centre of the box");
    }
}
