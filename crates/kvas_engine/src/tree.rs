//! Procedural trees. A `Tree` grows a trunk with branches (tapered tubes) and clusters of leaf
//! cards at the twigs. Bark and leaves are two `Transient` child meshes drawn with the built-in
//! `builtin:bark` and `builtin:foliage` shaders: leaves are cut out in the shader and sway in the
//! wind on the GPU, so the meshes are built only when a parameter changes.
//!
//! ```
//! use kvas_engine::tree::{Tree, build};
//! let (bark, leaves) = build(&Tree::default());
//! assert!(!bark.indices.is_empty() && !leaves.indices.is_empty());
//! ```

use crate::components::*;
use crate::hierarchy;
use crate::mesh::{MeshData, Vertex};
use crate::procedural::{signature, Rng};
use crate::reflect::Field;
use glam::{Quat, Vec3};
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

crate::choice_enum! {
    /// Overall shape of a tree: Broadleaf (oak), Conifer (pine), Palm, or Bush.
    TreeStyle { Broadleaf, Conifer, Palm, Bush } default Broadleaf
}

/// A seeded procedural tree. Change any field in the Inspector and the tree regrows.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Tree {
    /// Overall shape.
    pub style: TreeStyle,
    /// Height of the trunk in metres.
    pub height: f32,
    /// Radius of the trunk at the ground in metres.
    pub trunk_radius: f32,
    /// How much the trunk and branches wander, 0..1.
    pub crookedness: f32,
    /// Branching levels below the twigs, 1..4.
    pub levels: i32,
    /// Branches growing from each branch.
    pub branches: i32,
    /// Branch length relative to its parent.
    pub branch_length: f32,
    /// Angle between a branch and its parent in degrees.
    pub branch_angle: f32,
    /// Leaf cards per twig cluster.
    pub leaf_density: i32,
    /// Size of one leaf card in metres.
    pub leaf_size: f32,
    /// Bark colour.
    pub bark_color: [f32; 4],
    /// Main leaf colour.
    pub leaf_color: [f32; 4],
    /// Second leaf colour mixed in per card (autumn tints, sun-bleached leaves...).
    pub leaf_color_2: [f32; 4],
    /// Wind sway strength.
    pub wind: f32,
    /// Deterministic seed.
    pub seed: i32,
    #[serde(skip)]
    #[doc(hidden)]
    pub sig: u64,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            style: TreeStyle::Broadleaf,
            height: 5.5,
            trunk_radius: 0.22,
            crookedness: 0.35,
            levels: 3,
            branches: 4,
            branch_length: 0.62,
            branch_angle: 42.0,
            leaf_density: 14,
            leaf_size: 0.75,
            bark_color: [0.29, 0.22, 0.16, 1.0],
            leaf_color: [0.05, 0.14, 0.03, 1.0],
            leaf_color_2: [0.16, 0.24, 0.04, 1.0],
            wind: 1.0,
            seed: 1,
            sig: 0,
        }
    }
}

impl Component for Tree {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::choice("style", &mut self.style),
            Field::f32("height", &mut self.height),
            Field::f32("trunk_radius", &mut self.trunk_radius),
            Field::f32("crookedness", &mut self.crookedness),
            Field::i32("levels", &mut self.levels),
            Field::i32("branches", &mut self.branches),
            Field::f32("branch_length", &mut self.branch_length),
            Field::f32("branch_angle", &mut self.branch_angle),
            Field::i32("leaf_density", &mut self.leaf_density),
            Field::f32("leaf_size", &mut self.leaf_size),
            Field::color("bark_color", &mut self.bark_color),
            Field::color("leaf_color", &mut self.leaf_color),
            Field::color("leaf_color_2", &mut self.leaf_color_2),
            Field::f32("wind", &mut self.wind),
            Field::i32("seed", &mut self.seed),
        ]
    }
}

/// Marks a child mesh created for a tree (bark or leaves).
#[derive(Clone, Copy, Default)]
pub struct TreePart;

const SIDES: usize = 9;

struct Grow<'a> {
    rng: Rng,
    t: &'a Tree,
    bark: MeshData,
    /// Twig tips: position, direction and level size, where leaf clusters go.
    tips: Vec<(Vec3, Vec3, f32)>,
}

