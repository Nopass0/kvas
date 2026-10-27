//! Rapier integration. ECS components are the source of truth: bodies, colliders and joints are
//! created from them (also for entities spawned later), and dynamic results are written back.
//!
//! Structures: a collider without its own RigidBody attaches to the nearest ancestor that has one,
//! so a hierarchy of colliders under one body moves as a single compound body.

use crate::components::*;
use crate::hierarchy;
use crate::scene::Scene;
use glam::{Mat4, Quat, Vec3};
use kvas_ecs::{Entity, World};
use rapier3d::prelude as r;
use std::collections::HashMap;

/// Closest solid-collider hit. Distances are metres along a normalized ray.
#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    /// Entity carrying the hit collider (may be a child of the body owner).
    pub entity: Entity,
    /// World-space hit position.
    pub point: Vec3,
    /// World-space surface normal reported by Rapier.
    pub normal: Vec3,
    /// Non-negative distance from the ray origin.
    pub distance: f32,
}

/// Filtering policy for world-space physics queries.
/// Layer masks use the same symmetric rule as collider/collider interactions.
#[derive(Clone, Copy, Debug)]
pub struct QueryOptions {
    /// Collider entity to skip; if it owns a body, all colliders of that body are skipped.
    pub exclude: Option<Entity>,
    /// Whether sensor volumes participate in the query. Defaults to false.
    pub include_triggers: bool,
    /// Query membership bits used against collider filter masks.
    pub memberships: u32,
    /// Collider membership bits accepted by this query.
    pub filter: u32,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self { exclude: None, include_triggers: false, memberships: u32::MAX, filter: u32::MAX }
    }
}

fn groups(memberships: u32, filter: u32) -> r::InteractionGroups {
    r::InteractionGroups::new(r::Group::from_bits_retain(memberships), r::Group::from_bits_retain(filter), r::InteractionTestMode::And)
}

/// Host-owned Rapier state mirrored from ECS components during [`Self::step`].
/// Queries reflect the most recently completed physics step, not pending ECS edits.
/// Use [`Self::reset`] to explicitly rebuild after wholesale scene replacement.
pub struct Physics {
    world: r::PhysicsWorld,
    bodies: HashMap<Entity, r::RigidBodyHandle>,
    colliders: HashMap<Entity, r::ColliderHandle>,
    joints: HashMap<Entity, r::ImpulseJointHandle>,
    owners: HashMap<r::ColliderHandle, Entity>,
    anchors: HashMap<Entity, r::RigidBodyHandle>,
    shapes: HashMap<Entity, (ShapeKind, Vec3, Vec3)>,
    joint_configs: HashMap<Entity, (u64, Option<r::RigidBodyHandle>)>,
}

fn pose_mat(m: Mat4) -> (Mat4, Vec3) {
    let (scale, rot, pos) = m.to_scale_rotation_translation();
    (Mat4::from_rotation_translation(rot, pos), scale)
}

fn pose(m: Mat4) -> r::Pose {
    let (_, rot, pos) = m.to_scale_rotation_translation();
    r::Pose::from_parts(pos, rot)
}

fn global(world: &World, e: Entity) -> Mat4 {
    world.get::<GlobalTransform>(e).map_or_else(|| hierarchy::global_matrix(world, e), |transform| transform.0)
}

fn make_shape(c: &Collider, scale: Vec3) -> r::ColliderBuilder {
    let s = scale.abs();
    let size = c.size.abs().max(Vec3::splat(0.001)) * s;
    let radius = size.x.max(size.z) * 0.5;
    match c.shape {
        ShapeKind::Box => r::ColliderBuilder::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
        ShapeKind::Sphere => r::ColliderBuilder::ball(size.x.max(size.y).max(size.z) * 0.5),
        ShapeKind::Capsule => r::ColliderBuilder::capsule_y((size.y * 0.5 - radius).max(0.0), radius),
        ShapeKind::Cylinder => r::ColliderBuilder::cylinder(size.y * 0.5, radius),
    }
}

impl Physics {
    /// Creates an empty simulation. ECS objects appear on the first valid step.
    pub fn new() -> Self {
        Self {
            world: r::PhysicsWorld::default(),
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            joints: HashMap::new(),
            owners: HashMap::new(),
            anchors: HashMap::new(),
            shapes: HashMap::new(),
            joint_configs: HashMap::new(),
        }
    }

