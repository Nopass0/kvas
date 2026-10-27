//! Voxel water: an animated water surface built from wave functions, drawn either as stacked
//! voxel columns (blocky, Minecraft-like) or as a smooth semi-realistic sheet.
//!
//! The surface is one `DynamicMesh` rebuilt every frame from a handful of travelling waves plus
//! noise ripples, so it costs a single draw call. Transparency, reflections and colour come from
//! the entity's `MeshRenderer` (set sensibly the first time the component is added).
//!
//! ```
//! use kvas_engine::{prelude::*, scene::Scene, water::VoxelWater};
//! let mut scene = Scene::default();
//! let lake = scene.spawn("Lake");
//! scene.world.insert(lake, VoxelWater::default());
//! kvas_engine::water::update_water(&mut scene.world, 1.0);
//! assert!(scene.world.has::<DynamicMesh>(lake));
//! ```

use crate::components::*;
use crate::mesh::{MeshData, Vertex};
use crate::noise::{Noise, NoiseKind};
use crate::reflect::Field;
use glam::{Mat4, Vec3};
use kvas_ecs::World;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

crate::choice_enum! {
    /// How the water surface is shaped: stepped voxel columns, smooth, or smooth with sharp crests.
    WaterStyle { Voxel, Smooth, Realistic } default Smooth
}

/// An animated body of water centred on the entity.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VoxelWater {
    /// Look of the surface.
    pub style: WaterStyle,
    /// Cells along X and Z (the surface is `cells * cell` metres wide).
    pub cells: i32,
    /// Edge length of one cell in metres.
    pub cell: f32,
    /// Wave height (crest to rest level) in metres.
    pub wave_height: f32,
    /// How fast waves travel, 1 = normal.
    pub speed: f32,
    /// Distance between wave crests in metres.
    pub wavelength: f32,
    /// Compass direction the waves travel towards, in degrees.
    pub direction: f32,
    /// Small noise ripples on top of the main waves, 0..1.
    pub ripples: f32,
    /// Water depth in metres below the surface (voxel style draws the block sides).
    pub depth: f32,
    /// Deterministic ripple seed.
    pub seed: i32,
    /// Whether rigid bodies float in this water.
    pub physics: bool,
    /// Lift relative to weight when fully submerged: 1 hovers, above 1 floats up, below 1 sinks slowly.
    pub buoyancy: f32,
    /// Water resistance slowing submerged bodies.
    pub drag: f32,
}

impl Default for VoxelWater {
    fn default() -> Self {
        Self { style: WaterStyle::Smooth, cells: 48, cell: 0.5, wave_height: 0.18, speed: 1.0, wavelength: 6.0, direction: 30.0, ripples: 0.5, depth: 1.5, seed: 11, physics: true, buoyancy: 1.7, drag: 2.5 }
    }
}

impl Component for VoxelWater {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::choice("style", &mut self.style),
            Field::i32("cells", &mut self.cells),
            Field::f32("cell", &mut self.cell),
            Field::f32("wave_height", &mut self.wave_height),
            Field::f32("speed", &mut self.speed),
            Field::f32("wavelength", &mut self.wavelength),
            Field::f32("direction", &mut self.direction),
            Field::f32("ripples", &mut self.ripples),
            Field::f32("depth", &mut self.depth),
            Field::i32("seed", &mut self.seed),
            Field::bool("physics", &mut self.physics),
            Field::f32("buoyancy", &mut self.buoyancy),
            Field::f32("drag", &mut self.drag),
        ]
    }
}

impl VoxelWater {
    /// Surface height (metres above the rest level) at a local position and time.
    pub fn height_at(&self, noise: &Noise, x: f32, z: f32, t: f32) -> f32 {
        let k = std::f32::consts::TAU / self.wavelength.max(0.5);
        let dir = self.direction.to_radians();
        let main = Vec3::new(dir.cos(), 0.0, dir.sin());
        let crest = if self.style == WaterStyle::Realistic { 1.35 } else { 1.0 };
        let mut h = 0.0;
        // Four waves around the main direction with different lengths, speeds and phases.
        for (i, (rot, len, amp)) in [(0.0, 1.0, 1.0), (0.45, 0.62, 0.55), (-0.7, 0.4, 0.3), (1.1, 0.27, 0.16)].into_iter().enumerate() {
            let a = dir + rot;
            let d = Vec3::new(a.cos(), 0.0, a.sin());
            let kk = k / len;
            let w = (9.81f32 * kk).sqrt() * self.speed;
            let phase = kk * (d.x * x + d.z * z) - w * t + i as f32 * 1.7;
            // Sharpen crests for the realistic look: |sin| shaped peaks and flatter troughs.
            let s = phase.sin();
            let shaped = if crest > 1.0 { 1.0 - (1.0 - (s * 0.5 + 0.5)).powf(crest) * 2.0 } else { s };
            h += shaped * amp;
        }
        let _ = main;
        h *= self.wave_height / 1.8;
        h + noise.sample3(x * 0.7, z * 0.7, t * 0.6 * self.speed) * self.wave_height * self.ripples * 0.35
    }

