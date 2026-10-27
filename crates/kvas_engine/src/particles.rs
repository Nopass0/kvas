//! Particle emitters: fire, smoke, sparks, rain, snow, magic dust.
//!
//! A [`ParticleEmitter`] simulates its particles on the CPU and draws them as camera-facing quads
//! in one `DynamicMesh` (one draw call per emitter). Particles live in world space, so they trail
//! behind a moving emitter. Colours and fading are done by the built-in particle shader.
//!
//! ```
//! use kvas_engine::{prelude::*, scene::Scene, runtime::Runtime};
//! let mut scene = Scene::default();
//! let fire = scene.spawn("Campfire");
//! scene.world.insert(fire, ParticleEmitter::preset("fire"));
//! let mut runtime = Runtime::new(scene, None);
//! for _ in 0..30 { runtime.tick(1.0 / 30.0); }
//! assert!(runtime.scene.world.get::<ParticleEmitter>(fire).unwrap().alive() > 0);
//! ```

use crate::components::*;
use crate::mesh::{MeshData, Vertex};
use crate::procedural::Rng;
use crate::reflect::{AssetKind, Field};
use glam::{Mat4, Vec3};
use kvas_ecs::World;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

crate::choice_enum!(
    /// Where new particles appear.
    EmitterShape { Point, Sphere, Box, Cone, Disc } default Point
);

/// A particle emitter.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ParticleEmitter {
    /// Particles per second.
    pub rate: f32,
    /// Particles released at once when the emitter starts.
    pub burst: i32,
    /// Seconds a particle lives.
    pub lifetime: f32,
    /// Random +/- on the lifetime, 0..1.
    pub lifetime_variation: f32,
    /// Initial speed in metres per second.
    pub speed: f32,
    /// Random +/- on the speed, 0..1.
    pub speed_variation: f32,
    /// Spread of the launch direction around "up" (the emitter's local Y), in degrees.
    pub spread: f32,
    /// Acceleration in m/s^2 (negative Y falls, positive Y rises like smoke).
    pub gravity: f32,
    /// Air resistance: velocity lost per second, 0..1+.
    pub drag: f32,
    /// Size of a particle at birth, metres.
    pub size_start: f32,
    /// Size at the end of its life.
    pub size_end: f32,
    /// Colour at birth (RGBA).
    pub color_start: [f32; 4],
    /// Colour at the end of life (RGBA).
    pub color_end: [f32; 4],
    /// Brightness multiplier (glow), values above 1 make them shine.
    pub glow: f32,
    /// Where particles are born.
    pub shape: EmitterShape,
    /// Size of the emission area (radius of a sphere / disc / cone base, half-extent of a box).
    pub shape_size: f32,
    /// Particle image (a soft puff or spark with alpha). Empty = a soft round dot.
    pub texture: String,
    /// Keep emitting forever; off = one run of `duration` seconds.
    pub looping: bool,
    /// Length of one run when not looping.
    pub duration: f32,
    /// Upper limit of simultaneous particles.
    pub max_particles: i32,
    /// Random seed.
    pub seed: i32,
    #[serde(skip)]
    #[doc(hidden)]
    pub state: Option<ParticleState>,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            rate: 30.0,
            burst: 0,
            lifetime: 2.0,
            lifetime_variation: 0.3,
            speed: 2.0,
            speed_variation: 0.3,
            spread: 25.0,
            gravity: 0.0,
            drag: 0.2,
            size_start: 0.3,
            size_end: 0.05,
            color_start: [1.0, 0.8, 0.4, 1.0],
            color_end: [1.0, 0.2, 0.05, 0.0],
            glow: 1.5,
            shape: EmitterShape::Point,
            shape_size: 0.2,
            texture: String::new(),
            looping: true,
            duration: 5.0,
            max_particles: 600,
            seed: 7,
            state: None,
        }
    }
}

