//! Convenience methods on [`Ctx`] for the things almost every script does: reading and setting the
//! position, turning towards a target, moving, measuring distances and picking random values.
//!
//! These only call the public `Ctx` API, so they are plain shortcuts; use `ctx.scene.world`
//! for anything they do not cover.
//!
//! ```
//! use kvas_engine::prelude::*;
//!
//! #[derive(Default)]
//! pub struct Chaser { speed: f32 }
//! impl Script for Chaser {
//!     fn update(&mut self, ctx: &mut Ctx) {
//!         if let Some(player) = ctx.find("Player") {
//!             if let Some(target) = ctx.position_of(player) {
//!                 ctx.look_at(target);
//!                 ctx.move_towards(target, self.speed);
//!             }
//!         }
//!     }
//! }
//! ```

use crate::components::Transform;
use crate::hierarchy;
use crate::script::Ctx;
use glam::Vec3;
use kvas_ecs::Entity;
use std::sync::atomic::{AtomicU64, Ordering};

static STREAM: AtomicU64 = AtomicU64::new(0x1234_5678_9ABC_DEF0);

impl Ctx<'_> {
    /// World position of this entity (alias of [`Ctx::world_position`]).
    pub fn position(&self) -> Vec3 {
        self.world_position()
    }

    /// World position of another entity, `None` when it is dead or has no [`Transform`].
    pub fn position_of(&self, entity: Entity) -> Option<Vec3> {
        if self.scene.world.is_alive(entity) && self.scene.world.has::<Transform>(entity) {
            Some(hierarchy::global_matrix(&self.scene.world, entity).w_axis.truncate())
        } else {
            None
        }
    }

    /// Moves this entity to a world position immediately (no physics teleport; see [`Ctx::teleport`]).
    pub fn set_position(&mut self, position: Vec3) -> bool {
        hierarchy::set_world_position(&mut self.scene.world, self.entity, position)
    }

    /// Adds `delta` to the local position (parent space axes).
    pub fn translate(&mut self, delta: Vec3) {
        self.transform_mut().position += delta;
    }

    /// Moves along the entity's own axes: `translate_local(Vec3::NEG_Z * 2.0)` goes 2 m forward.
    pub fn translate_local(&mut self, delta: Vec3) {
        let t = self.transform_mut();
        let q = t.quat();
        t.position += q * delta;
    }

    /// Adds Euler degrees (pitch, yaw, roll) to the local rotation.
    pub fn rotate(&mut self, degrees: Vec3) {
        self.transform_mut().rotation += degrees;
    }

    /// Facing direction (local -Z in parent space).
    pub fn forward(&self) -> Vec3 {
        self.transform().forward()
    }

    /// Right direction (local +X in parent space).
    pub fn right(&self) -> Vec3 {
        self.transform().right()
    }

    /// Up direction (local +Y in parent space).
    pub fn up(&self) -> Vec3 {
        self.transform().up()
    }

    /// Straight-line distance to another entity, `None` when it cannot be measured.
    pub fn distance_to(&self, entity: Entity) -> Option<f32> {
        self.position_of(entity).map(|p| p.distance(self.world_position()))
    }

    /// Unit vector from this entity to a world point, zero when they coincide.
    pub fn direction_to(&self, target: Vec3) -> Vec3 {
        (target - self.world_position()).normalize_or_zero()
    }

    /// Turns the entity (yaw and pitch, no roll) so that its forward axis points at a world point.
    /// Assumes the parent is not rotated. Does nothing when the point is at the entity.
    pub fn look_at(&mut self, target: Vec3) {
        let d = self.direction_to(target);
        if d == Vec3::ZERO {
            return;
        }
        let yaw = (-d.x).atan2(-d.z).to_degrees();
        let pitch = d.y.clamp(-1.0, 1.0).asin().to_degrees();
        self.transform_mut().rotation = Vec3::new(pitch, yaw, 0.0);
    }

    /// Like [`Ctx::look_at`] but ignores height: only the yaw changes (characters, turrets).
    pub fn look_at_flat(&mut self, target: Vec3) {
        let mut d = target - self.world_position();
        d.y = 0.0;
        if d.length_squared() < 1e-8 {
            return;
        }
        let yaw = (-d.x).atan2(-d.z).to_degrees();
        self.transform_mut().rotation.y = yaw;
    }

    /// Walks towards a world point at `speed` metres per second (uses the callback `dt`).
    /// Returns `true` once the point is reached. Moves the transform directly, so use it for
    /// kinematic objects; dynamic bodies should use `add_impulse` or `set_velocity`.
    pub fn move_towards(&mut self, target: Vec3, speed: f32) -> bool {
        let here = self.world_position();
        let delta = target - here;
        let step = speed.max(0.0) * self.dt();
        if delta.length() <= step.max(1e-5) {
            self.set_position(target);
            true
        } else {
            self.set_position(here + delta.normalize() * step);
            false
        }
    }

    /// The closest entity carrying `tag` (other than this one) and its distance.
    pub fn nearest_tagged(&self, tag: &str) -> Option<(Entity, f32)> {
        let here = self.world_position();
        self.tagged(tag)
            .into_iter()
            .filter(|e| *e != self.entity)
            .filter_map(|e| self.position_of(e).map(|p| (e, p.distance(here))))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Removes this entity (and its children) from the scene.
    pub fn despawn_self(&mut self) {
        let me = self.entity;
        self.despawn(me);
    }

    /// A random number in `0.0..1.0`. Every call advances a shared stream, so values differ per call.
    /// For a repeatable sequence keep a [`crate::random::Rng`] in your script instead.
    pub fn random(&self) -> f32 {
        let seed = STREAM.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed) ^ self.time.frame ^ self.entity.to_bits();
        crate::random::Rng::new(seed).f32()
    }

    /// A random number in `min..max`.
    pub fn random_range(&self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.random()
    }

    /// True with probability `p` (0..1) on each call.
    pub fn chance(&self, p: f32) -> bool {
        self.random() < p
    }
}