    /// Builds the surface mesh for time `t`.
    pub fn build_mesh(&self, t: f32) -> MeshData {
        let n = self.cells.clamp(4, 160) as usize;
        let cell = self.cell.max(0.05);
        let noise = Noise::new(NoiseKind::Perlin).with_seed(self.seed as u32).with_octaves(2);
        let half = n as f32 * cell * 0.5;
        let mut m = MeshData::default();
        match self.style {
            WaterStyle::Voxel => {
                // One block height step = a fraction of the wave height, so the stairs stay visible.
                let step = (self.wave_height * 0.5).max(cell * 0.5);
                let heights: Vec<f32> = (0..n * n)
                    .map(|i| {
                        let (cx, cz) = ((i % n) as f32 + 0.5, (i / n) as f32 + 0.5);
                        let h = self.height_at(&noise, cx * cell - half, cz * cell - half, t);
                        (h / step).round() * step
                    })
                    .collect();
                let at = |x: i64, z: i64| -> Option<f32> {
                    (x >= 0 && z >= 0 && x < n as i64 && z < n as i64).then(|| heights[z as usize * n + x as usize])
                };
                let bottom = -self.depth.max(cell);
                for z in 0..n as i64 {
                    for x in 0..n as i64 {
                        let h = heights[z as usize * n + x as usize];
                        let (x0, x1) = (x as f32 * cell - half, (x + 1) as f32 * cell - half);
                        let (z0, z1) = (z as f32 * cell - half, (z + 1) as f32 * cell - half);
                        quad(&mut m, [Vec3::new(x0, h, z0), Vec3::new(x0, h, z1), Vec3::new(x1, h, z1), Vec3::new(x1, h, z0)], Vec3::Y, h);
                        // Side walls where the neighbour is lower or missing (outer edge goes down to the bottom).
                        for (dx, dz, nrm) in [(1, 0, Vec3::X), (-1, 0, Vec3::NEG_X), (0, 1, Vec3::Z), (0, -1, Vec3::NEG_Z)] {
                            let low = at(x + dx, z + dz).unwrap_or(bottom);
                            if low >= h {
                                continue;
                            }
                            let (a, b) = match (dx, dz) {
                                (1, 0) => (Vec3::new(x1, 0.0, z1), Vec3::new(x1, 0.0, z0)),
                                (-1, 0) => (Vec3::new(x0, 0.0, z0), Vec3::new(x0, 0.0, z1)),
                                (0, 1) => (Vec3::new(x1, 0.0, z1), Vec3::new(x0, 0.0, z1)),
                                _ => (Vec3::new(x0, 0.0, z0), Vec3::new(x1, 0.0, z0)),
                            };
                            quad(&mut m, [a.with_y(low), b.with_y(low), b.with_y(h), a.with_y(h)], nrm, h);
                        }
                    }
                }
            }
            WaterStyle::Smooth | WaterStyle::Realistic => {
                let stride = n + 1;
                // One height evaluation per vertex (normals come from neighbouring heights), spread
                // over the CPU cores in bands of rows.
                let mut heights = vec![0.0f32; stride * stride];
                let workers = std::thread::available_parallelism().map_or(1, |p| p.get()).min(8);
                let rows_per = stride.div_ceil(if n >= 64 { workers } else { 1 });
                std::thread::scope(|scope| {
                    for (band, chunk) in heights.chunks_mut(rows_per * stride).enumerate() {
                        let noise = &noise;
                        scope.spawn(move || {
                            for (r, row) in chunk.chunks_mut(stride).enumerate() {
                                let z = (band * rows_per + r) as f32 * cell - half;
                                for (x, h) in row.iter_mut().enumerate() {
                                    *h = self.height_at(noise, x as f32 * cell - half, z, t);
                                }
                            }
                        });
                    }
                });
                let at = |x: usize, z: usize| heights[z.min(n) * stride + x.min(n)];
                m.vertices.reserve(stride * stride);
                for z in 0..=n {
                    for x in 0..=n {
                        let (px, pz) = (x as f32 * cell - half, z as f32 * cell - half);
                        let dx = (at(x + 1, z) - at(x.saturating_sub(1), z)) / (2.0 * cell);
                        let dz = (at(x, z + 1) - at(x, z.saturating_sub(1))) / (2.0 * cell);
                        let normal = Vec3::new(-dx, 1.0, -dz).normalize();
                        m.vertices.push(Vertex { pos: [px, at(x, z), pz], normal: normal.into(), uv: [px * 0.1, pz * 0.1] });
                    }
                }
                for z in 0..n {
                    for x in 0..n {
                        let i = (z * stride + x) as u32;
                        let s = stride as u32;
                        m.indices.extend([i, i + s, i + 1, i + 1, i + s, i + s + 1]);
                    }
                }
            }
        }
        m
    }
}

