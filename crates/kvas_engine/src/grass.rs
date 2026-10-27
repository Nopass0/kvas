//! Procedural grass. The patch is split into square chunks; each chunk is one merged mesh with its
//! own bounds, so the renderer frustum-culls whole chunks. Wind animation and distance thinning
//! happen on the GPU in `builtin:grass`; the meshes are built once.
//!
//! Chunks are `Transient` child entities: they never appear in the hierarchy or in saved scenes.

use crate::components::*;
use crate::hierarchy;
use crate::mesh::{MeshData, Vertex};
use crate::procedural::{signature, Rng};
use crate::reflect::Field;
use glam::Vec3;
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Seeded grass patch split into transient mesh chunks for frustum culling.
/// Wind is animated by the grass shader; geometry is rebuilt when parameters change.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Grass {
    /// Blades in the whole patch.
    pub count: i32,
    /// Patch size in metres, centred on the entity.
    pub width: f32,
    /// Patch depth in metres along the local Z axis.
    pub depth: f32,
    /// Mean blade height in metres.
    pub blade_height: f32,
    /// Relative randomized variation of blade height.
    pub height_variation: f32,
    /// Blade base width in metres.
    pub blade_width: f32,
    /// Resting lean of the blades.
    pub lean: f32,
    /// Deterministic patch seed; identical parameters produce identical blades.
    pub seed: i32,
    /// RGBA tint at the blade base.
    pub base_color: [f32; 4],
    /// RGBA tint at the blade tip.
    pub tip_color: [f32; 4],
    /// Shader wind displacement multiplier.
    pub wind_strength: f32,
    /// Shader wind animation speed multiplier.
    pub wind_speed: f32,
    /// Chunk edge in metres: smaller chunks cull better, larger ones need fewer draw calls.
    pub chunk_size: f32,
    /// Distance where blades start thinning out, and where the grass is gone.
    pub fade_start: f32,
    /// World-space distance where all grass blades have faded out.
    pub fade_end: f32,
    /// 0..1: patchy dry / lush colour variation across the meadow.
    pub patchiness: f32,
    #[serde(skip)]
    #[doc(hidden)]
    pub sig: u64,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            count: 40_000,
            width: 30.0,
            depth: 30.0,
            blade_height: 0.55,
            height_variation: 0.6,
            blade_width: 0.085,
            lean: 0.3,
            seed: 1,
            base_color: [0.04, 0.2, 0.1, 1.0],
            tip_color: [0.72, 0.9, 0.3, 1.0],
            wind_strength: 0.5,
            wind_speed: 1.5,
            chunk_size: 8.0,
            fade_start: 28.0,
            fade_end: 70.0,
            patchiness: 0.7,
            sig: 0,
        }
    }
}

impl Component for Grass {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::i32("count", &mut self.count),
            Field::f32("width", &mut self.width),
            Field::f32("depth", &mut self.depth),
            Field::f32("blade_height", &mut self.blade_height),
            Field::f32("height_variation", &mut self.height_variation),
            Field::f32("blade_width", &mut self.blade_width),
            Field::f32("lean", &mut self.lean),
            Field::i32("seed", &mut self.seed),
            Field::color("base_color", &mut self.base_color),
            Field::color("tip_color", &mut self.tip_color),
            Field::f32("patchiness", &mut self.patchiness),
            Field::f32("wind_strength", &mut self.wind_strength),
            Field::f32("wind_speed", &mut self.wind_speed),
            Field::f32("chunk_size", &mut self.chunk_size),
            Field::f32("fade_start", &mut self.fade_start),
            Field::f32("fade_end", &mut self.fade_end),
        ]
    }
}

/// Vertices per blade: four rows of two plus the tip.
pub const BLADE_VERTICES: usize = 9;
/// Triangles per blade, both sides.
pub const BLADE_TRIANGLES: usize = 14;

/// Grid of chunks covering the patch: `(columns, rows)`.
pub fn chunk_grid(g: &Grass) -> (usize, usize) {
    let size = g.chunk_size.max(1.0);
    ((g.width / size).ceil().clamp(1.0, 64.0) as usize, (g.depth / size).ceil().clamp(1.0, 64.0) as usize)
}

