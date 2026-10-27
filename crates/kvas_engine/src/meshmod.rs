//! Mesh modifiers: a Blender-style stack that reshapes an object's mesh without touching the
//! source model. Add [`MeshModifiers`] to an object with a `Shape` or a model and stack
//! modifiers (Array, Mirror, Subdivide, Displace, Twist, Bend, Taper, Solidify) in any order.
//!
//! The result is a `DynamicMesh`, rebuilt only when the stack or the source changes.
//!
//! ```
//! use kvas_engine::{prelude::*, scene::Scene, meshmod::*};
//! let mut scene = Scene::default();
//! let e = scene.spawn("Fence");
//! scene.world.insert(e, Shape::of(ShapeType::Cube));
//! scene.world.insert(e, MeshRenderer::default());
//! scene.world.insert(e, MeshModifiers(vec![Modifier::array(3, Vec3::new(1.5, 0.0, 0.0))]));
//! update(&mut scene.world);
//! assert_eq!(scene.world.get::<DynamicMesh>(e).unwrap().data.vertices.len(), 3 * 24);
//! ```

use crate::assets::Assets;
use crate::components::*;
use crate::mesh::MeshData;
use crate::noise::{Noise, NoiseKind};
use crate::procedural::signature;
use crate::reflect::Field;
use glam::{Mat4, Quat, Vec3};
use kvas_ecs::World;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

crate::choice_enum!(
    /// Axis a modifier works along.
    ModAxis { X, Y, Z } default Y
);

impl ModAxis {
    fn index(self) -> usize {
        self as usize
    }
}

/// One entry of the stack. Fields are public so scripts can build stacks in code.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Modifier {
    /// Repeats the mesh `count` times, each copy moved, rotated and scaled relative to the previous one.
    Array {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Number of copies including the original (1..64).
        count: i32,
        /// Offset between copies in local metres.
        offset: Vec3,
        /// Extra rotation per copy in degrees.
        rotation: Vec3,
        /// Scale multiplier per copy (1 = unchanged).
        scale: f32,
    },
    /// Adds mirrored copies across the chosen local planes.
    Mirror {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Mirror across the YZ plane.
        x: bool,
        /// Mirror across the XZ plane.
        y: bool,
        /// Mirror across the XY plane.
        z: bool,
    },
    /// Splits every triangle into four, `levels` times (more vertices for Displace / Bend).
    Subdivide {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Number of subdivision passes (1..4).
        levels: i32,
        /// Pull new vertices onto a sphere of the original size, rounding the shape.
        smooth: f32,
    },
    /// Pushes vertices along their normals by noise.
    Displace {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Displacement in metres.
        strength: f32,
        /// Noise features per metre.
        scale: f32,
        /// Noise seed.
        seed: i32,
    },
    /// Rotates slices of the mesh progressively around an axis (a screw / wring).
    Twist {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Total twist in degrees from one end to the other.
        angle: f32,
        /// Axis the twist runs along.
        axis: ModAxis,
    },
    /// Curves the mesh along an axis.
    Bend {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Total bend in degrees from one end to the other.
        angle: f32,
        /// Axis that gets bent.
        axis: ModAxis,
    },
    /// Scales the cross-section along an axis: pointed or flared shapes.
    Taper {
        /// Whether the modifier is applied.
        enabled: bool,
        /// -1 pinches the top, +1 doubles it.
        amount: f32,
        /// Axis the taper runs along.
        axis: ModAxis,
    },
    /// Gives surfaces thickness by adding an inner shell.
    Solidify {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Shell thickness in metres.
        thickness: f32,
    },
    /// Ripples the surface like a flag or water: a sine wave along one axis lifts the vertices.
    Wave {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Height of the ripples in metres.
        amplitude: f32,
        /// Distance between crests in metres.
        wavelength: f32,
        /// Axis the wave travels along.
        axis: ModAxis,
        /// Shifts the wave pattern (0..1 = one full wavelength).
        phase: f32,
    },
    /// Pulls vertices towards a sphere (or pushes them away): turns boxes into balls.
    Cast {
        /// Whether the modifier is applied.
        enabled: bool,
        /// 0 = unchanged, 1 = a perfect sphere, negative = inflated spikes.
        factor: f32,
    },
    /// Slants the mesh: the further along the axis, the more it leans sideways.
    Shear {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Lean per metre along the axis.
        amount: f32,
        /// Axis that decides how far each vertex leans.
        axis: ModAxis,
    },
    /// Averages every vertex with its neighbours: rounds off hard edges and noise.
    Smooth {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Passes (1..10).
        iterations: i32,
        /// How far each pass moves a vertex towards its neighbours, 0..1.
        factor: f32,
    },
    /// Random push of every vertex: rough stone, crumpled paper, broken glass.
    Jitter {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Maximum offset in metres.
        amount: f32,
        /// Random seed.
        seed: i32,
    },
    /// Copies the mesh around a circle (columns of a temple, a fence ring, a wheel of spokes).
    RadialArray {
        /// Whether the modifier is applied.
        enabled: bool,
        /// Copies including the original (2..64).
        count: i32,
        /// Distance of each copy from the centre in metres.
        radius: f32,
        /// Axis the circle turns around.
        axis: ModAxis,
        /// Turn every copy to face outwards.
        face_outward: bool,
    },
}