    /// Closest solid hit, ignoring trigger volumes. `dir` need not be normalized.
    /// Invalid origins/directions, zero direction and invalid ranges return `None`.
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_distance: f32) -> Option<RayHit> {
        self.raycast_excluding(origin, dir, max_distance, None)
    }

    /// Like `raycast`, but ignores trigger volumes and the collider of `exclude` (the caster itself).
    pub fn raycast_excluding(&self, origin: Vec3, dir: Vec3, max_distance: f32, exclude: Option<Entity>) -> Option<RayHit> {
        self.raycast_with_options(origin, dir, max_distance, QueryOptions { exclude, ..Default::default() })
    }

    /// Closest ray hit under explicit sensor, layer and owner-exclusion filters.
    /// Direction is normalized internally; distance is in metres. Invalid input returns `None`.
    pub fn raycast_with_options(&self, origin: Vec3, dir: Vec3, max_distance: f32, options: QueryOptions) -> Option<RayHit> {
        if !origin.is_finite() || !dir.is_finite() || !max_distance.is_finite() || max_distance < 0.0 {
            return None;
        }
        let dir = crate::spatial::Ray::new(origin, dir)?.direction();
        let ray = r::Ray::new(origin, dir);
        let filter = self.query_filter(options);
        let (handle, hit) = self.world.cast_ray_and_get_normal(&ray, max_distance, true, filter)?;
        Some(RayHit {
            entity: *self.owners.get(&handle)?,
            point: origin + dir * hit.time_of_impact,
            normal: hit.normal,
            distance: hit.time_of_impact,
        })
    }

    fn query_filter(&self, options: QueryOptions) -> r::QueryFilter<'_> {
        let mut filter = r::QueryFilter::default().groups(groups(options.memberships, options.filter));
        if !options.include_triggers {
            filter = filter.exclude_sensors();
        }
        if let Some(handle) = options.exclude.and_then(|e| self.colliders.get(&e)) {
            filter = filter.exclude_collider(*handle);
        }
        if let Some(handle) = options.exclude.and_then(|e| self.bodies.get(&e)) {
            filter = filter.exclude_rigid_body(*handle);
        }
        filter
    }

    /// Clears all Rapier objects and queries; ECS data is left untouched.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Number of mirrored ECS rigid bodies (world joint anchors are excluded).
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
    /// Number of mirrored colliders.
    pub fn collider_count(&self) -> usize {
        self.colliders.len()
    }
    /// Number of mirrored impulse joints.
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// All collider entities intersecting a world-space sphere, including triggers.
    /// Returns a sorted unique list; invalid centres/radii return an empty list.
    pub fn overlap_sphere(&self, center: Vec3, radius: f32) -> Vec<Entity> {
        self.overlap_sphere_with_options(center, radius, QueryOptions { include_triggers: true, ..Default::default() })
    }

    /// Sphere-overlap query with explicit layer, trigger and owner filters.
    /// Results are unique collider entity handles in index order.
    pub fn overlap_sphere_with_options(&self, center: Vec3, radius: f32, options: QueryOptions) -> Vec<Entity> {
        if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
            return Vec::new();
        }
        let shape = r::Ball::new(radius);
        let pose = r::Pose::from_translation(center);
        let mut entities: Vec<_> = self
            .world
            .intersect_shape(pose, &shape, self.query_filter(options))
            .filter_map(|(handle, _)| self.owners.get(&handle).copied())
            .collect();
        entities.sort();
        entities.dedup();
        entities
    }

    /// Entities with active solid contact or sensor overlap against this collider.
    /// A compound body's children must be queried by their collider entity handles.
    /// Returns a sorted unique list from the last completed simulation step.
    pub fn touching(&self, entity: Entity) -> Vec<Entity> {
        let Some(&handle) = self.colliders.get(&entity) else {
            return Vec::new();
        };
        let contacts = self.world.contact_pairs_with(handle).filter(|p| p.has_any_active_contact()).map(|p| {
            if p.collider1 == handle {
                p.collider2
            } else {
                p.collider1
            }
        });
        let sensors = self.world.intersection_pairs_with(handle).filter(|p| p.4).map(|(a, _, b, _, _)| if a == handle { b } else { a });
        let mut entities: Vec<_> = contacts.chain(sensors).filter_map(|h| self.owners.get(&h).copied()).collect();
        entities.sort();
        entities.dedup();
        entities
    }

    /// Nearest entity at or above `e` that owns a rigid body.
    fn body_owner(&self, world: &World, mut e: Entity) -> Option<Entity> {
        let mut seen = std::collections::HashSet::new();
        loop {
            if !seen.insert(e) {
                return None;
            }
            if self.bodies.contains_key(&e) {
                return Some(e);
            }
            e = hierarchy::parent_of(world, e)?;
        }
    }

    fn remove_joint(&mut self, entity: Entity) {
        if let Some(handle) = self.joints.remove(&entity) {
            self.world.remove_impulse_joint(handle);
        }
        if let Some(anchor) = self.anchors.remove(&entity) {
            self.world.remove_body(anchor);
        }
        self.joint_configs.remove(&entity);
    }

    /// Creates physics objects for new entities and drops those of removed ones.
    fn reconcile(&mut self, world: &World) {
        let gone: Vec<_> = self
            .joints
            .iter()
            .filter(|(e, h)| !world.has::<Joint>(**e) || !world.has::<RigidBody>(**e) || self.world.impulse_joints.get(**h).is_none())
            .map(|(&e, _)| e)
            .collect();
        for e in gone {
            self.remove_joint(e);
        }
        let gone: Vec<_> = self.bodies.keys().copied().filter(|&e| !world.has::<RigidBody>(e)).collect();
        for e in gone {
            if let Some(h) = self.bodies.remove(&e) {
                self.world.remove_body(h);
            }
        }
        let gone: Vec<_> = self
            .colliders
            .iter()
            .filter(|(e, h)| !world.has::<Collider>(**e) || self.world.colliders.get(**h).is_none())
            .map(|(&e, _)| e)
            .collect();
        for e in gone {
            if let Some(h) = self.colliders.remove(&e) {
                self.owners.remove(&h);
                self.world.remove_collider(h);
            }
            self.shapes.remove(&e);
        }

        for (e, rb) in world.iter::<RigidBody>() {
            if self.bodies.contains_key(&e) {
                continue;
            }
            let b = match rb.kind {
                BodyKind::Dynamic => r::RigidBodyBuilder::dynamic(),
                BodyKind::Fixed => r::RigidBodyBuilder::fixed(),
                BodyKind::Kinematic => r::RigidBodyBuilder::kinematic_position_based(),
            }
            .pose(pose(global(world, e)))
            .gravity_scale(rb.gravity_scale)
            .linear_damping(rb.linear_damping)
            .angular_damping(rb.angular_damping)
            .ccd_enabled(rb.ccd)
            .additional_mass(rb.mass.max(0.001));
            let b = if rb.lock_rotation { b.lock_rotations() } else { b };
            self.bodies.insert(e, self.world.insert_body(b));
        }

        for (e, c) in world.iter::<Collider>() {
            if self.colliders.contains_key(&e) {
                let handle = self.colliders[&e];
                let owner = self.body_owner(world, e);
                let parent = owner.and_then(|p| self.bodies.get(&p).copied());
                let old_parent = self.world.colliders.get(handle).and_then(|c| c.parent());
                if old_parent != parent {
                    self.world.colliders.set_parent(handle, parent, &mut self.world.bodies);
                }
                let g = global(world, e);
                let (_, scale) = pose_mat(g);
                let shape_key = (c.shape, c.size, scale);
                let shape_changed = self.shapes.get(&e) != Some(&shape_key);
                if let Some(collider) = self.world.colliders.get_mut(handle) {
                    if shape_changed {
                        collider.set_shape(make_shape(c, scale).build().shared_shape().clone());
                    }
                    collider.set_friction(c.friction);
                    collider.set_restitution(c.restitution);
                    collider.set_sensor(c.is_trigger);
                    collider.set_collision_groups(groups(c.memberships, c.filter));
                    if let Some(owner) = owner {
                        let (owner_pose, _) = pose_mat(global(world, owner));
                        collider.set_position_wrt_parent(pose(owner_pose.inverse() * g));
                    } else {
                        collider.set_position(pose(g));
                    }
                }
                self.shapes.insert(e, shape_key);
                continue;
            }
            let g = global(world, e);
            let (_, scale) = pose_mat(g);
            let builder = make_shape(c, scale)
                .friction(c.friction)
                .restitution(c.restitution)
                .sensor(c.is_trigger)
                .collision_groups(groups(c.memberships, c.filter))
                .density(0.0);
            let handle = match self.body_owner(world, e) {
                Some(owner) => {
                    let (owner_pose, _) = pose_mat(global(world, owner));
                    let (_, rot, pos) = (owner_pose.inverse() * g).to_scale_rotation_translation();
                    let builder = builder.position(r::Pose::from_parts(pos, rot));
                    self.world.insert_collider(builder, Some(self.bodies[&owner]))
                }
                None => self.world.insert_collider(builder.position(pose(g)), None),
            };
            self.colliders.insert(e, handle);
            self.owners.insert(handle, e);
            self.shapes.insert(e, (c.shape, c.size, scale));
        }

        // Bodies can have disappeared above, and Inspector/scripts may edit
        // constraint settings or replace a named target while the entity lives.
        let changed: Vec<_> = self
            .joints
            .iter()
            .filter_map(|(&entity, &handle)| {
                let joint = world.get::<Joint>(entity)?;
                let target = if joint.target.is_empty() {
                    None
                } else {
                    hierarchy::find_by_name(world, &joint.target).and_then(|e| self.bodies.get(&e).copied())
                };
                let key = (crate::procedural::signature(joint), target);
                (self.world.impulse_joints.get(handle).is_none() || self.joint_configs.get(&entity) != Some(&key)).then_some(entity)
            })
            .collect();
        for entity in changed {
            self.remove_joint(entity);
        }

        for (e, j) in world.iter::<Joint>() {
            if self.joints.contains_key(&e) {
                continue;
            }
            let Some(&b1) = self.bodies.get(&e) else { continue };
            let (target_body, target_mat) = if j.target.is_empty() {
                (None, Mat4::IDENTITY)
            } else {
                match hierarchy::find_by_name(world, &j.target)
                    .and_then(|t| Some((self.bodies.get(&t).copied()?, pose_mat(global(world, t)).0)))
                {
                    Some((h, m)) => (Some(h), m),
                    None => continue, // target not spawned (yet)
                }
            };
            if target_body == Some(b1) {
                continue;
            } // self-joints are invalid constraints.
            let (m1, scale) = pose_mat(global(world, e));
            // Pivot joints attach at `anchor`; a spring hangs from its far end at `anchor`.
            let pivot = j.anchor * scale;
            let a1 = if j.kind == JointKind::Spring { Vec3::ZERO } else { pivot };
            let a2 = target_mat.inverse().transform_point3(m1.transform_point3(pivot));
            if let Some(joint) = build_joint(j, m1, target_mat, a1, a2) {
                let b2 = target_body.unwrap_or_else(|| {
                    let anchor = self.world.insert_body(r::RigidBodyBuilder::fixed());
                    self.anchors.insert(e, anchor);
                    anchor
                });
                self.joints.insert(e, self.world.insert_impulse_joint(b1, b2, joint));
                self.joint_configs.insert(e, (crate::procedural::signature(j), target_body));
            }
        }
    }

    /// Reconciles ECS objects, advances physics and writes back dynamic transforms.
    /// Invalid/non-positive deltas are ignored. Apply impulses before this call.
    /// Removing a physics component removes its Rapier counterpart on this step.
    pub fn step(&mut self, scene: &mut Scene, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        // Scripts and modifiers may have edited local poses since the last
        // frame. Refresh once before any reconciliation/query of cached poses.
        hierarchy::update_globals(&mut scene.world);
        self.reconcile(&scene.world);
        self.world.gravity = scene.settings.gravity;
        self.world.integration_parameters.dt = dt;

        for (&e, &h) in &self.bodies {
            let g = global(&scene.world, e);
            let Some(rb) = scene.world.get_mut::<RigidBody>(e) else { continue };
            let Some(body) = self.world.bodies.get_mut(h) else { continue };
            let kind = match rb.kind {
                BodyKind::Dynamic => r::RigidBodyType::Dynamic,
                BodyKind::Fixed => r::RigidBodyType::Fixed,
                BodyKind::Kinematic => r::RigidBodyType::KinematicPositionBased,
            };
            if body.body_type() != kind {
                body.set_body_type(kind, true);
            }
            body.set_gravity_scale(rb.gravity_scale, false);
            body.set_linear_damping(rb.linear_damping);
            body.set_angular_damping(rb.angular_damping);
            body.enable_ccd(rb.ccd);
            if (body.mass() - rb.mass.max(0.001)).abs() > f32::EPSILON {
                body.set_additional_mass(rb.mass.max(0.001), true);
            }
            body.lock_rotations(rb.lock_rotation, false);
            if std::mem::take(&mut rb.teleport) {
                body.set_position(pose(g), true);
            }
            match rb.kind {
                BodyKind::Kinematic => body.set_next_kinematic_position(pose(g)),
                BodyKind::Dynamic => {
                    if let Some(v) = rb.set_velocity.take() {
                        body.set_linvel(v, true);
                    }
                    if let Some(w) = rb.set_angular_velocity.take() {
                        body.set_angvel(w, true);
                    }
                    if rb.impulse != Vec3::ZERO {
                        body.apply_impulse(std::mem::take(&mut rb.impulse), true);
                    }
                }
                BodyKind::Fixed => body.set_position(pose(g), false),
            }
        }

        self.world.step();

        // Write results back, parents before children so local transforms see fresh ancestors.
        let mut dynamic: Vec<(Entity, r::RigidBodyHandle)> = self
            .bodies
            .iter()
            .filter(|(e, _)| scene.world.get::<RigidBody>(**e).is_some_and(|rb| rb.kind == BodyKind::Dynamic))
            .map(|(&e, &h)| (e, h))
            .collect();
        dynamic.sort_by_key(|&(e, _)| (hierarchy::ancestors_count(&scene.world, e), e));
        for (e, h) in dynamic {
            let Some(body) = self.world.bodies.get(h) else { continue };
            let p = body.position();
            let (_, scale) = pose_mat(global(&scene.world, e));
            let new_global = Mat4::from_scale_rotation_translation(scale, p.rotation, p.translation);
            let parent_m = hierarchy::parent_of(&scene.world, e).map_or(Mat4::IDENTITY, |pa| hierarchy::global_matrix(&scene.world, pa));
            if let Some(t) = scene.world.get_mut::<Transform>(e) {
                *t = Transform::from_matrix(parent_m.inverse() * new_global);
            }
            if let Some(rb) = scene.world.get_mut::<RigidBody>(e) {
                rb.velocity = body.linvel();
            }
        }
        hierarchy::update_globals(&mut scene.world);
    }
}