impl Component for ParticleEmitter {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::f32("rate", &mut self.rate),
            Field::i32("burst", &mut self.burst),
            Field::f32("lifetime", &mut self.lifetime),
            Field::f32("lifetime_variation", &mut self.lifetime_variation),
            Field::f32("speed", &mut self.speed),
            Field::f32("speed_variation", &mut self.speed_variation),
            Field::f32("spread", &mut self.spread),
            Field::f32("gravity", &mut self.gravity),
            Field::f32("drag", &mut self.drag),
            Field::f32("size_start", &mut self.size_start),
            Field::f32("size_end", &mut self.size_end),
            Field::color("color_start", &mut self.color_start),
            Field::color("color_end", &mut self.color_end),
            Field::f32("glow", &mut self.glow),
            Field::choice("shape", &mut self.shape),
            Field::f32("shape_size", &mut self.shape_size),
            Field::asset("texture", &mut self.texture, AssetKind::Texture),
            Field::bool("looping", &mut self.looping),
            Field::f32("duration", &mut self.duration),
            Field::i32("max_particles", &mut self.max_particles),
            Field::i32("seed", &mut self.seed),
        ]
    }
}

impl ParticleEmitter {
    /// Ready-made emitters: "fire", "smoke", "sparks", "snow", "rain", "magic", "dust", "explosion",
    /// "fog", "mist", "steam", "embers", "fireflies", "leaves", "bubbles", "campfire_smoke", "dust_storm".
    pub fn preset(name: &str) -> Self {
        let base = Self::default();
        match name {
            "fire" => Self { rate: 60.0, lifetime: 1.1, speed: 1.4, spread: 12.0, gravity: 2.5, drag: 0.4, size_start: 0.35, size_end: 0.05, shape: EmitterShape::Disc, shape_size: 0.25, color_start: [1.0, 0.75, 0.25, 0.9], color_end: [0.9, 0.1, 0.02, 0.0], glow: 3.0, ..base },
            "smoke" => Self { rate: 14.0, lifetime: 4.5, speed: 0.9, spread: 18.0, gravity: 0.6, drag: 0.5, size_start: 0.4, size_end: 2.2, shape: EmitterShape::Disc, shape_size: 0.2, color_start: [0.35, 0.35, 0.38, 0.55], color_end: [0.6, 0.6, 0.65, 0.0], glow: 0.4, ..base },
            "sparks" => Self { rate: 40.0, lifetime: 0.9, speed: 4.5, spread: 55.0, gravity: -9.0, drag: 0.1, size_start: 0.06, size_end: 0.02, color_start: [1.0, 0.85, 0.4, 1.0], color_end: [1.0, 0.3, 0.05, 0.0], glow: 4.0, ..base },
            "snow" => Self { rate: 120.0, lifetime: 7.0, speed: 0.2, spread: 180.0, gravity: -1.1, drag: 0.8, size_start: 0.07, size_end: 0.07, shape: EmitterShape::Box, shape_size: 10.0, color_start: [1.0, 1.0, 1.0, 0.9], color_end: [1.0, 1.0, 1.0, 0.0], glow: 1.2, max_particles: 1500, ..base },
            "rain" => Self { rate: 400.0, lifetime: 1.2, speed: 0.0, spread: 0.0, gravity: -18.0, drag: 0.0, size_start: 0.03, size_end: 0.03, shape: EmitterShape::Box, shape_size: 10.0, color_start: [0.7, 0.8, 1.0, 0.5], color_end: [0.7, 0.8, 1.0, 0.1], glow: 1.0, max_particles: 2500, ..base },
            "magic" => Self { rate: 35.0, lifetime: 2.2, speed: 0.7, spread: 180.0, gravity: 0.4, drag: 0.7, size_start: 0.12, size_end: 0.0, shape: EmitterShape::Sphere, shape_size: 0.6, color_start: [0.5, 0.8, 1.0, 1.0], color_end: [0.8, 0.3, 1.0, 0.0], glow: 4.0, ..base },
            "dust" => Self { rate: 25.0, lifetime: 6.0, speed: 0.15, spread: 180.0, gravity: 0.0, drag: 0.9, size_start: 0.04, size_end: 0.04, shape: EmitterShape::Box, shape_size: 4.0, color_start: [1.0, 0.95, 0.8, 0.5], color_end: [1.0, 0.95, 0.8, 0.0], glow: 1.5, ..base },
            "fog" => Self { rate: 6.0, lifetime: 14.0, lifetime_variation: 0.4, speed: 0.12, spread: 180.0, gravity: 0.0, drag: 0.3, size_start: 4.0, size_end: 7.0, shape: EmitterShape::Box, shape_size: 9.0, color_start: [0.78, 0.82, 0.88, 0.0], color_end: [0.78, 0.82, 0.88, 0.0], glow: 0.9, max_particles: 160, ..base },
            "mist" => Self { rate: 10.0, lifetime: 10.0, speed: 0.2, spread: 90.0, gravity: 0.0, drag: 0.4, size_start: 2.5, size_end: 4.5, shape: EmitterShape::Disc, shape_size: 7.0, color_start: [0.85, 0.9, 0.95, 0.18], color_end: [0.85, 0.9, 0.95, 0.0], glow: 1.0, max_particles: 220, ..base },
            "steam" => Self { rate: 22.0, lifetime: 2.4, speed: 1.2, spread: 14.0, gravity: 1.2, drag: 0.6, size_start: 0.2, size_end: 1.1, shape: EmitterShape::Disc, shape_size: 0.12, color_start: [0.95, 0.95, 0.97, 0.45], color_end: [0.95, 0.95, 0.97, 0.0], glow: 1.1, ..base },
            "embers" => Self { rate: 18.0, lifetime: 3.4, speed: 1.1, spread: 40.0, gravity: 1.4, drag: 0.35, size_start: 0.05, size_end: 0.01, shape: EmitterShape::Disc, shape_size: 0.5, color_start: [1.0, 0.55, 0.12, 1.0], color_end: [1.0, 0.1, 0.0, 0.0], glow: 5.0, ..base },
            "fireflies" => Self { rate: 6.0, lifetime: 7.0, speed: 0.25, spread: 180.0, gravity: 0.0, drag: 0.2, size_start: 0.07, size_end: 0.07, shape: EmitterShape::Box, shape_size: 6.0, color_start: [0.8, 1.0, 0.4, 0.9], color_end: [0.8, 1.0, 0.4, 0.0], glow: 6.0, max_particles: 80, ..base },
            "leaves" => Self { rate: 7.0, lifetime: 8.0, speed: 0.5, spread: 70.0, gravity: -0.8, drag: 1.2, size_start: 0.14, size_end: 0.14, shape: EmitterShape::Box, shape_size: 6.0, color_start: [0.8, 0.55, 0.15, 0.95], color_end: [0.6, 0.35, 0.1, 0.0], glow: 1.0, max_particles: 160, ..base },
            "bubbles" => Self { rate: 8.0, lifetime: 4.0, speed: 0.7, spread: 18.0, gravity: 0.5, drag: 0.3, size_start: 0.08, size_end: 0.16, shape: EmitterShape::Disc, shape_size: 0.4, color_start: [0.8, 0.95, 1.0, 0.5], color_end: [0.8, 0.95, 1.0, 0.0], glow: 1.4, ..base },
            "campfire_smoke" => Self { rate: 18.0, lifetime: 6.0, speed: 1.1, spread: 12.0, gravity: 0.7, drag: 0.35, size_start: 0.35, size_end: 2.6, shape: EmitterShape::Disc, shape_size: 0.25, color_start: [0.22, 0.22, 0.24, 0.5], color_end: [0.5, 0.5, 0.54, 0.0], glow: 0.6, ..base },
            "dust_storm" => Self { rate: 60.0, lifetime: 5.0, speed: 6.0, spread: 20.0, gravity: 0.0, drag: 0.2, size_start: 1.2, size_end: 3.0, shape: EmitterShape::Box, shape_size: 8.0, color_start: [0.85, 0.7, 0.45, 0.28], color_end: [0.85, 0.7, 0.45, 0.0], glow: 1.0, max_particles: 600, ..base },
            "explosion" => Self { rate: 0.0, burst: 140, lifetime: 1.4, speed: 7.0, spread: 180.0, gravity: -2.0, drag: 1.6, size_start: 0.6, size_end: 0.1, shape: EmitterShape::Sphere, shape_size: 0.2, color_start: [1.0, 0.7, 0.2, 1.0], color_end: [0.3, 0.1, 0.05, 0.0], glow: 3.5, looping: false, duration: 0.1, ..base },
            _ => base,
        }
    }