impl Modifier {
    /// Default array: `count` copies spaced by `offset`.
    pub fn array(count: i32, offset: Vec3) -> Self {
        Modifier::Array { enabled: true, count, offset, rotation: Vec3::ZERO, scale: 1.0 }
    }

    /// Default instance of a kind by name ("Array", "Mirror", ...), used by the editor's Add menu.
    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "Array" => Self::array(3, Vec3::new(1.2, 0.0, 0.0)),
            "Mirror" => Modifier::Mirror { enabled: true, x: true, y: false, z: false },
            "Subdivide" => Modifier::Subdivide { enabled: true, levels: 1, smooth: 0.0 },
            "Displace" => Modifier::Displace { enabled: true, strength: 0.15, scale: 2.0, seed: 1 },
            "Twist" => Modifier::Twist { enabled: true, angle: 90.0, axis: ModAxis::Y },
            "Bend" => Modifier::Bend { enabled: true, angle: 60.0, axis: ModAxis::Y },
            "Taper" => Modifier::Taper { enabled: true, amount: -0.6, axis: ModAxis::Y },
            "Solidify" => Modifier::Solidify { enabled: true, thickness: 0.1 },
            "Wave" => Modifier::Wave { enabled: true, amplitude: 0.2, wavelength: 2.0, axis: ModAxis::X, phase: 0.0 },
            "Cast" => Modifier::Cast { enabled: true, factor: 0.6 },
            "Shear" => Modifier::Shear { enabled: true, amount: 0.4, axis: ModAxis::Y },
            "Smooth" => Modifier::Smooth { enabled: true, iterations: 2, factor: 0.5 },
            "Jitter" => Modifier::Jitter { enabled: true, amount: 0.05, seed: 1 },
            "RadialArray" => Modifier::RadialArray { enabled: true, count: 8, radius: 3.0, axis: ModAxis::Y, face_outward: true },
            _ => return None,
        })
    }

    /// Names accepted by [`Self::named`], in menu order.
    pub const NAMES: &'static [&'static str] = &["Array", "RadialArray", "Mirror", "Subdivide", "Smooth", "Displace", "Jitter", "Wave", "Twist", "Bend", "Taper", "Shear", "Cast", "Solidify"];

    /// Display name of this modifier.
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::Array { .. } => "Array",
            Modifier::Mirror { .. } => "Mirror",
            Modifier::Subdivide { .. } => "Subdivide",
            Modifier::Displace { .. } => "Displace",
            Modifier::Twist { .. } => "Twist",
            Modifier::Bend { .. } => "Bend",
            Modifier::Taper { .. } => "Taper",
            Modifier::Solidify { .. } => "Solidify",
            Modifier::Wave { .. } => "Wave",
            Modifier::Cast { .. } => "Cast",
            Modifier::Shear { .. } => "Shear",
            Modifier::Smooth { .. } => "Smooth",
            Modifier::Jitter { .. } => "Jitter",
            Modifier::RadialArray { .. } => "RadialArray",
        }
    }

    /// Whether the modifier is applied.
    pub fn enabled(&self) -> bool {
        match self {
            Modifier::Array { enabled, .. }
            | Modifier::Mirror { enabled, .. }
            | Modifier::Subdivide { enabled, .. }
            | Modifier::Displace { enabled, .. }
            | Modifier::Twist { enabled, .. }
            | Modifier::Bend { enabled, .. }
            | Modifier::Taper { enabled, .. }
            | Modifier::Solidify { enabled, .. }
            | Modifier::Wave { enabled, .. }
            | Modifier::Cast { enabled, .. }
            | Modifier::Shear { enabled, .. }
            | Modifier::Smooth { enabled, .. }
            | Modifier::Jitter { enabled, .. }
            | Modifier::RadialArray { enabled, .. } => *enabled,
        }
    }

    /// Inspector fields of this modifier (including the enabled switch).
    pub fn fields(&mut self) -> Vec<Field<'_>> {
        match self {
            Modifier::Array { enabled, count, offset, rotation, scale } => {
                vec![Field::bool("enabled", enabled), Field::i32("count", count), Field::vec3("offset", offset), Field::vec3("rotation", rotation), Field::f32("scale", scale)]
            }
            Modifier::Mirror { enabled, x, y, z } => vec![Field::bool("enabled", enabled), Field::bool("x", x), Field::bool("y", y), Field::bool("z", z)],
            Modifier::Subdivide { enabled, levels, smooth } => vec![Field::bool("enabled", enabled), Field::i32("levels", levels), Field::f32("smooth", smooth)],
            Modifier::Displace { enabled, strength, scale, seed } => {
                vec![Field::bool("enabled", enabled), Field::f32("strength", strength), Field::f32("scale", scale), Field::i32("seed", seed)]
            }
            Modifier::Twist { enabled, angle, axis } | Modifier::Bend { enabled, angle, axis } => {
                vec![Field::bool("enabled", enabled), Field::f32("angle", angle), Field::choice("axis", axis)]
            }
            Modifier::Taper { enabled, amount, axis } => vec![Field::bool("enabled", enabled), Field::f32("amount", amount), Field::choice("axis", axis)],
            Modifier::Solidify { enabled, thickness } => vec![Field::bool("enabled", enabled), Field::f32("thickness", thickness)],
            Modifier::Wave { enabled, amplitude, wavelength, axis, phase } => {
                vec![Field::bool("enabled", enabled), Field::f32("amplitude", amplitude), Field::f32("wavelength", wavelength), Field::choice("axis", axis), Field::f32("phase", phase)]
            }
            Modifier::Cast { enabled, factor } => vec![Field::bool("enabled", enabled), Field::f32("factor", factor)],
            Modifier::Shear { enabled, amount, axis } => vec![Field::bool("enabled", enabled), Field::f32("amount", amount), Field::choice("axis", axis)],
            Modifier::Smooth { enabled, iterations, factor } => vec![Field::bool("enabled", enabled), Field::i32("iterations", iterations), Field::f32("factor", factor)],
            Modifier::Jitter { enabled, amount, seed } => vec![Field::bool("enabled", enabled), Field::f32("amount", amount), Field::i32("seed", seed)],
            Modifier::RadialArray { enabled, count, radius, axis, face_outward } => {
                vec![Field::bool("enabled", enabled), Field::i32("count", count), Field::f32("radius", radius), Field::choice("axis", axis), Field::bool("face_outward", face_outward)]
            }
        }
    }
}

