//! Ragdolls for humanoid rigs: when [`Ragdoll::active`] turns on, the character's body is
//! rebuilt from physics capsules (pelvis, chest, head, upper arms, forearms, thighs, shins)
//! joined by ball joints, starting exactly in the pose the clip was in and moving with the
//! velocity it is given. The skeleton then follows the bodies every frame (hands and feet ride
//! on their forearms and shins); turning it off removes the bodies and the clip takes over again.
//!
//! The capsules collide with the world but not with each other ([`RAGDOLL_GROUP`]). They are
//! transient objects: not saved, not listed in the hierarchy.
//!
//! ```no_run
//! # use kvas_engine::prelude::*;
//! # fn f(ctx: &mut Ctx, monster: Entity, hit: Vec3) {
//! ctx.ragdoll(monster, hit * 40.0); // collapse, pushed the way it was hit
//! # }
//! ```

use crate::components::*;
use crate::skeletal::{SkinnedModel, Trs};
use glam::{Mat4, Quat, Vec3};
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};

/// Collision layer of ragdoll capsules: they hit everything else, not one another.
pub const RAGDOLL_GROUP: u32 = 1 << 29;

/// Physics-driven body for an animated humanoid.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ragdoll {
    /// The body is limp and simulated (on) or animated by the clip (off).
    pub active: bool,
    /// Total mass in kilograms, shared out over the body parts.
    pub mass: f32,
    /// A push given to the chest when the ragdoll starts (newton-seconds, world space).
    pub impulse: Vec3,
    /// Joint looseness: angular damping of the parts (higher = stiffer, slower fall).
    pub damping: f32,
}

impl Default for Ragdoll {
    fn default() -> Self {
        Self { active: false, mass: 65.0, impulse: Vec3::ZERO, damping: 1.5 }
    }
}

impl Component for Ragdoll {
    fn fields(&mut self) -> Vec<crate::reflect::Field<'_>> {
        use crate::reflect::Field;
        vec![Field::bool("active", &mut self.active), Field::f32("mass", &mut self.mass), Field::vec3("impulse", &mut self.impulse), Field::f32("damping", &mut self.damping)]
    }
}

/// One simulated part: the bone it drives and where the bone sits in the capsule.
#[derive(Clone)]
pub(crate) struct Part {
    node: usize,
    entity: Entity,
    offset: Mat4,
}

/// Bodies of an active ragdoll and the pose it started from.
#[derive(Clone, Default)]
pub(crate) struct RagdollState {
    pub(crate) parts: Vec<Part>,
    start: Vec<Trs>,
}

impl RagdollState {
    pub(crate) fn is_active(&self) -> bool {
        !self.parts.is_empty()
    }
}

fn body_name(owner: Entity, i: usize) -> String {
    format!("ragdoll#{}#{i}", owner.to_bits())
}

