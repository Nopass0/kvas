//! Type-erased component table. The editor and scene loader work through this and
//! never name a concrete component, so adding a component = one `info::<T>()` line.

use crate::components::*;
use crate::reflect::Field;
use kvas_ecs::{Entity, World};
use serde_json::Value;

/// Type-erased editor/persistence operations for a registered component.
/// Callback functions are host-owned; they must outlive every registry using them.
pub struct ComponentInfo {
    /// Stable component name, matching the ECS storage name.
    pub name: &'static str,
    /// Whether user-facing removal should be permitted.
    pub removable: bool,
    /// Creates/checks the component storage in a fresh world using permanent host code.
    pub register: fn(&mut World),
    /// Inserts a default value, replacing an existing one.
    pub add: fn(&mut World, Entity),
    /// Removes the component if present (does not enforce `removable`).
    pub remove: fn(&mut World, Entity),
    /// Checks presence on a living entity.
    pub has: fn(&World, Entity) -> bool,
    /// Serializes a present component; missing/unserializable values yield `None`.
    pub to_json: fn(&World, Entity) -> Option<Value>,
    /// Deserializes and inserts a component, reporting malformed values.
    pub from_json: fn(&mut World, Entity, Value) -> Result<(), String>,
    /// Checks that a present value can round-trip through its persisted serde representation.
    /// This rejects non-finite built-in floats that JSON would otherwise replace with null.
    pub validate: fn(&World, Entity) -> Result<(), String>,
    /// Visits editable borrowed fields if the component exists.
    pub edit: for<'a> fn(&'a mut World, Entity, &mut dyn FnMut(Vec<Field<'a>>)),
}

fn info<T: Component>() -> ComponentInfo {
    ComponentInfo {
        name: <T as kvas_ecs::Component>::NAME,
        removable: T::REMOVABLE,
        register: |world| world.register::<T>(),
        add: |w, e| w.insert(e, T::default()),
        remove: |w, e| {
            w.remove::<T>(e);
        },
        has: |w, e| w.has::<T>(e),
        to_json: |w, e| w.get::<T>(e).and_then(|c| serde_json::to_value(c).ok()),
        from_json: |w, e, v| {
            w.insert(e, serde_json::from_value::<T>(v).map_err(|err| format!("{}: {err}", <T as kvas_ecs::Component>::NAME))?);
            Ok(())
        },
        validate: |world, entity| {
            let Some(component) = world.get::<T>(entity) else {
                return Ok(());
            };
            let json = serde_json::to_value(component).map_err(|e| e.to_string())?;
            serde_json::from_value::<T>(json).map(|_| ()).map_err(|e| format!("{}: {e}", <T as kvas_ecs::Component>::NAME))
        },
        edit: |w, e, f| {
            if let Some(c) = w.get_mut::<T>(e) {
                f(c.fields());
            }
        },
    }
}

/// Ordered built-in component reflection/serialization table.
/// This is distinct from ECS registration: use [`Self::register`] for host extensions.
pub struct Registry {
    /// Component metadata in Inspector/serialization order.
    pub components: Vec<ComponentInfo>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            components: vec![
                info::<Name>(),
                info::<Tags>(),
                info::<UiElement>(),
                info::<Lifetime>(),
                info::<Visibility>(),
                info::<MoveTween>(),
                info::<TransformAnimation>(),
                info::<NeuralAgent>(),
                info::<Transform>(),
                info::<MeshRenderer>(),
                info::<EditableMesh>(),
                info::<Shape>(),
                info::<Camera>(),
                info::<Light>(),
                info::<RigidBody>(),
                info::<Collider>(),
                info::<Joint>(),
                info::<Scripts>(),
                info::<Spin>(),
                info::<Bob>(),
                info::<Pulse>(),
                info::<Wobble>(),
                info::<ColorCycle>(),
                info::<Cloth>(),
                info::<Grass>(),
                info::<Tree>(),
                info::<Decal>(),
                info::<VoxelSmoke>(),
                info::<VoxelClouds>(),
                info::<VoxelWater>(),
                info::<Switch>(),
                info::<Slider>(),
                info::<AudioSource>(),
                info::<MeshModifiers>(),
                info::<TexturePaint>(),
                info::<ParticleEmitter>(),
            ],
        }
    }
}

impl Registry {
    /// Registers every metadata type in a fresh world through host-owned callbacks.
    /// Needed when loading scenes containing permanent host extension components.
    pub fn register_world(&self, world: &mut World) {
        for component in &self.components { (component.register)(world); }
    }

    /// Finds metadata by exact component name.
    pub fn get(&self, name: &str) -> Option<&ComponentInfo> {
        self.components.iter().find(|c| c.name == name)
    }

    /// Registers an editor-visible component in this table and a host world.
    /// Re-registering the same type is idempotent. Call only in permanent host code,
    /// because native library unloading would invalidate storage/callback vtables.
    pub fn register<T: Component>(&mut self, world: &mut World) {
        world.register::<T>();
        let name = <T as kvas_ecs::Component>::NAME;
        if self.get(name).is_none() {
            self.components.push(info::<T>());
        }
    }

    /// Metadata for components present on this entity, preserving registry order.
    pub fn of_entity<'a>(&'a self, world: &World, entity: Entity) -> Vec<&'a ComponentInfo> {
        self.components.iter().filter(|c| (c.has)(world, entity)).collect()
    }

    /// Reports registered component values that cannot round-trip through JSON.
    /// Unknown/runtime-only storages are ignored; stale handles return one diagnostic.
    pub fn validate_entity(&self, world: &World, entity: Entity) -> Vec<String> {
        if !world.is_alive(entity) {
            return vec!["entity is not alive".into()];
        }
        self.components.iter().filter_map(|component| (component.validate)(world, entity).err()).collect()
    }

    /// Inserts a default named component only when it is absent.
    /// Returns whether insertion occurred; errors report unknown types or stale handles.
    pub fn add_named(&self, world: &mut World, entity: Entity, name: &str) -> Result<bool, String> {
        if !world.is_alive(entity) {
            return Err("entity is not alive".into());
        }
        let info = self.get(name).ok_or_else(|| format!("unknown component '{name}'"))?;
        if (info.has)(world, entity) {
            return Ok(false);
        }
        (info.add)(world, entity);
        Ok(true)
    }

    /// Removes a named component, enforcing its Inspector removal policy.
    /// Returns whether a value existed. Unknown/stale/non-removable requests fail.
    pub fn remove_named(&self, world: &mut World, entity: Entity, name: &str) -> Result<bool, String> {
        if !world.is_alive(entity) {
            return Err("entity is not alive".into());
        }
        let info = self.get(name).ok_or_else(|| format!("unknown component '{name}'"))?;
        if !info.removable {
            return Err(format!("component '{name}' cannot be removed"));
        }
        let present = (info.has)(world, entity);
        (info.remove)(world, entity);
        Ok(present)
    }

    /// Copies every registered component from one entity to another (used for duplicate).
    pub fn copy_entity(&self, world: &mut World, from: Entity, to: Entity) {
        for c in &self.components {
            if let Some(v) = (c.to_json)(world, from) {
                let _ = (c.from_json)(world, to, v);
            }
        }
    }
}