    /// Particles alive right now.
    pub fn alive(&self) -> usize {
        self.state.as_ref().map_or(0, |s| s.particles.len())
    }

    /// Restarts the emitter (a new burst, a new run).
    pub fn restart(&mut self) {
        self.state = None;
    }
}

#[derive(Clone)]
struct Particle {
    pos: Vec3,
    vel: Vec3,
    age: f32,
    life: f32,
}

/// Simulation state (internal).
#[doc(hidden)]
#[derive(Clone)]
pub struct ParticleState {
    particles: Vec<Particle>,
    rng: Rng,
    carry: f32,
    elapsed: f32,
    last_time: f32,
    version: u64,
}

impl ParticleState {
    fn new(seed: i32) -> Self {
        Self { particles: Vec::new(), rng: Rng::new(seed as u64), carry: 0.0, elapsed: 0.0, last_time: f32::NAN, version: 0 }
    }
}

fn random_in_cone(rng: &mut Rng, axis: Vec3, spread_deg: f32) -> Vec3 {
    let spread = spread_deg.clamp(0.0, 180.0).to_radians();
    let cos_t = 1.0 - rng.f32() * (1.0 - spread.cos());
    let sin_t = (1.0 - cos_t * cos_t).max(0.0).sqrt();
    let phi = rng.f32() * std::f32::consts::TAU;
    let u = axis.any_orthonormal_vector();
    let v = axis.cross(u);
    (axis * cos_t + (u * phi.cos() + v * phi.sin()) * sin_t).normalize_or_zero()
}