/// Builds the capsules in the current pose. `world` is the owner's transform.
pub(crate) fn activate(world_ecs: &mut World, owner: Entity, model: &SkinnedModel, pose: &[Trs], world: Mat4, rd: &Ragdoll, velocity: Vec3) -> RagdollState {
    let Some(h) = &model.humanoid else { return RagdollState::default() };
    let at = |n: Option<usize>| n.and_then(|i| pose.get(i)).map(|p| world.transform_point3(p.t));
    let up = world.transform_vector3(h.up).normalize_or(Vec3::Y);
    let (Some(hips), Some(head)) = (at(h.hips), at(h.head)) else { return RagdollState::default() };
    let height = ((head - hips).length() * 2.4).max(0.5);
    let chest = at(h.chest).or(at(h.spine));
    let chest_node = h.chest.or(h.spine);
    // (bone, from, to, radius / height, mass share, parent part)
    let mut spec: Vec<(usize, Vec3, Vec3, f32, f32, Option<usize>)> = Vec::new();
    let torso_top = chest.unwrap_or(head);
    spec.push((h.hips.unwrap_or(0), hips, torso_top, 0.075, 0.26, None));
    let mut upper = 0;
    if let (Some(c), Some(cn)) = (chest, chest_node) {
        spec.push((cn, c, at(h.neck).unwrap_or(head), 0.08, 0.24, Some(0)));
        upper = 1;
    }
    let head_part = spec.len();
    spec.push((h.head.unwrap_or(0), head, head + up * height * 0.13, 0.06, 0.08, Some(upper)));
    let _ = head_part;
    for s in 0..2 {
        if let (Some(ua), Some(fa)) = (at(h.upper_arm[s]), at(h.forearm[s])) {
            let p = spec.len();
            spec.push((h.upper_arm[s].unwrap_or(0), ua, fa, 0.03, 0.03, Some(upper)));
            let hand = at(h.hand[s]).unwrap_or(fa + (fa - ua));
            spec.push((h.forearm[s].unwrap_or(0), fa, hand, 0.025, 0.02, Some(p)));
        }
        if let (Some(th), Some(sh)) = (at(h.thigh[s]), at(h.shin[s])) {
            let p = spec.len();
            spec.push((h.thigh[s].unwrap_or(0), th, sh, 0.045, 0.1, Some(0)));
            let foot = at(h.foot[s]).unwrap_or(sh + (sh - th));
            spec.push((h.shin[s].unwrap_or(0), sh, foot, 0.035, 0.05, Some(p)));
        }
    }
    let mut parts = Vec::new();
    for (i, (node, from, to, r, share, parent)) in spec.iter().enumerate() {
        let seg = *to - *from;
        let len = seg.length().max(0.02);
        let radius = (r * height).clamp(0.015, len * 0.5);
        let center = (*from + *to) * 0.5;
        let rot = Quat::from_rotation_arc(Vec3::Y, seg / len);
        let e = world_ecs.spawn();
        world_ecs.insert(e, Name(body_name(owner, i)));
        let mut t = Transform { position: center, ..Default::default() };
        t.set_quat(rot);
        world_ecs.insert(e, t);
        world_ecs.insert(e, Transient);
        world_ecs.insert(
            e,
            RigidBody {
                mass: (rd.mass * share).max(0.2),
                linear_damping: 0.15,
                angular_damping: rd.damping,
                velocity,
                set_velocity: Some(velocity),
                impulse: if i == upper { rd.impulse } else { Vec3::ZERO },
                ..Default::default()
            },
        );
        world_ecs.insert(
            e,
            Collider { shape: ShapeKind::Capsule, size: Vec3::new(radius * 2.0, len.max(radius * 2.0 + 0.01), radius * 2.0), friction: 0.8, restitution: 0.0, memberships: RAGDOLL_GROUP, filter: !RAGDOLL_GROUP, ..Default::default() },
        );
        if let Some(p) = parent {
            world_ecs.insert(e, Joint { kind: JointKind::Ball, target: body_name(owner, *p), anchor: Vec3::new(0.0, -len * 0.5, 0.0), ..Default::default() });
        }
        let body = Mat4::from_rotation_translation(rot, center);
        let bone = world * pose[*node].mat();
        parts.push(Part { node: *node, entity: e, offset: body.inverse() * bone });
    }
    RagdollState { parts, start: pose.to_vec() }
}

/// The pose the bodies hold now (model space). Bones without a body keep their place
/// relative to their parent from the moment the ragdoll started.
pub(crate) fn pose(world_ecs: &World, model: &SkinnedModel, st: &RagdollState, world: Mat4, out: &mut Vec<Trs>) {
    let inv = world.inverse();
    let start: Vec<Mat4> = st.start.iter().map(Trs::mat).collect();
    let mut now: Vec<Option<Mat4>> = vec![None; model.nodes.len()];
    for p in &st.parts {
        if let Some(t) = world_ecs.get::<Transform>(p.entity) {
            now[p.node] = Some(inv * t.matrix() * p.offset);
        }
    }
    // Parents first.
    let depth = |mut i: usize| {
        let mut d = 0;
        while let Some(p) = model.nodes[i].parent {
            i = p;
            d += 1;
            if d > 128 {
                break;
            }
        }
        d
    };
    let mut order: Vec<usize> = (0..model.nodes.len()).collect();
    order.sort_by_key(|&i| depth(i));
    for &i in &order {
        if now[i].is_some() {
            continue;
        }
        now[i] = Some(match model.nodes[i].parent {
            Some(p) => now[p].unwrap_or(start[p]) * start[p].inverse() * start[i],
            None => start[i],
        });
    }
    out.clear();
    out.extend(now.into_iter().map(|m| Trs::from_mat(m.unwrap_or(Mat4::IDENTITY))));
}

/// Removes the bodies of a ragdoll.
pub(crate) fn deactivate(world_ecs: &mut World, st: &mut RagdollState) {
    for p in st.parts.drain(..) {
        world_ecs.despawn(p.entity);
    }
    st.start.clear();
}
