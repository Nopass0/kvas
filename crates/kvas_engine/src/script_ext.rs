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

/// Hierarchy, tags and shared variables.
impl Ctx<'_> {
    /// The parent of this entity, if it has a living one.
    pub fn parent(&self) -> Option<Entity> {
        hierarchy::parent_of(&self.scene.world, self.entity)
    }

    /// The parent of any entity.
    pub fn parent_of(&self, entity: Entity) -> Option<Entity> {
        hierarchy::parent_of(&self.scene.world, entity)
    }

    /// The top-most ancestor (the entity itself when it has no parent).
    pub fn root(&self) -> Entity {
        let mut e = self.entity;
        while let Some(p) = hierarchy::parent_of(&self.scene.world, e) {
            e = p;
        }
        e
    }

    /// Every entity below `entity`: children, grandchildren and so on.
    pub fn descendants(&self, entity: Entity) -> Vec<Entity> {
        hierarchy::descendants(&self.scene.world, entity)
    }

    /// The first descendant of this entity with the given name.
    pub fn find_child(&self, name: &str) -> Option<Entity> {
        hierarchy::find_descendant(&self.scene.world, self.entity, name, false)
    }

    /// Entities carrying `tag` within `radius` metres of a world point, nearest first.
    pub fn tagged_within(&self, tag: &str, center: Vec3, radius: f32) -> Vec<Entity> {
        let mut found: Vec<(Entity, f32)> = self.tagged(tag).into_iter().filter_map(|e| self.position_of(e).map(|p| (e, p.distance(center)))).filter(|(_, d)| *d <= radius).collect();
        found.sort_by(|a, b| a.1.total_cmp(&b.1));
        found.into_iter().map(|(e, _)| e).collect()
    }

    /// All tags of an entity.
    pub fn tags_of(&self, entity: Entity) -> Vec<String> {
        self.scene.world.get::<crate::gameplay::Tags>(entity).map(|t| t.value.split_whitespace().map(str::to_string).collect()).unwrap_or_default()
    }

    /// Stores a value that every script can read with [`Ctx::global`]. Lives until the game stops.
    pub fn set_global(&mut self, key: &str, value: serde_json::Value) {
        self.scene.globals.insert(key.to_string(), value);
    }

    /// A shared value, if it was set.
    pub fn global(&self, key: &str) -> Option<&serde_json::Value> {
        self.scene.globals.get(key)
    }

    /// A shared number (integers are converted), or `default`.
    pub fn global_f32(&self, key: &str, default: f32) -> f32 {
        self.global(key).and_then(|v| v.as_f64()).map_or(default, |v| v as f32)
    }

    /// A shared flag, or `default`.
    pub fn global_bool(&self, key: &str, default: bool) -> bool {
        self.global(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    /// Adds to a shared number (starting from 0) and returns the new value: `ctx.add_global("score", 10.0)`.
    pub fn add_global(&mut self, key: &str, delta: f32) -> f32 {
        let value = self.global_f32(key, 0.0) + delta;
        self.set_global(key, serde_json::json!(value));
        value
    }
}

/// Vertex access and deformation (see [`crate::mesh_deform`]).
impl Ctx<'_> {
    /// Makes the entity's built-in primitive mesh (`primitive:sphere`...) editable so its vertices
    /// can be changed from scripts. Returns `true` when the entity is editable afterwards.
    pub fn make_editable(&mut self, entity: Entity) -> bool {
        use crate::components::MeshRenderer;
        use crate::mesh::MeshData;
        use crate::mesh_edit::EditableMesh;
        if self.scene.world.has::<EditableMesh>(entity) {
            return true;
        }
        let Some(name) = self.scene.world.get::<MeshRenderer>(entity).and_then(|m| m.mesh.strip_prefix("primitive:").map(str::to_string)) else { return false };
        let Some(editable) = MeshData::primitive(&name).and_then(|m| EditableMesh::from_mesh(&m).ok()) else { return false };
        self.scene.world.insert(entity, editable);
        true
    }

    /// The editable mesh of an entity, for deformation: `ctx.editable(e)?.twist(Vec3::Y, 30.0)`.
    pub fn editable(&mut self, entity: Entity) -> Option<&mut crate::mesh_edit::EditableMesh> {
        self.scene.world.get_mut::<crate::mesh_edit::EditableMesh>(entity)
    }
}