/// Builds the blades of chunk `(ix, iz)`; vertices are relative to the chunk centre.
pub fn chunk_mesh(g: &Grass, ix: usize, iz: usize) -> (MeshData, Vec3) {
    let (nx, nz) = chunk_grid(g);
    let (cw, cd) = (g.width / nx as f32, g.depth / nz as f32);
    let center = Vec3::new(-g.width * 0.5 + (ix as f32 + 0.5) * cw, 0.0, -g.depth * 0.5 + (iz as f32 + 0.5) * cd);
    let share = (cw * cd) / (g.width * g.depth).max(1e-3);
    let count = ((g.count.clamp(1, 400_000) as f32 * share).round() as usize).max(1);
    let mut rng = Rng::new((g.seed as u64) << 20 ^ (ix as u64) << 8 ^ iz as u64);
    let mut m = MeshData::default();
    m.vertices.reserve(count * BLADE_VERTICES);
    m.indices.reserve(count * BLADE_TRIANGLES * 3);
    for _ in 0..count {
        let (x, z) = ((rng.f32() - 0.5) * cw, (rng.f32() - 0.5) * cd);
        let yaw = rng.f32() * std::f32::consts::TAU;
        let tint = rng.f32();
        // A few tall tufts break the uniform look.
        let tuft = if rng.f32() > 0.93 { 1.7 } else { 1.0 };
        let h = g.blade_height * tuft * (1.0 + g.height_variation * (rng.f32() * 2.0 - 1.0)).max(0.15);
        let lean = g.lean * (0.3 + rng.f32());
        let (sin, cos) = yaw.sin_cos();
        let (right, forward) = (Vec3::new(cos, 0.0, -sin), Vec3::new(sin, 0.0, cos));
        let root = Vec3::new(x, 0.0, z);
        let base = m.vertices.len() as u32;
        // Blade outline: widest above the root, tapering to a curved tip.
        for (t, width) in [(0.0f32, 0.85f32), (0.28, 1.0), (0.58, 0.72), (0.84, 0.38)] {
            let curve = lean * h * t * t;
            let center = root + Vec3::Y * (h * t * (1.0 - 0.12 * t)) + forward * curve;
            let half = g.blade_width * 0.5 * width;
            let normal = (Vec3::Y * 2.0 + forward * 0.5).normalize();
            for side in [-1.0f32, 1.0] {
                m.vertices.push(Vertex { pos: (center + right * half * side).into(), normal: normal.into(), uv: [tint, t] });
            }
        }
        let tip = root + Vec3::Y * h * 0.9 + forward * (lean * h * 1.05);
        m.vertices.push(Vertex { pos: tip.into(), normal: Vec3::Y.into(), uv: [tint, 1.0] });
        for (a, b, c) in [(0, 1, 2), (1, 3, 2), (2, 3, 4), (3, 5, 4), (4, 5, 6), (5, 7, 6), (6, 7, 8)] {
            m.indices.extend([base + a, base + b, base + c]);
            m.indices.extend([base + c, base + b, base + a]);
        }
    }
    (m, center)
}