fn any_perp(d: Vec3) -> Vec3 {
    d.any_orthonormal_vector()
}

impl Grow<'_> {
    fn r(&mut self) -> f32 {
        self.rng.f32()
    }

    /// A tapered tube along `points` with radii `radii`; `v0` continues the bark texture.
    fn tube(&mut self, points: &[Vec3], radii: &[f32], v0: f32) {
        let base = self.bark.vertices.len() as u32;
        let mut v = v0;
        let mut frame = any_perp((points[1] - points[0]).normalize_or_zero());
        for (i, p) in points.iter().enumerate() {
            let dir = if i + 1 < points.len() { points[i + 1] - *p } else { *p - points[i - 1] }.normalize_or_zero();
            // Keep the ring orientation continuous (parallel transport).
            frame = (frame - dir * frame.dot(dir)).normalize_or(any_perp(dir));
            let side = dir.cross(frame);
            if i > 0 {
                v += (points[i] - points[i - 1]).length() / (radii[i].max(0.01) * 6.0);
            }
            for k in 0..=SIDES {
                let a = k as f32 / SIDES as f32 * std::f32::consts::TAU;
                let n = frame * a.cos() + side * a.sin();
                self.bark.vertices.push(Vertex { pos: (*p + n * radii[i]).into(), normal: n.into(), uv: [k as f32 / SIDES as f32, v] });
            }
        }
        let ring = (SIDES + 1) as u32;
        for i in 0..points.len() as u32 - 1 {
            for k in 0..SIDES as u32 {
                let (a, b, c, d) = (base + i * ring + k, base + i * ring + k + 1, base + (i + 1) * ring + k, base + (i + 1) * ring + k + 1);
                self.bark.indices.extend([a, c, b, b, c, d]);
            }
        }
    }

    /// Grows a branch from `start` along `dir`; recurses into children until `level` reaches 0.
    fn branch(&mut self, start: Vec3, dir: Vec3, length: f32, radius: f32, level: i32, v0: f32) {
        let t = self.t;
        let segments = if level == t.levels.max(1) { 7 } else { 4 };
        let mut points = vec![start];
        let mut radii = vec![radius];
        let mut d = dir;
        let mut p = start;
        let seg = length / segments as f32;
        let bend = t.crookedness * 0.45;
        for i in 1..=segments {
            let wander = Vec3::new(self.r() - 0.5, (self.r() - 0.5) * 0.4, self.r() - 0.5) * bend;
            // Broadleaf branches reach up and out; conifer branches droop.
            let tropism = match t.style {
                TreeStyle::Conifer if level < t.levels => Vec3::NEG_Y * 0.08,
                TreeStyle::Palm => Vec3::Y * 0.04 + Vec3::new(0.06, 0.0, 0.0) * (i as f32 / segments as f32),
                _ => Vec3::Y * 0.05,
            };
            d = (d + wander + tropism).normalize();
            p += d * seg;
            points.push(p);
            let k = i as f32 / segments as f32;
            radii.push((radius * (1.0 - 0.72 * k)).max(0.008));
        }
        self.tube(&points, &radii, v0);
        if level <= 0 {
            self.tips.push((p, d, length));
            // Leaves also along the last stretch of the twig.
            let mid = points[points.len() / 2];
            self.tips.push((mid, d, length * 0.8));
            return;
        }
        let children = t.branches.clamp(1, 8);
        let angle = t.branch_angle.to_radians();
        let mut twist = self.r() * std::f32::consts::TAU;
        for c in 0..children {
            let along = match t.style {
                TreeStyle::Conifer if level == t.levels => 0.25 + 0.7 * (c as f32 + self.r() * 0.5) / children as f32,
                _ => 0.35 + 0.6 * (c as f32 + self.r()) / children as f32,
            };
            let idx = ((along * segments as f32) as usize).min(segments);
            let at = points[idx];
            let r_at = radii[idx];
            twist += 2.399_963; // golden angle
            let axis = (Quat::from_axis_angle(d, twist) * any_perp(d)).normalize();
            let child_dir = (Quat::from_axis_angle(axis, angle * (0.8 + 0.4 * self.r())) * d).normalize();
            let mut child_len = length * t.branch_length * (0.85 + 0.3 * self.r());
            if t.style == TreeStyle::Conifer && level == t.levels {
                // Whorls get shorter towards the top: a cone.
                child_len = t.height * 0.45 * (1.0 - along) * (0.85 + 0.3 * self.r());
            }
            self.branch(at, child_dir, child_len, r_at * 0.62, level - 1, v0);
        }
    }
}

