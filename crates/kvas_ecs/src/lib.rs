//! Minimal ECS: generational entities + sparse-set component storage.
//! Each component type lives in its own densely packed array, so iteration is cache friendly
//! and add/remove/lookup are O(1).
//!
//! Component types are identified by a compile-time hash of `Component::NAME` instead of `TypeId`.
//! `TypeId`s differ between separately compiled binaries (a game's script library vs. the editor),
//! names do not, which is what lets scripts share a `World` with the host.
//! Storages are created by `register`, which the host calls; code in a hot-reloaded library never
//! creates one (its vtable would dangle after unloading).
//!
//! # Example
//! ```
//! use kvas_ecs::{Component, World};
//! struct Health(u32);
//! impl Component for Health { const NAME: &'static str = "example.Health"; }
//! let mut world = World::new();
//! world.register::<Health>();
//! let entity = world.spawn();
//! world.insert(entity, Health(100));
//! for (_, health) in world.iter_mut::<Health>() { health.0 -= 1; }
//! assert_eq!(world.get::<Health>(entity).unwrap().0, 99);
//! world.despawn(entity);
//! assert!(!world.is_alive(entity));
//! ```

#![warn(missing_docs)]

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// Implemented once per component type. `NAME` must be unique within a world.
pub trait Component: 'static {
    /// Stable, globally unique name; keep it identical in the host and script library.
    const NAME: &'static str;
    /// Storage key derived from [`Self::NAME`]. Normally should not be overridden.
    const KEY: u64 = fnv1a(Self::NAME);
}

const fn fnv1a(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
        i += 1;
    }
    hash
}

/// Keys are already hashes; hashing them again would only cost time.
#[derive(Default)]
struct KeyHasher(u64);

impl Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("only u64 keys are used")
    }
    fn write_u64(&mut self, v: u64) {
        self.0 = v;
    }
}

type KeyMap<V> = HashMap<u64, V, BuildHasherDefault<KeyHasher>>;

/// Generational handle to an entity in one [`World`].
///
/// Handles are world-local, not persistent scene IDs. A recycled index receives
/// a different generation, so stale handles cannot access the replacement.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Entity {
    index: u32,
    gen: u32,
}

impl Entity {
    /// Slot index, useful for display. Alone it does not identify a living entity.
    pub fn index(self) -> u32 {
        self.index
    }

    /// Generation distinguishing successive occupants of the same slot.
    pub fn generation(self) -> u32 {
        self.gen
    }

    /// Opaque world-local identity, including the generation.
    ///
    /// This value is suitable for temporary lookup keys, not scene persistence.
    pub fn to_bits(self) -> u64 {
        ((self.gen as u64) << 32) | self.index as u64
    }
}

const NONE: u32 = u32::MAX;

/// Type-erased view of a storage, used for operations that don't know `T`.
trait AnyStorage {
    fn remove_entity(&mut self, index: u32);
    fn name(&self) -> &'static str;
    fn type_info(&self) -> (&'static str, usize, usize);
}

struct SparseSet<T> {
    sparse: Vec<u32>,
    entities: Vec<u32>,
    data: Vec<T>,
}

impl<T> SparseSet<T> {
    fn new() -> Self {
        Self { sparse: Vec::new(), entities: Vec::new(), data: Vec::new() }
    }

    fn slot(&self, index: u32) -> Option<usize> {
        match self.sparse.get(index as usize) {
            Some(&s) if s != NONE => Some(s as usize),
            _ => None,
        }
    }

    fn insert(&mut self, index: u32, value: T) -> Option<T> {
        if let Some(slot) = self.slot(index) {
            return Some(std::mem::replace(&mut self.data[slot], value));
        }
        if self.sparse.len() <= index as usize {
            self.sparse.resize(index as usize + 1, NONE);
        }
        self.sparse[index as usize] = self.data.len() as u32;
        self.entities.push(index);
        self.data.push(value);
        None
    }