/// The modifier stack of an object, applied top to bottom.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MeshModifiers(
    /// Modifiers in application order.
    pub Vec<Modifier>,
);

impl Component for MeshModifiers {}

/// Cache of what the last build used, so unchanged stacks cost nothing.
#[derive(Clone, Default)]
pub struct ModifierState {
    sig: u64,
}

static ASSETS: RwLock<Option<Assets>> = RwLock::new(None);

/// Tells the modifier system where to load model files from (the editor and player call this).
pub fn set_assets(assets: Assets) {
    if let Ok(mut slot) = ASSETS.write() {
        *slot = Some(assets);
    }
}

/// The asset source set with [`set_assets`], if any.
pub fn assets() -> Option<Assets> {
    ASSETS.read().ok()?.clone()
}

fn source_mesh(key: &str) -> Option<MeshData> {
    let assets = ASSETS.read().ok()?.clone().unwrap_or(Assets::Dir(std::path::PathBuf::new()));
    MeshData::load(&assets, key).ok()
}

fn axes(axis: ModAxis) -> (usize, usize, usize) {
    let a = axis.index();
    (a, (a + 1) % 3, (a + 2) % 3)
}

fn extent(mesh: &MeshData, axis: usize) -> (f32, f32) {
    mesh.vertices.iter().fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(v.pos[axis]), hi.max(v.pos[axis])))
}

