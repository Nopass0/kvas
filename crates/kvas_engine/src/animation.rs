//! Serializable transform keyframes with quaternion interpolation, scrubbing and runtime playback.
use crate::{components::{Component, Transform}, reflect::Field};
use glam::Vec3;
use kvas_ecs::World;
use serde::{Deserialize, Serialize};

/// Local-space transform at an absolute clip time in seconds.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TransformKey {
    /// Seconds from clip start, finite and non-negative.
    pub time: f32,
    /// Local pose sampled at this time.
    pub pose: Transform,
}

/// Interpolation between adjacent keyframes.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Interpolation {
    /// Hold the previous pose until the next key.
    Step,
    /// Linear position/scale and shortest-path quaternion rotation.
    #[default]
    Linear,
    /// Smoothstep position/scale and quaternion rotation.
    Smooth,
}

/// Authorable local-transform clip. Animate static/kinematic entities; dynamic physics owns its pose.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TransformAnimation {
    /// Ordered poses; use [`Self::set_key`] when recording keys.
    pub keys: Vec<TransformKey>,
    /// Playback cursor in seconds; also used for editor scrubbing.
    pub time: f32,
    /// Playback multiplier; negative values play backwards.
    pub speed: f32,
    /// Repeat from the start after reaching the last key.
    pub looping: bool,
    /// Runtime playback switch.
    pub playing: bool,
    /// Curve used between keys.
    pub interpolation: Interpolation,
}

impl Default for TransformAnimation {
    fn default() -> Self { Self { keys: Vec::new(), time: 0.0, speed: 1.0, looping: true, playing: true, interpolation: Interpolation::Linear } }
}

impl Component for TransformAnimation {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::bool("playing", &mut self.playing), Field::bool("looping", &mut self.looping), Field::f32("speed", &mut self.speed)]
    }
}

impl TransformAnimation {
    /// Last key time, or zero for an empty clip.
    pub fn duration(&self) -> f32 { self.keys.iter().map(|k| k.time).filter(|t| t.is_finite()).fold(0.0, f32::max) }

    /// Inserts or replaces a key within 0.1 ms. Invalid times/poses leave the clip unchanged.
    pub fn set_key(&mut self, time: f32, pose: Transform) -> bool {
        if !time.is_finite() || time < 0.0 || !pose.is_finite() { return false; }
        self.keys.retain(|k| (k.time - time).abs() > 0.0001);
        self.keys.push(TransformKey { time, pose });
        self.keys.sort_by(|a, b| a.time.total_cmp(&b.time));
        true
    }

    /// Samples without changing playback state. Unsorted imported keys and duplicate times are tolerated.
    pub fn sample(&self, time: f32) -> Option<Transform> {
        if !time.is_finite() { return None; }
        let valid = |k: &&TransformKey| k.time.is_finite() && k.time >= 0.0 && k.pose.is_finite();
        let first = self.keys.iter().filter(valid).min_by(|a, b| a.time.total_cmp(&b.time))?;
        let a = self.keys.iter().filter(valid).filter(|k| k.time <= time).max_by(|a, b| a.time.total_cmp(&b.time)).unwrap_or(first);
        let Some(b) = self.keys.iter().filter(valid).filter(|k| k.time > time).min_by(|a, b| a.time.total_cmp(&b.time)) else { return Some(a.pose); };
        let mut t = ((time - a.time) / (b.time - a.time).max(1e-6)).clamp(0.0, 1.0);
        t = match self.interpolation { Interpolation::Step => 0.0, Interpolation::Linear => t, Interpolation::Smooth => t * t * (3.0 - 2.0 * t) };
        let mut pose = Transform { position: a.pose.position.lerp(b.pose.position, t), scale: a.pose.scale.lerp(b.pose.scale, t), rotation: Vec3::ZERO };
        pose.set_quat(a.pose.quat().slerp(b.pose.quat(), t));
        Some(pose)
    }
}

/// Advances all enabled clips. Disabled clips leave their transform untouched.
pub fn update(world: &mut World, dt: f32) {
    if !dt.is_finite() || dt < 0.0 { return; }
    for e in world.entities_with::<TransformAnimation>() {
        let pose = {
            let Some(clip) = world.get_mut::<TransformAnimation>(e) else { continue };
            if !clip.playing || !clip.speed.is_finite() || !clip.time.is_finite() { continue; }
            let duration = clip.duration();
            clip.time += dt * clip.speed;
            if duration > 0.0 && clip.looping { clip.time = clip.time.rem_euclid(duration); }
            else if clip.time >= duration || clip.time < 0.0 { clip.time = clip.time.clamp(0.0, duration); clip.playing = false; }
            clip.sample(clip.time)
        };
        if let (Some(pose), Some(target)) = (pose, world.get_mut::<Transform>(e)) { *target = pose; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys_interpolate_shortest_rotation_and_reject_invalid_data() {
        let mut clip = TransformAnimation::default();
        let mut a = Transform::at(Vec3::ZERO); a.rotation.y = 350.0;
        let mut b = Transform::at(Vec3::X * 10.0); b.rotation.y = 10.0;
        assert!(clip.set_key(2.0, b)); assert!(clip.set_key(0.0, a));
        let mid = clip.sample(1.0).unwrap();
        assert!((mid.position.x - 5.0).abs() < 1e-5);
        assert!(mid.forward().dot(glam::Vec3::NEG_Z) > 0.999);
        assert!(!clip.set_key(f32::NAN, a));
        clip.interpolation = Interpolation::Step;
        assert_eq!(clip.sample(1.0).unwrap().position, Vec3::ZERO);
    }
    #[test]
    fn persisted_clip_runs_and_stops_in_the_shared_runtime() {
        let mut scene = crate::scene::Scene::default(); let e = scene.spawn("animated");
        let mut clip = TransformAnimation { looping: false, ..Default::default() };
        clip.set_key(0.0, Transform::default()); clip.set_key(1.0, Transform::at(Vec3::X));
        scene.world.insert(e, clip);
        let reg = crate::registry::Registry::default();
        let (scene, warnings) = crate::scene::Scene::from_json(&reg, &scene.to_json(&reg)).unwrap(); assert!(warnings.is_empty());
        let mut rt = crate::runtime::Runtime::new(scene, None);
        for _ in 0..12 { rt.tick(0.1); }
        let e = crate::hierarchy::find_by_name(&rt.scene.world, "animated").unwrap();
        assert_eq!(rt.scene.world.get::<Transform>(e).unwrap().position, Vec3::X);
        assert!(!rt.scene.world.get::<TransformAnimation>(e).unwrap().playing);
    }
}