impl Default for Physics {
    fn default() -> Self {
        Self::new()
    }
}

fn build_joint(j: &Joint, m1: Mat4, m2: Mat4, a1: Vec3, a2: Vec3) -> Option<r::GenericJoint> {
    Some(match j.kind {
        JointKind::Fixed => {
            // Weld at the current relative pose.
            let frame = m1.inverse() * m2 * Mat4::from_translation(a2);
            let (_, rot, pos) = frame.to_scale_rotation_translation();
            r::FixedJointBuilder::new().local_frame1(r::Pose::from_parts(pos, rot)).local_anchor2(a2).build().into()
        }
        JointKind::Ball => r::SphericalJointBuilder::new().local_anchor1(a1).local_anchor2(a2).build().into(),
        JointKind::Hinge => {
            let axis = j.axis.try_normalize()?;
            let axis2 = Quat::from_mat4(&m2).inverse() * (Quat::from_mat4(&m1) * axis);
            let mut b = r::RevoluteJointBuilder::new(axis).local_anchor1(a1).local_anchor2(a2);
            if j.use_limits {
                b = b.limits([j.limit_min.to_radians(), j.limit_max.to_radians()]);
            }
            let mut joint: r::GenericJoint = b.build().into();
            joint.set_local_axis2(axis2);
            joint
        }
        JointKind::Spring => {
            r::SpringJointBuilder::new(j.rest_length, j.stiffness, j.damping).local_anchor1(a1).local_anchor2(a2).build().into()
        }
    })
}