fn renormalize(mesh: &mut MeshData) {
    let _ = mesh.recalculate_normals();
}

fn subdivide(mesh: &mut MeshData, levels: i32, smooth: f32) {
    let radius = mesh.vertices.iter().map(|v| Vec3::from(v.pos).length()).fold(0.0, f32::max);
    for _ in 0..levels.clamp(1, 4) {
        if mesh.indices.len() > 400_000 {
            break;
        }
        let mut out = MeshData { albedo: mesh.albedo.clone(), base_color: mesh.base_color, dependencies: mesh.dependencies.clone(), ..Default::default() };
        out.parts = mesh.parts.iter().map(|p| crate::mesh::MeshPart { first_index: p.first_index * 4, index_count: p.index_count * 4, albedo: p.albedo.clone(), normal: p.normal.clone(), orm: p.orm.clone() }).collect();
        for tri in mesh.indices.chunks_exact(3) {
            let v = [mesh.vertices[tri[0] as usize], mesh.vertices[tri[1] as usize], mesh.vertices[tri[2] as usize]];
            let mid = |a: crate::mesh::Vertex, b: crate::mesh::Vertex| {
                let p = (Vec3::from(a.pos) + Vec3::from(b.pos)) * 0.5;
                let p = if smooth > 0.0 && radius > 1e-4 && p.length() > 1e-4 { p.lerp(p.normalize() * radius, smooth.clamp(0.0, 1.0)) } else { p };
                let n = (Vec3::from(a.normal) + Vec3::from(b.normal)).normalize_or_zero();
                crate::mesh::Vertex { pos: p.into(), normal: n.into(), uv: [(a.uv[0] + b.uv[0]) * 0.5, (a.uv[1] + b.uv[1]) * 0.5] }
            };
            let (m01, m12, m20) = (mid(v[0], v[1]), mid(v[1], v[2]), mid(v[2], v[0]));
            let base = out.vertices.len() as u32;
            out.vertices.extend([v[0], m01, m20, m01, v[1], m12, m20, m12, v[2], m01, m12, m20]);
            out.indices.extend([base, base + 1, base + 2, base + 3, base + 4, base + 5, base + 6, base + 7, base + 8, base + 9, base + 10, base + 11]);
        }
        *mesh = out;
    }
}