fn quad(m: &mut MeshData, corners: [Vec3; 4], normal: Vec3, tag: f32) {
    let base = m.vertices.len() as u32;
    for (i, c) in corners.iter().enumerate() {
        m.vertices.push(Vertex { pos: (*c).into(), normal: normal.into(), uv: [tag, i as f32] });
    }
    m.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Rebuilds every water surface for time `time` (seconds). Cheap enough to run every frame.
pub fn update_water(world: &mut World, time: f32) {
    for e in world.entities_with::<VoxelWater>() {
        let Some(water) = world.get::<VoxelWater>(e).cloned() else { continue };
        let version = world.get::<DynamicMesh>(e).map_or(0, |d| d.version) + 1;
        world.insert(e, DynamicMesh { data: Arc::new(water.build_mesh(time)), version });
        let fresh = !world.has::<MeshRenderer>(e);
        if fresh {
            world.insert(e, MeshRenderer::default());
        }
        let m = world.get_mut::<MeshRenderer>(e).unwrap();
        m.cast_shadows = false;
        // Smooth and realistic water use the built-in water shader unless the user picked another one.
        if water.style != WaterStyle::Voxel && (m.shader.is_empty() || m.shader == "builtin:water") {
            m.shader = "builtin:water".into();
        } else if water.style == WaterStyle::Voxel && m.shader == "builtin:water" {
            m.shader.clear();
        }
        if fresh || m.color == MeshRenderer::default().color {
            m.color = match water.style {
                WaterStyle::Voxel => [0.2, 0.55, 0.85, 0.72],
                WaterStyle::Smooth => [0.12, 0.42, 0.6, 0.62],
                WaterStyle::Realistic => [0.05, 0.28, 0.42, 0.55],
            };
            m.roughness = 0.04;
            m.reflectivity = if water.style == WaterStyle::Realistic { 0.85 } else { 0.55 };
        }
    }
}

/// Floats rigid bodies in water: each dynamic body gets an upward force proportional to how much of
/// it is under the moving surface, plus water drag. Call once per fixed physics step.
pub fn apply_buoyancy(world: &mut World, time: f32, dt: f32) {
    use crate::hierarchy;
    let waters: Vec<(Mat4, VoxelWater)> = world
        .entities_with::<VoxelWater>()
        .into_iter()
        .filter_map(|e| Some((hierarchy::global_matrix(world, e), world.get::<VoxelWater>(e).filter(|w| w.physics)?.clone())))
        .collect();
    if waters.is_empty() {
        return;
    }
    for e in world.entities_with::<RigidBody>() {
        let (mass, velocity) = match world.get::<RigidBody>(e) {
            Some(b) if b.kind == BodyKind::Dynamic => (b.mass.max(0.001), b.velocity),
            _ => continue,
        };
        let m = hierarchy::global_matrix(world, e);
        let pos = m.w_axis.truncate();
        let scale = m.to_scale_rotation_translation().0;
        let radius = world.get::<Collider>(e).map_or(0.5, |c| c.size.max_element().max(0.05) * 0.5 * scale.max_element());
        for (wm, water) in &waters {
            let local = wm.inverse().transform_point3(pos);
            let half = water.cells.clamp(4, 160) as f32 * water.cell.max(0.05) * 0.5;
            if local.x.abs() > half || local.z.abs() > half {
                continue;
            }
            let noise = Noise::new(NoiseKind::Perlin).with_seed(water.seed as u32).with_octaves(2);
            let surface = wm.w_axis.y + water.height_at(&noise, local.x, local.z, time);
            let depth = surface - (pos.y - radius);
            if depth <= 0.0 {
                continue;
            }
            let fraction = (depth / (2.0 * radius)).clamp(0.0, 1.0);
            if let Some(body) = world.get_mut::<RigidBody>(e) {
                body.impulse.y += mass * 9.81 * fraction * water.buoyancy * dt;
                // Water resistance, stronger the deeper the body sits; keeps floating objects from bouncing forever.
                let k = (water.drag * fraction * dt).min(0.9);
                body.impulse -= velocity * mass * k;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{runtime::Runtime, scene::Scene};

    #[test]
    fn bodies_float_instead_of_sinking() {
        let mut scene = Scene::default();
        let lake = scene.spawn("Lake");
        scene.world.insert(lake, VoxelWater { wave_height: 0.05, ..Default::default() });
        let ball = scene.spawn("Ball");
        scene.world.insert(ball, Transform::at(Vec3::new(0.0, 2.0, 0.0)));
        scene.world.insert(ball, Collider { shape: ShapeKind::Sphere, ..Default::default() });
        scene.world.insert(ball, RigidBody { mass: 1.0, ..Default::default() });
        let mut runtime = Runtime::new(scene, None);
        for _ in 0..360 {
            runtime.tick(1.0 / 60.0);
        }
        let y = runtime.scene.world.get::<Transform>(ball).unwrap().position.y;
        assert!(y > -0.6 && y < 1.0, "ball should bob near the surface, was at {y}");
    }
}