    fn remove(&mut self, index: u32) -> Option<T> {
        let slot = self.slot(index)?;
        self.sparse[index as usize] = NONE;
        let last = self.entities.len() - 1;
        self.entities.swap(slot, last);
        let value = self.data.swap_remove(slot);
        self.entities.pop();
        if slot != last {
            self.sparse[self.entities[slot] as usize] = slot as u32;
        }
        Some(value)
    }
}

impl<T: Component> AnyStorage for SparseSet<T> {
    fn remove_entity(&mut self, index: u32) {
        self.remove(index);
    }
    fn name(&self) -> &'static str {
        T::NAME
    }
    fn type_info(&self) -> (&'static str, usize, usize) {
        (std::any::type_name::<T>(), std::mem::size_of::<T>(), std::mem::align_of::<T>())
    }
}

/// Collection of entities and registered sparse component storages.
///
/// Register component types before insertion. Iteration order within a storage
/// changes on removal; use [`Self::entities`] for deterministic index order.
/// Built-in engine worlds are initialized by `Scene::default`.
#[derive(Default)]
pub struct World {
    live_count: usize,
    generations: Vec<u32>,
    alive: Vec<bool>,
    free: Vec<u32>,
    storages: KeyMap<Box<dyn AnyStorage>>,
}

impl World {
    /// Creates an empty world with no registered component types.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the storage for `T`. Must be called (by the host) before `T` is inserted.
    pub fn register<T: Component>(&mut self) {
        let existing = self.storages.entry(T::KEY).or_insert_with(|| Box::new(SparseSet::<T>::new()));
        assert_eq!(existing.name(), T::NAME, "component name hash collision or duplicate name");
        Self::check_storage::<T>(&**existing);
    }

    fn check_storage<T: Component>(storage: &dyn AnyStorage) {
        assert_eq!(storage.name(), T::NAME, "component key collision");
        assert_eq!(
            storage.type_info(),
            (std::any::type_name::<T>(), std::mem::size_of::<T>(), std::mem::align_of::<T>()),
            "different component types share a storage key"
        );
    }

    /// Whether the host registered this exact component type.
    ///
    /// # Panics
    /// Panics if the key belongs to an incompatible component type.
    pub fn is_registered<T: Component>(&self) -> bool {
        self.set::<T>().is_some()
    }

    /// Number of living entities, including entities without any components.
    pub fn len(&self) -> usize {
        self.live_count
    }

    /// Whether no living entities remain. Registered storages are not counted.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a living entity without components, reusing a vacant slot when possible.
    pub fn spawn(&mut self) -> Entity {
        assert!(!self.free.is_empty() || self.generations.len() < NONE as usize, "entity index space exhausted");
        self.live_count += 1;
        if let Some(index) = self.free.pop() {
            self.alive[index as usize] = true;
            Entity { index, gen: self.generations[index as usize] }
        } else {
            self.generations.push(0);
            self.alive.push(true);
            Entity { index: self.generations.len() as u32 - 1, gen: 0 }
        }
    }

    /// Removes an entity and all its components. Stale handles are ignored.
    ///
    /// This ECS operation does not traverse engine hierarchy relations.
    pub fn despawn(&mut self, e: Entity) {
        if !self.is_alive(e) {
            return;
        }
        for storage in self.storages.values_mut() {
            storage.remove_entity(e.index);
        }
        self.alive[e.index as usize] = false;
        self.live_count -= 1;
        // Exhausted slots are retired instead of wrapping and reviving a stale handle.
        if let Some(next) = self.generations[e.index as usize].checked_add(1) {
            self.generations[e.index as usize] = next;
            self.free.push(e.index);
        }
    }

    /// Checks both slot occupancy and generation. Handles from other worlds are unsupported.
    pub fn is_alive(&self, e: Entity) -> bool {
        self.alive.get(e.index as usize) == Some(&true) && self.generations[e.index as usize] == e.gen
    }

    /// All living entities in index order.
    pub fn entities(&self) -> Vec<Entity> {
        self.iter_entities().collect()
    }