fn apply(mesh: &mut MeshData, modifier: &Modifier) {
    match modifier {
        Modifier::Array { count, offset, rotation, scale, .. } => {
            let count = (*count).clamp(1, 64) as usize;
            let original = mesh.clone();
            let mut matrix = Mat4::IDENTITY;
            let step = Mat4::from_scale_rotation_translation(Vec3::splat(scale.max(0.01)), Quat::from_euler(glam::EulerRot::YXZ, rotation.y.to_radians(), rotation.x.to_radians(), rotation.z.to_radians()), *offset);
            for _ in 1..count {
                if mesh.vertices.len() + original.vertices.len() > 600_000 {
                    break;
                }
                matrix *= step;
                if let Ok(copy) = original.transformed(matrix) {
                    let _ = mesh.append(&copy);
                }
            }
        }
        Modifier::Mirror { x, y, z, .. } => {
            for (on, scale) in [(*x, Vec3::new(-1.0, 1.0, 1.0)), (*y, Vec3::new(1.0, -1.0, 1.0)), (*z, Vec3::new(1.0, 1.0, -1.0))] {
                if on && mesh.vertices.len() < 300_000 {
                    if let Ok(copy) = mesh.transformed(Mat4::from_scale(scale)) {
                        let _ = mesh.append(&copy);
                    }
                }
            }
        }
        Modifier::Subdivide { levels, smooth, .. } => {
            subdivide(mesh, *levels, *smooth);
        }
        Modifier::Displace { strength, scale, seed, .. } => {
            let noise = Noise::new(NoiseKind::Perlin).with_seed(*seed as u32).with_octaves(3);
            for v in &mut mesh.vertices {
                let p = Vec3::from(v.pos);
                let d = noise.sample3(p.x * scale, p.y * scale, p.z * scale) * strength;
                v.pos = (p + Vec3::from(v.normal) * d).into();
            }
            renormalize(mesh);
        }
        Modifier::Twist { angle, axis, .. } => {
            let (a, b, c) = axes(*axis);
            let (lo, hi) = extent(mesh, a);
            let span = (hi - lo).max(1e-4);
            for v in &mut mesh.vertices {
                let t = (v.pos[a] - lo) / span - 0.5;
                let (s, co) = (angle.to_radians() * t).sin_cos();
                let (pb, pc) = (v.pos[b], v.pos[c]);
                v.pos[b] = pb * co - pc * s;
                v.pos[c] = pb * s + pc * co;
            }
            renormalize(mesh);
        }
        Modifier::Bend { angle, axis, .. } => {
            let (a, b, _) = axes(*axis);
            let (lo, hi) = extent(mesh, a);
            let span = (hi - lo).max(1e-4);
            let k = angle.to_radians() / span;
            if k.abs() > 1e-4 {
                let r = 1.0 / k;
                let mid = (lo + hi) * 0.5;
                for v in &mut mesh.vertices {
                    let along = v.pos[a] - mid;
                    let theta = k * along;
                    let off = v.pos[b];
                    v.pos[a] = (r - off) * theta.sin() + mid;
                    v.pos[b] = r - (r - off) * theta.cos();
                }
                renormalize(mesh);
            }
        }
        Modifier::Taper { amount, axis, .. } => {
            let (a, b, c) = axes(*axis);
            let (lo, hi) = extent(mesh, a);
            let span = (hi - lo).max(1e-4);
            for v in &mut mesh.vertices {
                let t = (v.pos[a] - lo) / span;
                let f = (1.0 + amount * t).max(0.01);
                v.pos[b] *= f;
                v.pos[c] *= f;
            }
            renormalize(mesh);
        }
        Modifier::Wave { amplitude, wavelength, axis, phase, .. } => {
            let (a, b, _) = axes(*axis);
            let up = (a + 1) % 3;
            let _ = b;
            let k = std::f32::consts::TAU / wavelength.max(0.05);
            for v in &mut mesh.vertices {
                v.pos[up] += ((v.pos[a] * k) + phase * std::f32::consts::TAU).sin() * amplitude;
            }
            renormalize(mesh);
        }
        Modifier::Cast { factor, .. } => {
            let radius = mesh.vertices.iter().map(|v| Vec3::from(v.pos).length()).sum::<f32>() / mesh.vertices.len().max(1) as f32;
            for v in &mut mesh.vertices {
                let p = Vec3::from(v.pos);
                if p.length() > 1e-5 {
                    v.pos = p.lerp(p.normalize() * radius, *factor).into();
                }
            }
            renormalize(mesh);
        }
        Modifier::Shear { amount, axis, .. } => {
            let (a, b, _) = axes(*axis);
            for v in &mut mesh.vertices {
                v.pos[b] += v.pos[a] * amount;
            }
            renormalize(mesh);
        }
        Modifier::Smooth { iterations, factor, .. } => {
            // Vertices that share a position (hard edges are split in the mesh) are treated as one.
            let key = |p: [f32; 3]| ((p[0] * 1000.0).round() as i64, (p[1] * 1000.0).round() as i64, (p[2] * 1000.0).round() as i64);
            let mut ids = std::collections::HashMap::new();
            let mut slot = Vec::with_capacity(mesh.vertices.len());
            for v in &mesh.vertices {
                let next = ids.len();
                slot.push(*ids.entry(key(v.pos)).or_insert(next));
            }
            let mut neighbours: Vec<std::collections::HashSet<usize>> = vec![Default::default(); ids.len()];
            for tri in mesh.indices.chunks_exact(3) {
                let s = [slot[tri[0] as usize], slot[tri[1] as usize], slot[tri[2] as usize]];
                for i in 0..3 {
                    neighbours[s[i]].insert(s[(i + 1) % 3]);
                    neighbours[s[i]].insert(s[(i + 2) % 3]);
                }
            }
            let mut positions = vec![Vec3::ZERO; ids.len()];
            for (v, &i) in mesh.vertices.iter().zip(&slot) {
                positions[i] = Vec3::from(v.pos);
            }
            for _ in 0..(*iterations).clamp(1, 10) {
                let next: Vec<Vec3> = (0..positions.len())
                    .map(|i| {
                        if neighbours[i].is_empty() {
                            return positions[i];
                        }
                        let avg = neighbours[i].iter().map(|&n| positions[n]).sum::<Vec3>() / neighbours[i].len() as f32;
                        positions[i].lerp(avg, factor.clamp(0.0, 1.0))
                    })
                    .collect();
                positions = next;
            }
            for (v, &i) in mesh.vertices.iter_mut().zip(&slot) {
                v.pos = positions[i].into();
            }
            renormalize(mesh);
        }
        Modifier::Jitter { amount, seed, .. } => {
            let key = |p: [f32; 3]| ((p[0] * 1000.0).round() as i64, (p[1] * 1000.0).round() as i64, (p[2] * 1000.0).round() as i64);
            let mut rng = crate::procedural::Rng::new(*seed as u64);
            let mut offsets = std::collections::HashMap::new();
            for v in &mut mesh.vertices {
                // Split vertices at the same spot get the same push, so the mesh stays closed.
                let o = *offsets.entry(key(v.pos)).or_insert_with(|| Vec3::new(rng.f32() - 0.5, rng.f32() - 0.5, rng.f32() - 0.5) * 2.0 * *amount);
                v.pos = (Vec3::from(v.pos) + o).into();
            }
            renormalize(mesh);
        }
        Modifier::RadialArray { count, radius, axis, face_outward, .. } => {
            let count = (*count).clamp(2, 64) as usize;
            let original = mesh.clone();
            let (a, b, c) = axes(*axis);
            for i in 1..count {
                if mesh.vertices.len() + original.vertices.len() > 600_000 {
                    break;
                }
                let angle = std::f32::consts::TAU * i as f32 / count as f32;
                let mut offset = [0.0f32; 3];
                offset[b] = angle.cos() * radius;
                offset[c] = angle.sin() * radius;
                let mut axis_vec = Vec3::ZERO;
                axis_vec[a] = 1.0;
                let spin = if *face_outward { Quat::from_axis_angle(axis_vec, -angle) } else { Quat::IDENTITY };
                let matrix = Mat4::from_rotation_translation(spin, Vec3::from(offset));
                if let Ok(copy) = original.transformed(matrix) {
                    let _ = mesh.append(&copy);
                }
            }
        }
        Modifier::Solidify { thickness, .. } => {
            if mesh.vertices.len() < 300_000 {
                let base = mesh.vertices.len() as u32;
                let inner: Vec<_> = mesh
                    .vertices
                    .iter()
                    .map(|v| crate::mesh::Vertex { pos: (Vec3::from(v.pos) - Vec3::from(v.normal) * *thickness).into(), normal: (-Vec3::from(v.normal)).into(), uv: v.uv })
                    .collect();
                let tris: Vec<u32> = mesh.indices.chunks_exact(3).flat_map(|t| [t[0] + base, t[2] + base, t[1] + base]).collect();
                if !mesh.parts.is_empty() {
                    let shift = mesh.indices.len() as u32;
                    let copy: Vec<_> = mesh.parts.iter().map(|p| crate::mesh::MeshPart { first_index: p.first_index + shift, ..p.clone() }).collect();
                    mesh.parts.extend(copy);
                }
                mesh.vertices.extend(inner);
                mesh.indices.extend(tris);
            }
        }
    }
}