/// Builds the bark and leaf meshes of a tree (in the tree's local space, root at the origin).
pub fn build(t: &Tree) -> (MeshData, MeshData) {
    let mut g = Grow { rng: Rng::new(t.seed as u64 ^ 0x7ee5), t, bark: MeshData::default(), tips: Vec::new() };
    let levels = t.levels.clamp(1, 4);
    let height = t.height.max(0.3);
    match t.style {
        TreeStyle::Bush => {
            for i in 0..5 {
                let a = i as f32 / 5.0 * std::f32::consts::TAU + g.r();
                let dir = Vec3::new(a.cos() * 0.6, 1.0, a.sin() * 0.6).normalize();
                g.branch(Vec3::ZERO, dir, height * 0.6, t.trunk_radius * 0.5, (levels - 1).max(1), 0.0);
            }
        }
        TreeStyle::Palm => {
            // One bending trunk, then a crown of long fronds (leaf cards along drooping stems).
            let tree = Tree { crookedness: t.crookedness * 0.5, ..t.clone() };
            let mut p = Grow { rng: Rng::new(t.seed as u64 ^ 0x9a1), t: &tree, bark: MeshData::default(), tips: Vec::new() };
            p.branch(Vec3::ZERO, Vec3::Y, height, t.trunk_radius, 0, 0.0);
            let (top, _, _) = p.tips[0];
            g.bark = p.bark;
            for i in 0..9 {
                let a = i as f32 / 9.0 * std::f32::consts::TAU + g.r() * 0.3;
                let out = Vec3::new(a.cos(), 0.0, a.sin());
                for s in 1..=6 {
                    let k = s as f32 / 6.0;
                    let pos = top + out * (k * height * 0.38) + Vec3::Y * (0.35 * k - 0.9 * k * k) * height * 0.3;
                    g.tips.push((pos, out, height * 0.1));
                }
            }
        }
        _ => {
            let trunk_levels = levels;
            g.branch(Vec3::ZERO, Vec3::Y, height, t.trunk_radius, trunk_levels, 0.0);
        }
    }

    // Leaves: cards around every twig tip, turned to random directions; normals point away from
    // the crown centre so the foliage is lit like one soft volume.
    let mut leaves = MeshData::default();
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for (p, _, _) in &g.tips {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }
    let centre = (lo + hi) * 0.5;
    let conifer = t.style == TreeStyle::Conifer;
    let per = t.leaf_density.clamp(1, 64) as usize;
    let tips = g.tips.clone();
    for (p, dir, len) in tips {
        let spread = (len * 0.35).clamp(0.15, 1.2) * if conifer { 0.6 } else { 1.0 };
        for _ in 0..per {
            let offset = Vec3::new(g.r() - 0.5, g.r() - 0.5, g.r() - 0.5) * 2.0 * spread + dir * g.r() * spread * 0.5;
            let at = p + offset;
            let size = t.leaf_size * (0.7 + 0.6 * g.r()) * if conifer { 0.8 } else { 1.0 };
            let facing = Vec3::new(g.r() - 0.5, g.r() - 0.5, g.r() - 0.5).normalize_or(Vec3::Y);
            let u = any_perp(facing);
            let v = facing.cross(u);
            let (u, v) = if conifer { (dir * size * 0.9, v * size * 0.55) } else { (u * size * 0.5, v * size * 0.5) };
            let mut n = (at - centre).normalize_or(Vec3::Y);
            n = (n + Vec3::Y * 0.35).normalize();
            let tint = (g.r() * 8.0).floor();
            let base = leaves.vertices.len() as u32;
            for (cx, cy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let pos = at + u * cx + v * cy;
                leaves.vertices.push(Vertex { pos: pos.into(), normal: n.into(), uv: [(cx * 0.5 + 0.5) + tint, cy * 0.5 + 0.5] });
            }
            leaves.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            leaves.indices.extend([base + 2, base + 1, base, base + 3, base + 2, base]);
        }
    }
    (g.bark, leaves)
}