/// Rebuilds changed grass patches and their transient mesh chunks.
pub fn update(world: &mut World) {
    // Chunks whose patch is gone.
    let orphans: Vec<Entity> = world
        .entities_with::<GrassChunk>()
        .into_iter()
        .filter(|&c| hierarchy::parent_of(world, c).is_none_or(|p| !world.is_alive(p) || !world.has::<Grass>(p)))
        .collect();
    for c in orphans {
        world.despawn(c);
    }
    for e in world.entities_with::<Grass>() {
        // Older scenes stored the mesh on the grass entity itself.
        world.remove::<DynamicMesh>(e);
        world.remove::<MeshRenderer>(e);
        let g = world.get::<Grass>(e).unwrap().clone();
        let sig = signature(&(
            g.count,
            g.width.to_bits(),
            g.depth.to_bits(),
            g.blade_height.to_bits(),
            g.height_variation.to_bits(),
            g.blade_width.to_bits(),
            g.lean.to_bits(),
            g.seed,
            g.chunk_size.to_bits(),
        ));
        let chunks: Vec<Entity> = hierarchy::children_all(world, e).into_iter().filter(|&c| world.has::<GrassChunk>(c)).collect();
        if world.get::<Grass>(e).unwrap().sig != sig || chunks.is_empty() {
            for c in &chunks {
                world.despawn(*c);
            }
            let (nx, nz) = chunk_grid(&g);
            for iz in 0..nz {
                for ix in 0..nx {
                    let (mesh, center) = chunk_mesh(&g, ix, iz);
                    let c = world.spawn();
                    world.insert(c, Name("Grass chunk".into()));
                    world.insert(c, Transform::at(center));
                    world.insert(c, Parent(e));
                    world.insert(c, Transient);
                    world.insert(c, GrassChunk);
                    world.insert(c, DynamicMesh { data: Arc::new(mesh), version: sig });
                    world.insert(c, MeshRenderer::default());
                }
            }
            world.get_mut::<Grass>(e).unwrap().sig = sig;
        }
        // Appearance is applied live to every chunk.
        let user = [
            g.wind_strength,
            g.wind_speed,
            g.fade_start,
            g.fade_end.max(g.fade_start + 1.0),
            g.tip_color[0],
            g.tip_color[1],
            g.tip_color[2],
            g.patchiness,
        ];
        for c in hierarchy::children_all(world, e) {
            if let Some(m) = world.get_mut::<MeshRenderer>(c) {
                m.shader = "builtin:grass".into();
                m.color = g.base_color;
                m.cast_shadows = false;
                m.roughness = 1.0;
                m.user = user;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;

    #[test]
    fn chunks_partition_the_patch_and_keep_the_blade_count() {
        let g = Grass { count: 4000, width: 20.0, depth: 12.0, chunk_size: 4.0, ..Default::default() };
        let (nx, nz) = chunk_grid(&g);
        assert_eq!((nx, nz), (5, 3));
        let mut blades = 0;
        for iz in 0..nz {
            for ix in 0..nx {
                let (m, center) = chunk_mesh(&g, ix, iz);
                blades += m.vertices.len() / BLADE_VERTICES;
                let (min, max) = m.bounds();
                assert!(
                    min.x > -2.8 && max.x < 2.8 && min.z > -2.8 && max.z < 2.8,
                    "chunk stays near its cell (blades may lean out a little): {min} {max}"
                );
                assert!(center.x.abs() <= 10.0 && center.z.abs() <= 6.0);
            }
        }
        assert!((3900..=4100).contains(&blades), "{blades}");
    }

    #[test]
    fn same_seed_same_field_and_triangle_budget() {
        let g = Grass { count: 800, chunk_size: 100.0, ..Default::default() };
        let (a, _) = chunk_mesh(&g, 0, 0);
        assert_eq!(a.vertices[10].pos, chunk_mesh(&g, 0, 0).0.vertices[10].pos);
        assert_ne!(a.vertices[10].pos, chunk_mesh(&Grass { seed: 2, ..g.clone() }, 0, 0).0.vertices[10].pos);
        assert_eq!(a.indices.len() / 3, 800 * BLADE_TRIANGLES);
    }

    #[test]
    fn chunks_are_transient_children_that_do_not_leak_into_scenes() {
        let reg = crate::registry::Registry::default();
        let mut scene = Scene::default();
        let e = hierarchy::spawn_named(&mut scene.world, "Lawn");
        scene.world.insert(e, Grass { count: 2000, width: 16.0, depth: 16.0, ..Default::default() });
        update(&mut scene.world);
        let all = hierarchy::children_all(&scene.world, e);
        assert_eq!(all.len(), 4, "16x16 patch in 8 m chunks");
        assert!(hierarchy::children(&scene.world, e).is_empty(), "hidden from the hierarchy");
        let json = scene.to_json(&reg);
        assert_eq!(json.matches("\"type\": \"Name\"").count(), 1, "only the Lawn is saved");
        // Changing only appearance keeps the chunks; changing the layout rebuilds them.
        scene.world.get_mut::<Grass>(e).unwrap().wind_strength = 2.0;
        update(&mut scene.world);
        assert_eq!(hierarchy::children_all(&scene.world, e), all);
        scene.world.get_mut::<Grass>(e).unwrap().chunk_size = 4.0;
        update(&mut scene.world);
        assert_eq!(hierarchy::children_all(&scene.world, e).len(), 16);
        scene.world.despawn(e);
        update(&mut scene.world);
        assert_eq!(scene.world.count::<GrassChunk>(), 0, "orphan chunks are removed");
    }
}