/// Runs the stack on a source mesh.
pub fn build(source: MeshData, stack: &[Modifier]) -> MeshData {
    let mut mesh = source;
    for m in stack.iter().filter(|m| m.enabled()) {
        apply(&mut mesh, m);
    }
    mesh
}

/// Rebuilds the `DynamicMesh` of every object with modifiers whose stack or source changed.
pub fn update(world: &mut World) {
    for e in world.entities_with::<MeshModifiers>() {
        let Some(key) = world.get::<Shape>(e).map(Shape::key).or_else(|| world.get::<MeshRenderer>(e).map(|m| m.mesh.clone())).filter(|k| !k.is_empty()) else { continue };
        let stack = world.get::<MeshModifiers>(e).map(|m| m.0.clone()).unwrap_or_default();
        let sig = signature(&(&stack, &key));
        let known = world.get::<ModifierStateCell>(e).map(|c| c.0.sig);
        if known == Some(sig) && world.has::<DynamicMesh>(e) {
            continue;
        }
        if stack.iter().all(|m| !m.enabled()) {
            if known.is_some() {
                world.remove::<DynamicMesh>(e);
                world.remove::<ModifierStateCell>(e);
            }
            continue;
        }
        let Some(source) = source_mesh(&key) else { continue };
        let version = world.get::<DynamicMesh>(e).map_or(0, |d| d.version) + 1;
        let mesh = build(source, &stack);
        world.insert(e, DynamicMesh { data: Arc::new(mesh), version });
        world.insert(e, ModifierStateCell(ModifierState { sig }));
    }
}

/// Runtime-only marker holding the build signature (never saved).
#[derive(Clone)]
pub struct ModifierStateCell(pub ModifierState);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_modifier_keeps_the_mesh_valid() {
        for name in Modifier::NAMES {
            let source = MeshData::primitive("sphere").unwrap();
            let before = source.vertices.len();
            let modifier = Modifier::named(name).unwrap();
            let out = build(source, &[Modifier::named("Subdivide").unwrap(), modifier]);
            assert!(out.vertices.len() >= before, "{name}");
            assert!(out.validate().is_ok(), "{name} produced an invalid mesh");
            assert!(out.vertices.iter().all(|v| v.pos.iter().all(|c| c.is_finite())), "{name} produced NaN");
        }
    }
}