    /// Borrows all living handles in index order without allocating a snapshot.
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        (0..self.alive.len() as u32)
            .filter(|&i| self.alive[i as usize])
            .map(|index| Entity { index, gen: self.generations[index as usize] })
    }

    /// Destroys every entity while preserving registrations and allocated storage.
    /// All previously issued handles become stale.
    pub fn clear(&mut self) {
        for e in self.entities() {
            self.despawn(e);
        }
    }

    fn set<T: Component>(&self) -> Option<&SparseSet<T>> {
        let storage = self.storages.get(&T::KEY)?;
        Self::check_storage::<T>(&**storage);
        // SAFETY: the storage under `T::KEY` was created by `register::<T>()` (the name is checked
        // there), so its concrete type is `SparseSet<T>`.
        Some(unsafe { &*(&**storage as *const dyn AnyStorage as *const SparseSet<T>) })
    }

    fn set_mut<T: Component>(&mut self) -> Option<&mut SparseSet<T>> {
        let storage = self.storages.get_mut(&T::KEY)?;
        Self::check_storage::<T>(&**storage);
        // SAFETY: see `set`.
        Some(unsafe { &mut *(&mut **storage as *mut dyn AnyStorage as *mut SparseSet<T>) })
    }

    /// Inserts or replaces a component. Stale entities are ignored.
    ///
    /// # Panics
    /// Panics if a living entity is used before [`Self::register`] for `T`,
    /// or the component key conflicts with an incompatible registered type.
    pub fn insert<T: Component>(&mut self, e: Entity, value: T) {
        if !self.is_alive(e) {
            return;
        }
        match self.set_mut::<T>() {
            Some(set) => {
                set.insert(e.index, value);
            }
            None => panic!("component '{}' is not registered in this World", T::NAME),
        }
    }

    /// Inserts or replaces a component without panicking for stale entities or
    /// unregistered types. On success returns the replaced value, if any; on
    /// rejection returns ownership of the supplied value unchanged.
    /// Conflicting component-key metadata still panics before any erased cast.
    pub fn try_insert<T: Component>(&mut self, entity: Entity, value: T) -> Result<Option<T>, T> {
        if !self.is_alive(entity) {
            return Err(value);
        }
        match self.set_mut::<T>() {
            Some(storage) => Ok(storage.insert(entity.index, value)),
            None => Err(value),
        }
    }

    /// Removes and returns a component, or `None` for a stale entity or absent component.
    pub fn remove<T: Component>(&mut self, e: Entity) -> Option<T> {
        if !self.is_alive(e) {
            return None;
        }
        self.set_mut::<T>()?.remove(e.index)
    }

    /// Whether a living entity has `T`.
    pub fn has<T: Component>(&self, e: Entity) -> bool {
        self.get::<T>(e).is_some()
    }

    /// Borrows a component; returns `None` for stale entities and absent types.
    pub fn get<T: Component>(&self, e: Entity) -> Option<&T> {
        if !self.is_alive(e) {
            return None;
        }
        let set = self.set::<T>()?;
        set.slot(e.index).map(|s| &set.data[s])
    }

    /// Mutably borrows a component; returns `None` for stale entities and absent types.
    pub fn get_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        if !self.is_alive(e) {
            return None;
        }
        let set = self.set_mut::<T>()?;
        let slot = set.slot(e.index)?;
        Some(&mut set.data[slot])
    }

    /// Borrows two components of one living entity. Repeated component types are allowed.
    pub fn get2<A: Component, B: Component>(&self, entity: Entity) -> Option<(&A, &B)> {
        Some((self.get::<A>(entity)?, self.get::<B>(entity)?))
    }

    /// Mutably borrows two distinct component storages on one living entity.
    /// Returns `None` for stale entities, missing components, unregistered types
    /// or identical storage keys (which would otherwise alias mutable references).
    ///
    /// # Panics
    /// Panics if component metadata conflicts with a registered storage key.
    pub fn get2_mut<A: Component, B: Component>(&mut self, entity: Entity) -> Option<(&mut A, &mut B)> {
        if !self.is_alive(entity) || A::KEY == B::KEY {
            return None;
        }
        let [a, b] = self.storages.get_disjoint_mut([&A::KEY, &B::KEY]);
        let (a, b) = (a?, b?);
        Self::check_storage::<A>(&**a);
        Self::check_storage::<B>(&**b);
        // SAFETY: disjoint map entries guarantee separate exclusive storages.
        // Registration and metadata checks match each entry to its component type.
        let a = unsafe { &mut *(&mut **a as *mut dyn AnyStorage as *mut SparseSet<A>) };
        let b = unsafe { &mut *(&mut **b as *mut dyn AnyStorage as *mut SparseSet<B>) };
        let (a_slot, b_slot) = (a.slot(entity.index)?, b.slot(entity.index)?);
        Some((&mut a.data[a_slot], &mut b.data[b_slot]))
    }

    /// Applies a callback to every entity containing two distinct mutable component types.
    /// Returns the processed count; identical storage keys process zero entities.
    /// Structural changes are intentionally unavailable inside the callback.
    pub fn for_each2_mut<A: Component, B: Component>(&mut self, mut visit: impl FnMut(Entity, &mut A, &mut B)) -> usize {
        if A::KEY == B::KEY {
            return 0;
        }
        let entities = self.entities_with2::<A, B>();
        for &entity in &entities {
            if let Some((a, b)) = self.get2_mut::<A, B>(entity) {
                visit(entity, a, b);
            }
        }
        entities.len()
    }

    /// Removes `T` values rejected by a mutable predicate, retaining all entities.
    /// Returns the number removed. Dense storage order may change during removals.
    pub fn retain<T: Component>(&mut self, mut keep: impl FnMut(Entity, &mut T) -> bool) -> usize {
        let mut removed = 0;
        for entity in self.entities_with::<T>() {
            let retain = self.get_mut::<T>(entity).is_some_and(|value| keep(entity, value));
            if !retain {
                self.remove::<T>(entity);
                removed += 1;
            }
        }
        removed
    }

    fn entity_of(&self, index: u32) -> Entity {
        Entity { index, gen: self.generations[index as usize] }
    }

    /// Iterates a dense storage without allocation. An unregistered type yields no items.
    pub fn iter<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.set::<T>().into_iter().flat_map(|s| s.entities.iter().zip(s.data.iter())).map(|(&i, v)| (self.entity_of(i), v))
    }

    /// Mutably iterates a dense storage; structural world changes require a snapshot instead.
    pub fn iter_mut<T: Component>(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        let gens = &self.generations;
        self.storages
            .get_mut(&T::KEY)
            .map(|storage| {
                Self::check_storage::<T>(&**storage);
                // SAFETY: see `set`. (Inlined because `set_mut` would borrow all of `self`.)
                unsafe { &mut *(&mut **storage as *mut dyn AnyStorage as *mut SparseSet<T>) }
            })
            .into_iter()
            .flat_map(|s| s.entities.iter().zip(s.data.iter_mut()))
            .map(|(&i, v)| (Entity { index: i, gen: gens[i as usize] }, v))
    }

    /// Entities that have component `T` (snapshot, safe to mutate the world while looping).
    pub fn entities_with<T: Component>(&self) -> Vec<Entity> {
        self.iter::<T>().map(|(e, _)| e).collect()
    }

    /// Entities that have both `A` and `B`.
    pub fn entities_with2<A: Component, B: Component>(&self) -> Vec<Entity> {
        self.iter2::<A, B>().map(|(e, _, _)| e).collect()
    }

    /// Borrows entities containing both component types, in `A` storage order.
    /// Matching a type with itself is supported because both borrows are shared.
    pub fn iter2<A: Component, B: Component>(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        self.iter::<A>().filter_map(|(e, a)| self.get::<B>(e).map(|b| (e, a, b)))
    }

    /// Borrows entities containing all three component types, in `A` storage order.
    pub fn iter3<A: Component, B: Component, C: Component>(&self) -> impl Iterator<Item = (Entity, &A, &B, &C)> {
        self.iter2::<A, B>().filter_map(|(e, a, b)| self.get::<C>(e).map(|c| (e, a, b, c)))
    }

    /// Snapshot of entities containing all three component types.
    pub fn entities_with3<A: Component, B: Component, C: Component>(&self) -> Vec<Entity> {
        self.iter3::<A, B, C>().map(|(e, _, _, _)| e).collect()
    }

    /// Snapshot of entities with `A` but without `B`, in `A` storage order.
    pub fn entities_without<A: Component, B: Component>(&self) -> Vec<Entity> {
        self.iter::<A>().map(|(e, _)| e).filter(|&e| !self.has::<B>(e)).collect()
    }

    /// Inserts a lazily constructed component only when missing, then borrows it.
    /// Returns `None` without invoking `create` for stale entities.
    ///
    /// # Panics
    /// Panics if `T` is unregistered on a living entity.
    pub fn get_or_insert_with<T: Component>(&mut self, e: Entity, create: impl FnOnce() -> T) -> Option<&mut T> {
        if !self.is_alive(e) {
            return None;
        }
        if !self.has::<T>(e) {
            self.insert(e, create());
        }
        self.get_mut::<T>(e)
    }

    /// Removes every `T`, preserving entities and the registered storage.
    pub fn clear_components<T: Component>(&mut self) {
        for e in self.entities_with::<T>() {
            self.remove::<T>(e);
        }
    }

    /// Number of entities with `T`, or zero for an unregistered type.
    pub fn count<T: Component>(&self) -> usize {
        self.set::<T>().map_or(0, |s| s.data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Hp(i32);
    struct Tag(#[allow(dead_code)] &'static str);
    impl Component for Hp {
        const NAME: &'static str = "Hp";
    }
    impl Component for Tag {
        const NAME: &'static str = "Tag";
    }

    fn world() -> World {
        let mut w = World::new();
        w.register::<Hp>();
        w.register::<Tag>();
        w
    }

    #[test]
    fn spawn_insert_remove() {
        let mut w = world();
        let a = w.spawn();
        let b = w.spawn();
        w.insert(a, Hp(1));
        w.insert(b, Hp(2));
        w.insert(b, Tag("b"));
        assert_eq!(w.get::<Hp>(a).map(|h| h.0), Some(1));
        assert_eq!(w.remove::<Hp>(a).map(|h| h.0), Some(1));
        assert_eq!(w.get::<Hp>(b).map(|h| h.0), Some(2)); // swap-remove must keep b valid
        w.despawn(b);
        assert!(!w.is_alive(b));
        assert!(w.get::<Hp>(b).is_none());
        let c = w.spawn(); // reuses b's index with a new generation
        assert!(w.get::<Tag>(c).is_none());
        assert!(!w.is_alive(b));
    }

    #[test]
    fn iteration_and_queries() {
        let mut w = world();
        let es: Vec<_> = (0..5).map(|_| w.spawn()).collect();
        for (i, &e) in es.iter().enumerate() {
            w.insert(e, Hp(i as i32));
            if i % 2 == 0 {
                w.insert(e, Tag("even"));
            }
        }
        assert_eq!(w.count::<Hp>(), 5);
        assert_eq!(w.entities_with2::<Hp, Tag>().len(), 3);
        for (_, hp) in w.iter_mut::<Hp>() {
            hp.0 *= 10;
        }
        assert_eq!(w.iter::<Hp>().map(|(_, h)| h.0).sum::<i32>(), 100);
    }

    #[test]
    #[should_panic(expected = "not registered")]
    fn inserting_an_unregistered_component_is_a_clear_error() {
        let mut w = World::new();
        let e = w.spawn();
        w.insert(e, Hp(1));
    }

    #[test]
    fn clear_invalidates_handles_but_preserves_registrations() {
        let mut w = world();
        let e = w.spawn();
        w.insert(e, Hp(5));
        let bits = e.to_bits();
        w.clear();
        assert!(w.is_empty());
        assert!(w.is_registered::<Hp>());
        let next = w.spawn();
        assert_eq!(e.index(), next.index());
        assert_ne!(e.generation(), next.generation());
        assert_ne!(bits, next.to_bits());
        assert!(w.get::<Hp>(e).is_none());
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn lazy_insertion_and_join_queries_handle_missing_and_repeated_types() {
        let mut w = world();
        let a = w.spawn();
        let b = w.spawn();
        w.get_or_insert_with(a, || Hp(3)).unwrap().0 += 1;
        w.get_or_insert_with::<Hp>(a, || panic!("must be lazy"));
        w.insert(a, Tag("a"));
        w.insert(b, Hp(7));
        assert_eq!(w.iter2::<Hp, Tag>().count(), 1);
        assert_eq!(w.iter3::<Hp, Tag, Hp>().next().unwrap().1 .0, 4);
        assert_eq!(w.entities_without::<Hp, Tag>(), vec![b]);
        w.despawn(a);
        assert!(w.get_or_insert_with::<Hp>(a, || panic!("stale")).is_none());
        w.clear_components::<Hp>();
        assert_eq!(w.count::<Hp>(), 0);
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn conflicting_component_names_panic_before_any_type_cast() {
        struct Imposter(u64);
        impl Component for Imposter {
            const NAME: &'static str = "Hp";
        }
        let mut w = world();
        let e = w.spawn();
        w.insert(e, Hp(7));
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| w.get::<Imposter>(e))).is_err());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            w.iter_mut::<Imposter>().count();
        }))
        .is_err());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| w.register::<Imposter>())).is_err());
        assert_eq!(w.get::<Hp>(e).unwrap().0, 7);
        let _ = Imposter(0).0;
    }

    #[test]
    fn disjoint_mutation_rejects_aliases_and_retention_keeps_sparse_slots_valid() {
        struct Damage(i32);
        impl Component for Damage {
            const NAME: &'static str = "Damage";
        }
        let mut world = world();
        world.register::<Damage>();
        for hp in [10, 20, 30] {
            let e = world.spawn();
            world.insert(e, Hp(hp));
            world.insert(e, Damage(3));
        }
        let first = world.iter_entities().next().unwrap();
        assert!(world.get2_mut::<Hp, Hp>(first).is_none());
        let (hp, damage) = world.get2_mut::<Hp, Damage>(first).unwrap();
        hp.0 -= damage.0;
        damage.0 = 2;
        assert_eq!(world.for_each2_mut::<Hp, Damage>(|_, hp, damage| hp.0 -= damage.0), 3);
        assert_eq!(world.get::<Hp>(first).unwrap().0, 5);
        assert_eq!(world.retain::<Hp>(|_, hp| hp.0 > 10), 1);
        assert_eq!(world.iter2::<Hp, Damage>().count(), 2);
        assert_eq!(world.len(), 3);
        world.despawn(first);
        assert!(world.get2_mut::<Hp, Damage>(first).is_none());
    }

    #[test]
    fn exhausted_generations_are_retired_without_reviving_stale_handles() {
        let mut world = world();
        let original = world.spawn();
        world.generations[original.index as usize] = u32::MAX;
        let last = Entity { index: original.index, gen: u32::MAX };
        world.despawn(last);
        assert!(world.is_empty());
        let next = world.spawn();
        assert_ne!(next.index(), original.index());
        assert!(!world.is_alive(original));
        assert!(!world.is_alive(last));
        assert_eq!(world.len(), 1);
    }

    #[test]
    fn checked_insert_preserves_rejected_and_replaced_values() {
        let mut world = World::new();
        let entity = world.spawn();
        assert_eq!(world.try_insert(entity, Hp(4)).err().unwrap().0, 4);
        world.register::<Hp>();
        assert!(world.try_insert(entity, Hp(5)).unwrap_or_else(|_| panic!("registered")).is_none());
        assert_eq!(world.try_insert(entity, Hp(8)).unwrap_or_else(|_| panic!("registered")).unwrap().0, 5);
        world.despawn(entity);
        assert_eq!(world.try_insert(entity, Hp(9)).err().unwrap().0, 9);
    }
}