fn spawn(e: &ParticleEmitter, st: &mut ParticleState, world: &Mat4) {
    if st.particles.len() >= e.max_particles.clamp(1, 20_000) as usize {
        return;
    }
    let rng = &mut st.rng;
    let size = e.shape_size.max(0.0);
    let local = match e.shape {
        EmitterShape::Point => Vec3::ZERO,
        EmitterShape::Sphere => random_in_cone(rng, Vec3::Y, 180.0) * size * rng.f32().cbrt(),
        EmitterShape::Box => Vec3::new(rng.f32() * 2.0 - 1.0, rng.f32() * 2.0 - 1.0, rng.f32() * 2.0 - 1.0) * size,
        EmitterShape::Cone => {
            let a = rng.f32() * std::f32::consts::TAU;
            let r = rng.f32().sqrt() * size;
            Vec3::new(a.cos() * r, 0.0, a.sin() * r)
        }
        EmitterShape::Disc => {
            let a = rng.f32() * std::f32::consts::TAU;
            let r = rng.f32().sqrt() * size;
            Vec3::new(a.cos() * r, 0.0, a.sin() * r)
        }
    };
    let dir_local = random_in_cone(rng, Vec3::Y, e.spread);
    let speed = e.speed * (1.0 + (rng.f32() * 2.0 - 1.0) * e.speed_variation.clamp(0.0, 1.0));
    let life = (e.lifetime * (1.0 + (rng.f32() * 2.0 - 1.0) * e.lifetime_variation.clamp(0.0, 1.0))).max(0.05);
    st.particles.push(Particle { pos: world.transform_point3(local), vel: world.transform_vector3(dir_local).normalize_or_zero() * speed, age: 0.0, life });
}

fn step(e: &ParticleEmitter, st: &mut ParticleState, world: &Mat4, dt: f32) {
    st.elapsed += dt;
    let drag = (1.0 - e.drag.max(0.0) * dt).max(0.0);
    for p in &mut st.particles {
        p.age += dt;
        p.vel.y += e.gravity * dt;
        p.vel *= drag;
        p.pos += p.vel * dt;
    }
    st.particles.retain(|p| p.age < p.life);
    let emitting = e.looping || st.elapsed <= e.duration;
    if emitting && e.rate > 0.0 {
        st.carry += e.rate * dt;
        while st.carry >= 1.0 {
            st.carry -= 1.0;
            spawn(e, st, world);
        }
    }
}

