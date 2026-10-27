//! Parent/child relations and world-space transforms.
//!
//! [`set_parent`] preserves the world pose; [`set_parent_local`] preserves the
//! local transform. Both reject dead handles, cycles and invalid parent chains.
//! Matrices use column vectors, +Y up and local -Z forward. Transform rotations
//! are stored as Euler degrees. Directly inserting [`Parent`] bypasses validation;
//! traversal still terminates on malformed cycles.

use crate::components::*;
use glam::Mat4;
use kvas_ecs::{Entity, World};
use std::collections::{HashMap, HashSet};

/// Spawns an entity with [`Name`] and identity [`Transform`].
/// Built-in component storages must already be registered by the host.
pub fn spawn_named(world: &mut World, name: &str) -> Entity {
    let e = world.spawn();
    world.insert(e, Name(name.to_string()));
    world.insert(e, Transform::default());
    e
}

/// Entity name, or an empty string for stale or unnamed entities.
pub fn name_of(world: &World, e: Entity) -> &str {
    world.get::<Name>(e).map_or("", |n| n.0.as_str())
}

/// First exact name match in component storage order. Names need not be unique.
pub fn find_by_name(world: &World, name: &str) -> Option<Entity> {
    world.iter::<Name>().find(|(_, n)| n.0 == name).map(|(e, _)| e)
}

/// Living parent, or `None` for roots and dangling links.
pub fn parent_of(world: &World, e: Entity) -> Option<Entity> {
    world.get::<Parent>(e).map(|p| p.0).filter(|&p| world.is_alive(p))
}

/// Children in index order, including transient ones created by systems.
pub fn children_all(world: &World, e: Entity) -> Vec<Entity> {
    if !world.is_alive(e) {
        return Vec::new();
    }
    let mut out: Vec<Entity> = world.iter::<Parent>().filter(|(_, p)| p.0 == e).map(|(c, _)| c).collect();
    out.sort();
    out
}

/// Children in index order, without transient system-made entities.
pub fn children(world: &World, e: Entity) -> Vec<Entity> {
    children_all(world, e).into_iter().filter(|&c| !world.has::<Transient>(c)).collect()
}

/// Non-transient entities without a living parent, in index order.
pub fn roots(world: &World) -> Vec<Entity> {
    world.iter_entities().filter(|&e| parent_of(world, e).is_none() && !world.has::<Transient>(e)).collect()
}

/// Whether `ancestor` occurs strictly above `of`. Traversal is cycle-safe.
pub fn is_ancestor(world: &World, ancestor: Entity, of: Entity) -> bool {
    if ancestor == of || !world.is_alive(ancestor) {
        return false;
    }
    let mut seen = HashSet::from([of]);
    let mut cur = parent_of(world, of);
    loop {
        match cur {
            Some(p) if p == ancestor => return true,
            Some(p) if seen.insert(p) => cur = parent_of(world, p),
            Some(_) => return false,
            None => return false,
        }
    }
}

/// Number of distinct living ancestors, excluding the entity itself.
pub fn ancestors_count(world: &World, e: Entity) -> usize {
    let mut n = 0;
    let mut seen = HashSet::from([e]);
    let mut cur = parent_of(world, e);
    while let Some(p) = cur {
        if !seen.insert(p) {
            break;
        }
        n += 1;
        cur = parent_of(world, p);
    }
    n
}

/// All descendants, including transient entities, each returned once.
/// Children are visited deterministically; the root is excluded even on cycles.
/// The parent map is built once, avoiding a world scan per visited node.
pub fn descendants(world: &World, e: Entity) -> Vec<Entity> {
    if !world.is_alive(e) {
        return Vec::new();
    }
    let mut links: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (child, p) in world.iter::<Parent>() {
        links.entry(p.0).or_default().push(child);
    }
    for children in links.values_mut() {
        children.sort();
    }
    let mut out = Vec::new();
    let mut seen = HashSet::from([e]);
    let mut stack = vec![e];
    while let Some(cur) = stack.pop() {
        for &c in links.get(&cur).into_iter().flatten() {
            if seen.insert(c) {
                out.push(c);
                stack.push(c);
            }
        }
    }
    out
}

/// Destroys a subtree, including transient system entities. Stale roots are ignored.
pub fn despawn_recursive(world: &mut World, e: Entity) {
    for d in descendants(world, e) {
        world.despawn(d);
    }
    world.despawn(e);
}

/// Computes the current world matrix directly from local transforms.
/// Missing transforms contribute identity; stale handles return identity.
/// Malformed cycles are truncated before revisiting an entity.
pub fn global_matrix(world: &World, e: Entity) -> Mat4 {
    let mut m = Mat4::IDENTITY;
    let mut seen = HashSet::new();
    let mut cur = Some(e);
    while let Some(c) = cur {
        if !world.is_alive(c) || !seen.insert(c) {
            break;
        }
        m = world.get::<Transform>(c).map_or(Mat4::IDENTITY, Transform::matrix) * m;
        cur = parent_of(world, c);
    }
    m
}

fn valid_parent(world: &World, child: Entity, parent: Option<Entity>) -> bool {
    if !world.is_alive(child) {
        return false;
    }
    let mut seen = HashSet::from([child]);
    let mut cur = parent;
    while let Some(e) = cur {
        if !world.is_alive(e) || !seen.insert(e) {
            return false;
        }
        cur = world.get::<Parent>(e).map(|p| p.0);
    }
    true
}