/// Rebuilds changed trees and their transient bark and leaf meshes.
pub fn update(world: &mut World) {
    let orphans: Vec<Entity> = world
        .entities_with::<TreePart>()
        .into_iter()
        .filter(|&c| hierarchy::parent_of(world, c).is_none_or(|p| !world.is_alive(p) || !world.has::<Tree>(p)))
        .collect();
    for c in orphans {
        world.despawn(c);
    }
    for e in world.entities_with::<Tree>() {
        let t = world.get::<Tree>(e).unwrap().clone();
        let sig = signature(&(t.style, t.height.to_bits(), t.trunk_radius.to_bits(), t.crookedness.to_bits(), t.levels, t.branches, t.branch_length.to_bits(), t.branch_angle.to_bits(), t.leaf_density, t.leaf_size.to_bits(), t.seed));
        let parts: Vec<Entity> = hierarchy::children_all(world, e).into_iter().filter(|&c| world.has::<TreePart>(c)).collect();
        if t.sig != sig || parts.len() != 2 {
            for c in &parts {
                world.despawn(*c);
            }
            let (bark, leaves) = build(&t);
            for (name, mesh) in [("Bark", bark), ("Leaves", leaves)] {
                let c = world.spawn();
                world.insert(c, Name(name.into()));
                world.insert(c, Transform::default());
                world.insert(c, Parent(e));
                world.insert(c, Transient);
                world.insert(c, TreePart);
                world.insert(c, DynamicMesh { data: Arc::new(mesh), version: sig });
                world.insert(c, MeshRenderer::default());
            }
            world.get_mut::<Tree>(e).unwrap().sig = sig;
        }
        // Colours and wind apply live.
        for c in hierarchy::children_all(world, e) {
            let is_leaves = world.get::<Name>(c).is_some_and(|n| n.0 == "Leaves");
            if let Some(m) = world.get_mut::<MeshRenderer>(c) {
                if is_leaves {
                    m.shader = "builtin:foliage".into();
                    m.color = t.leaf_color;
                    m.user = [t.wind, if t.style == TreeStyle::Conifer { 1.0 } else if t.style == TreeStyle::Palm { 2.0 } else { 0.0 }, 0.0, 0.0, t.leaf_color_2[0], t.leaf_color_2[1], t.leaf_color_2[2], 0.0];
                    m.cast_shadows = true;
                    m.roughness = 0.8;
                } else {
                    m.shader = "builtin:bark".into();
                    m.color = t.bark_color;
                    m.user = [t.wind, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
                    m.cast_shadows = true;
                    m.roughness = 0.95;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;

    #[test]
    fn every_style_grows_bark_and_leaves_deterministically() {
        for style in [TreeStyle::Broadleaf, TreeStyle::Conifer, TreeStyle::Palm, TreeStyle::Bush] {
            let t = Tree { style, ..Default::default() };
            let (bark, leaves) = build(&t);
            assert!(bark.indices.len() > 100 && leaves.indices.len() > 100, "{style:?}");
            assert!(bark.validate().is_ok() && leaves.validate().is_ok());
            assert_eq!(build(&t).1.vertices[5].pos, leaves.vertices[5].pos);
            let (_, max) = bark.bounds();
            assert!(max.y > t.height * 0.5, "{style:?} reaches up");
        }
    }

    #[test]
    fn parts_are_transient_and_rebuilt_on_change() {
        let mut scene = Scene::default();
        let e = hierarchy::spawn_named(&mut scene.world, "Oak");
        scene.world.insert(e, Tree::default());
        update(&mut scene.world);
        let parts = hierarchy::children_all(&scene.world, e);
        assert_eq!(parts.len(), 2);
        assert!(hierarchy::children(&scene.world, e).is_empty(), "hidden from the hierarchy");
        scene.world.get_mut::<Tree>(e).unwrap().wind = 3.0;
        update(&mut scene.world);
        assert_eq!(hierarchy::children_all(&scene.world, e), parts, "appearance changes keep the meshes");
        scene.world.get_mut::<Tree>(e).unwrap().seed = 9;
        update(&mut scene.world);
        assert_ne!(hierarchy::children_all(&scene.world, e), parts);
    }
}