fn camera_axes(world: &World) -> Option<(Vec3, Vec3)> {
    let (entity, _) = world.iter::<Camera>().find(|(_, c)| c.primary).or_else(|| world.iter::<Camera>().next())?;
    let m = world.get::<GlobalTransform>(entity)?.0;
    Some((m.x_axis.truncate().normalize_or_zero(), m.y_axis.truncate().normalize_or_zero()))
}

fn build_mesh(e: &ParticleEmitter, st: &ParticleState, to_local: &Mat4, axes: Option<(Vec3, Vec3)>) -> MeshData {
    let mut m = MeshData::default();
    m.vertices.reserve(st.particles.len() * 4);
    // Without a camera (editor view of a scene that has none) draw two crossed quads instead.
    let planes: Vec<(Vec3, Vec3)> = match axes {
        Some((r, u)) => vec![(r, u)],
        None => vec![(Vec3::X, Vec3::Y), (Vec3::Z, Vec3::Y)],
    };
    for p in &st.particles {
        let t = (p.age / p.life).clamp(0.0, 1.0);
        let size = e.size_start + (e.size_end - e.size_start) * t;
        let center = to_local.transform_point3(p.pos);
        for (right, up) in &planes {
            let (r, u) = (to_local.transform_vector3(*right) * size, to_local.transform_vector3(*up) * size);
            let base = m.vertices.len() as u32;
            for (qx, qy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                m.vertices.push(Vertex { pos: (center + r * qx + u * qy).into(), normal: [qx, qy, t], uv: [qx * 0.5 + 0.5, qy * 0.5 + 0.5] });
            }
            m.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    m
}

/// Simulates every emitter and refreshes its mesh and material. `simulate` is false in the editor
/// outside Play: emitters then animate from wall-clock `time` so they can be previewed.
pub fn update(world: &mut World, time: f32, dt: f32, simulate: bool) {
    for entity in world.entities_with::<ParticleEmitter>() {
        let Some(mut emitter) = world.get::<ParticleEmitter>(entity).cloned() else { continue };
        let global = world.get::<GlobalTransform>(entity).map_or(Mat4::IDENTITY, |g| g.0);
        let mut st = emitter.state.take().unwrap_or_else(|| ParticleState::new(emitter.seed));
        let first = st.last_time.is_nan();
        let delta = if simulate { dt } else if first { 0.0 } else { (time - st.last_time).clamp(0.0, 0.1) };
        st.last_time = time;
        if first {
            for _ in 0..emitter.burst.clamp(0, 5000) {
                spawn(&emitter, &mut st, &global);
            }
        }
        // Several small steps keep fast particles smooth at low frame rates.
        let steps = ((delta / 0.033).ceil() as usize).max(1);
        for _ in 0..steps {
            step(&emitter, &mut st, &global, delta / steps as f32);
        }
        st.version += 1;
        let inverse = if global.determinant().abs() > 1e-9 { global.inverse() } else { Mat4::IDENTITY };
        let mesh = build_mesh(&emitter, &st, &inverse, camera_axes(world));
        let version = st.version;
        emitter.state = Some(st);
        if let Some(slot) = world.get_mut::<ParticleEmitter>(entity) {
            *slot = emitter.clone();
        }
        world.insert(entity, DynamicMesh { data: Arc::new(mesh), version });
        if !world.has::<MeshRenderer>(entity) {
            world.insert(entity, MeshRenderer::default());
        }
        let m = world.get_mut::<MeshRenderer>(entity).unwrap();
        m.shader = "builtin:particle".into();
        m.cast_shadows = false;
        m.color = [1.0, 1.0, 1.0, 0.99];
        m.texture = emitter.texture.clone();
        m.emissive = emitter.glow;
        m.user = [
            emitter.color_start[0], emitter.color_start[1], emitter.color_start[2], emitter.color_start[3],
            emitter.color_end[0], emitter.color_end[1], emitter.color_end[2], emitter.color_end[3],
        ];
    }
}
