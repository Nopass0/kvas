//! Small built-in gameplay components usable from both the editor and native scripts.
//!
//! [`Tags`] provides non-unique group labels, [`Lifetime`] removes temporary
//! subtrees, and [`Visibility`] hides a rendered subtree while gameplay continues.
//!
//! ```
//! use kvas_engine::{scene::Scene, gameplay::{Tags, Lifetime}, runtime::Runtime};
//! let mut scene = Scene::default();
//! let effect = scene.spawn("temporary effect");
//! let mut tags = Tags::default();
//! assert!(tags.insert("effect"));
//! scene.world.insert(effect, tags);
//! scene.world.insert(effect, Lifetime::new(0.02).unwrap());
//! let mut runtime = Runtime::new(scene, None);
//! runtime.tick(0.03);
//! assert!(!runtime.scene.world.is_alive(effect));
//! ```

use crate::components::Component;
use crate::hierarchy;
use crate::reflect::Field;
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};

/// Case-sensitive whitespace-separated entity group labels.
/// Tags are non-unique across entities; tag strings cannot contain whitespace.
/// Duplicate/extra whitespace entered in the Inspector is tolerated on reads.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Tags {
    /// Editable tag list, e.g. `enemy damageable patrol`.
    pub value: String,
}

impl Tags {
    /// Whether the exact tag is present. Empty strings never match.
    pub fn contains(&self, tag: &str) -> bool {
        self.iter().any(|current| current == tag)
    }
    /// Borrows tag tokens in stored order (manually entered duplicates are preserved).
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.value.split_whitespace()
    }
    /// Inserts a nonempty tag without whitespace; returns false for invalid/duplicate tags.
    pub fn insert(&mut self, tag: &str) -> bool {
        if tag.is_empty() || tag.chars().any(char::is_whitespace) || self.contains(tag) {
            return false;
        }
        if !self.value.is_empty() {
            self.value.push(' ');
        }
        self.value.push_str(tag);
        true
    }
    /// Removes every occurrence, normalizing remaining whitespace. Returns whether found.
    pub fn remove(&mut self, tag: &str) -> bool {
        if !self.contains(tag) {
            return false;
        }
        self.value = self.iter().filter(|&current| current != tag).collect::<Vec<_>>().join(" ");
        true
    }
}

impl Component for Tags {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::text("tags", &mut self.value)]
    }
}

/// Remaining simulation lifetime of an entity and its complete subtree.
/// Updated once per accepted runtime frame after scripts. Pausing/time scaling
/// applies naturally. Zero or invalid remaining time expires on the next update.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Lifetime {
    /// Remaining seconds; serialized so an edited scene can provide the initial lifetime.
    pub seconds: f32,
}

impl Default for Lifetime {
    fn default() -> Self {
        Self { seconds: 5.0 }
    }
}

impl Lifetime {
    /// Creates a finite non-negative lifetime, or `None` for invalid input.
    pub fn new(seconds: f32) -> Option<Self> {
        (seconds.is_finite() && seconds >= 0.0).then_some(Self { seconds })
    }
}

impl Component for Lifetime {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("seconds", &mut self.seconds)]
    }
}

/// Rendering visibility inherited by descendants; it does not disable scripts or physics.
/// An absent component behaves as visible. A hidden ancestor hides all descendants,
/// including transient procedural meshes. Lights remain active independently.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Visibility {
    /// Whether this entity allows its own and descendant meshes to render.
    pub visible: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

impl Component for Visibility {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::bool("visible", &mut self.visible)]
    }
}

/// Snapshot of all entities containing an exact tag, in entity index order.
pub fn tagged(world: &World, tag: &str) -> Vec<Entity> {
    let mut entities: Vec<_> = world.iter::<Tags>().filter(|(_, tags)| tags.contains(tag)).map(|(e, _)| e).collect();
    entities.sort();
    entities
}

/// Advances remaining lifetimes and removes expired subtrees before physics.
/// Non-positive/non-finite deltas are ignored; malformed lifetimes expire on a valid update.
pub fn update_lifetimes(world: &mut World, dt: f32) {
    if !dt.is_finite() || dt <= 0.0 {
        return;
    }
    let mut expired = Vec::new();
    for (entity, lifetime) in world.iter_mut::<Lifetime>() {
        lifetime.seconds -= dt;
        if !lifetime.seconds.is_finite() || lifetime.seconds <= 0.0 {
            expired.push(entity);
        }
    }
    for entity in expired {
        hierarchy::despawn_recursive(world, entity);
    }
}