/// Reparents while keeping the world pose when it is representable as TRS.
/// Returns `false` without mutation for dead handles, cycles, non-finite matrices
/// or a singular parent. Shear introduced by a rotated non-uniformly scaled parent
/// is rejected because [`Transform`] cannot represent it exactly.
pub fn set_parent(world: &mut World, child: Entity, parent: Option<Entity>) -> bool {
    if !valid_parent(world, child, parent) {
        return false;
    }
    let world_m = global_matrix(world, child);
    let parent_m = parent.map_or(Mat4::IDENTITY, |p| global_matrix(world, p));
    if !world_m.is_finite() || !parent_m.is_finite() || parent_m.determinant() == 0.0 {
        return false;
    }
    let local = parent_m.inverse() * world_m;
    // Zero-scale local transforms cannot be decomposed into a finite rotation.
    if !local.is_finite() || local.determinant() == 0.0 {
        return false;
    }
    let Some(transform) = Transform::try_from_matrix(local) else {
        return false;
    };
    set_parent_local(world, child, parent);
    if let Some(t) = world.get_mut::<Transform>(child) {
        *t = transform;
    }
    true
}

/// Reparents without changing the local transform. Accepts zero-scale parents.
/// Returns `false` without mutation for stale handles or invalid/cyclic chains.
pub fn set_parent_local(world: &mut World, child: Entity, parent: Option<Entity>) -> bool {
    if !valid_parent(world, child, parent) {
        return false;
    }
    match parent {
        Some(p) => world.insert(child, Parent(p)),
        None => {
            world.remove::<Parent>(child);
        }
    }
    true
}

/// Returns every exact name match in entity index order.
pub fn find_all_by_name(world: &World, name: &str) -> Vec<Entity> {
    let mut matches: Vec<_> = world.iter::<Name>().filter(|(_, n)| n.0 == name).map(|(e, _)| e).collect();
    matches.sort();
    matches
}

/// Finds a named entity inside a subtree, optionally including the root.
pub fn find_descendant(world: &World, root: Entity, name: &str, include_root: bool) -> Option<Entity> {
    if include_root && world.is_alive(root) && name_of(world, root) == name {
        return Some(root);
    }
    descendants(world, root).into_iter().find(|&e| name_of(world, e) == name)
}

/// Effective mesh visibility inherited through the living parent chain.
/// Missing visibility components default to visible; stale entities return false.
/// Cycles are visited once, so directly inserted malformed links cannot hang rendering.
pub fn is_visible(world: &World, entity: Entity) -> bool {
    if !world.is_alive(entity) {
        return false;
    }
    let mut current = Some(entity);
    let mut seen = HashSet::new();
    while let Some(e) = current {
        if !seen.insert(e) {
            break;
        }
        if world.get::<Visibility>(e).is_some_and(|v| !v.visible) {
            return false;
        }
        current = parent_of(world, e);
    }
    true
}

/// Changes only local translation to place an entity at a world-space point.
/// Rejects stale/missing transforms and singular/non-finite parent matrices.
/// Dynamic bodies also need their `RigidBody.teleport` request set before the
/// next physics step; [`crate::script::Ctx::teleport_of`] handles this for scripts.
pub fn set_world_position(world: &mut World, entity: Entity, position: glam::Vec3) -> bool {
    if !position.is_finite() || !world.has::<Transform>(entity) {
        return false;
    }
    let parent = parent_of(world, entity).map_or(Mat4::IDENTITY, |parent| global_matrix(world, parent));
    if !parent.is_finite() || parent.determinant() == 0.0 {
        return false;
    }
    let local = parent.inverse().transform_point3(position);
    if !local.is_finite() {
        return false;
    }
    world.get_mut::<Transform>(entity).unwrap().position = local;
    true
}

/// Refreshes [`GlobalTransform`] for every entity that has a [`Transform`].
/// Healthy parent chains are resolved once and cached for this call, making a
/// complete refresh linear in the entity count even for a long chain. Directly
/// inserted cyclic links fall back to bounded per-entity evaluation.
pub fn update_globals(world: &mut World) {
    let entities = world.entities();
    let mut resolved: HashMap<Entity, Mat4> = HashMap::with_capacity(entities.len());
    let mut path = Vec::new();
    let mut seen = HashSet::new();
    for &entity in &entities {
        if resolved.contains_key(&entity) {
            continue;
        }
        path.clear();
        seen.clear();
        let mut current = Some(entity);
        let mut base = Mat4::IDENTITY;
        let mut cycle = false;
        while let Some(e) = current {
            if let Some(&matrix) = resolved.get(&e) {
                base = matrix;
                break;
            }
            if !seen.insert(e) {
                cycle = true;
                break;
            }
            path.push(e);
            current = parent_of(world, e);
        }
        if cycle {
            for &e in &path {
                resolved.insert(e, global_matrix(world, e));
            }
        } else {
            for &e in path.iter().rev() {
                base *= world.get::<Transform>(e).map_or(Mat4::IDENTITY, Transform::matrix);
                resolved.insert(e, base);
            }
        }
    }
    for entity in entities {
        if world.has::<Transform>(entity) {
            world.insert(entity, GlobalTransform(resolved[&entity]));
        }
    }
}
