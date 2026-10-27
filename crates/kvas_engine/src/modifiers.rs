//! Built-in behaviours you attach in the editor with no code. They run every frame in Play mode.
//! (Spin, Bob, Pulse, Wobble, ColorCycle.) Scripts can add, remove and tweak them like any component.

use crate::components::{Component, MeshRenderer, Transform};
use crate::noise::{Noise, NoiseKind};
use crate::reflect::Field;
use glam::Vec3;
use kvas_ecs::World;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Rotates continuously (degrees per second per axis).
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Spin {
    /// Euler rotation velocity in degrees per second for each local axis.
    pub speed: Vec3,
}

impl Default for Spin {
    fn default() -> Self {
        Self { speed: Vec3::new(0.0, 45.0, 0.0) }
    }
}

impl Component for Spin {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::vec3("speed", &mut self.speed)]
    }
}

/// Floats back and forth along an axis.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Bob {
    /// Parent-space oscillation direction; normalized by the update system.
    pub axis: Vec3,
    /// Maximum displacement from the initial position, in metres.
    pub amplitude: f32,
    /// Cycles per second.
    pub speed: f32,
    /// Initial oscillation phase in cycles.
    pub phase: f32,
    #[serde(skip)]
    #[doc(hidden)]
    pub origin: Option<Vec3>,
}

impl Default for Bob {
    fn default() -> Self {
        Self { axis: Vec3::Y, amplitude: 0.5, speed: 0.5, phase: 0.0, origin: None }
    }
}

impl Component for Bob {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::vec3("axis", &mut self.axis),
            Field::f32("amplitude", &mut self.amplitude),
            Field::f32("speed", &mut self.speed),
            Field::f32("phase", &mut self.phase),
        ]
    }
}

/// Breathes: scale oscillates around its starting value.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Pulse {
    /// 0.2 = +-20 % size.
    pub amount: f32,
    /// Scale oscillation frequency in cycles per second.
    pub speed: f32,
    #[serde(skip)]
    #[doc(hidden)]
    pub origin: Option<Vec3>,
}

impl Default for Pulse {
    fn default() -> Self {
        Self { amount: 0.15, speed: 1.0, origin: None }
    }
}

impl Component for Pulse {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("amount", &mut self.amount), Field::f32("speed", &mut self.speed)]
    }
}

/// Organic drift: moves around its starting position following procedural noise.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Wobble {
    /// Maximum noise displacement per parent-space axis, in metres.
    pub amount: Vec3,
    /// Temporal noise sampling frequency multiplier.
    pub frequency: f32,
    /// Noise family driving the displacement.
    pub kind: NoiseKind,
    /// Deterministic noise seed.
    pub seed: i32,
    #[serde(skip)]
    #[doc(hidden)]
    pub origin: Option<Vec3>,
}

impl Default for Wobble {
    fn default() -> Self {
        Self { amount: Vec3::splat(0.3), frequency: 0.7, kind: NoiseKind::Perlin, seed: 1, origin: None }
    }
}

impl Component for Wobble {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::vec3("amount", &mut self.amount),
            Field::f32("frequency", &mut self.frequency),
            Field::choice("noise", &mut self.kind),
            Field::i32("seed", &mut self.seed),
        ]
    }
}

/// Cycles the material colour through the hue wheel.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorCycle {
    /// Full hue cycles per second.
    pub speed: f32,
    /// HSV saturation used during cycling, normally in 0..1.
    pub saturation: f32,
    /// HSV brightness used during cycling, normally in 0..1.
    pub value: f32,
}

impl Default for ColorCycle {
    fn default() -> Self {
        Self { speed: 0.15, saturation: 0.75, value: 1.0 }
    }
}

impl Component for ColorCycle {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("speed", &mut self.speed), Field::f32("saturation", &mut self.saturation), Field::f32("value", &mut self.value)]
    }
}

/// Converts HSV to RGB. Hue wraps in cycles; saturation and value should be in 0..1.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let k = |n: f32| (n + h.rem_euclid(1.0) * 6.0).rem_euclid(6.0);
    let f = |n: f32| v - v * s * k(n).min(4.0 - k(n)).clamp(0.0, 1.0);
    [f(5.0), f(3.0), f(1.0)]
}

/// Advances every modifier by `dt`; `time` is seconds since the game started.
/// Non-finite time/delta and negative deltas are ignored.
pub fn update(world: &mut World, time: f32, dt: f32) {
    if !time.is_finite() || !dt.is_finite() || dt < 0.0 { return; }
    use kvas_ecs::Entity;
    let spinning: Vec<(Entity, Vec3)> = world.iter::<Spin>().map(|(e, s)| (e, s.speed)).collect();
    for (e, speed) in spinning {
        if let Some(t) = world.get_mut::<Transform>(e) {
            t.rotation += speed * dt;
        }
    }

    for e in world.entities_with::<Bob>() {
        let Some(pos) = world.get::<Transform>(e).map(|t| t.position) else { continue };
        let b = world.get_mut::<Bob>(e).unwrap();
        let origin = *b.origin.get_or_insert(pos);
        let offset = b.axis.normalize_or_zero() * b.amplitude * ((time * b.speed + b.phase) * TAU).sin();
        let target = origin + offset;
        world.get_mut::<Transform>(e).unwrap().position = target;
    }

    for e in world.entities_with::<Pulse>() {
        let Some(scale) = world.get::<Transform>(e).map(|t| t.scale) else { continue };
        let p = world.get_mut::<Pulse>(e).unwrap();
        let origin = *p.origin.get_or_insert(scale);
        let k = 1.0 + p.amount * (time * p.speed * TAU).sin();
        world.get_mut::<Transform>(e).unwrap().scale = origin * k;
    }

    for e in world.entities_with::<Wobble>() {
        let Some(pos) = world.get::<Transform>(e).map(|t| t.position) else { continue };
        let w = world.get_mut::<Wobble>(e).unwrap();
        let origin = *w.origin.get_or_insert(pos);
        let n = Noise::new(w.kind).with_seed(w.seed as u32).with_octaves(2);
        let t = time * w.frequency;
        let offset = Vec3::new(n.sample3(t, 0.0, 11.0), n.sample3(0.0, t, 23.0), n.sample3(37.0, 0.0, t)) * w.amount;
        world.get_mut::<Transform>(e).unwrap().position = origin + offset;
    }

    for e in world.entities_with::<ColorCycle>() {
        let c = world.get::<ColorCycle>(e).unwrap().clone();
        if let Some(m) = world.get_mut::<MeshRenderer>(e) {
            let rgb = hsv_to_rgb((time * c.speed).rem_euclid(1.0), c.saturation, c.value);
            m.color = [rgb[0], rgb[1], rgb[2], m.color[3]];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_primaries() {
        let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4);
        assert!(near(hsv_to_rgb(0.0, 1.0, 1.0), [1.0, 0.0, 0.0]));
        assert!(near(hsv_to_rgb(1.0 / 3.0, 1.0, 1.0), [0.0, 1.0, 0.0]));
        assert!(near(hsv_to_rgb(2.0 / 3.0, 1.0, 1.0), [0.0, 0.0, 1.0]));
    }
}
