//! Inverse kinematics on top of a playing clip, for humanoid rigs ([`crate::humanoid`]):
//!
//! * **Feet on the ground.** Each foot is traced down onto the real colliders; the hips drop
//!   for the lower foot and each leg bends (two-bone IK, the knee stays in its plane) so the
//!   foot stands on stairs, slopes and kerbs instead of floating or sinking. The clip's own
//!   foot lift (a step) is kept. Needs physics, so it works in the running game.
//! * **Looking at something.** The chest, neck and head turn (20 / 30 / 50 %) towards a target,
//!   within a limit, easing in and out; a target behind the back is let go.
//!
//! Add [`CharacterIk`] next to an [`crate::skeletal::Animator`]. In a script:
//! `ctx.look_at_point(npc, Some(player_eyes))` or set `look_at` to an object's name.

use crate::skeletal::{SkinnedModel, Trs};
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// IK settings of an animated humanoid.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterIk {
    /// Plant the feet on the colliders under them.
    pub feet: bool,
    /// How far (metres) a foot may be raised or lowered, and the hips dropped.
    pub max_step: f32,
    /// Name of the object to look at (empty: none, unless a script gives a point).
    pub look_at: String,
    /// 0..1: how much of the turn the body follows.
    pub look_weight: f32,
    /// Largest turn away from the clip's own facing, in degrees.
    pub look_limit: f32,
    /// A world point to look at, set by scripts (wins over `look_at`; not saved).
    #[serde(skip)]
    pub look_point: Option<Vec3>,
}

impl Default for CharacterIk {
    fn default() -> Self {
        Self { feet: true, max_step: 0.45, look_at: String::new(), look_weight: 1.0, look_limit: 75.0, look_point: None }
    }
}

impl crate::components::Component for CharacterIk {
    fn fields(&mut self) -> Vec<crate::reflect::Field<'_>> {
        use crate::reflect::Field;
        vec![
            Field::bool("feet", &mut self.feet),
            Field::f32("max_step", &mut self.max_step),
            Field::entity("look_at", &mut self.look_at),
            Field::f32("look_weight", &mut self.look_weight),
            Field::f32("look_limit", &mut self.look_limit),
        ]
    }
}

/// What IK remembers between frames (smoothing).
#[derive(Clone, Default)]
pub struct IkState {
    look_dir: Option<Vec3>,
    look_amount: f32,
    hips_drop: f32,
}

/// Rotates a node and everything below it about `pivot`.
pub(crate) fn rotate_subtree(model: &SkinnedModel, pose: &mut [Trs], root: usize, pivot: Vec3, q: Quat) {
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        if let Some(p) = pose.get_mut(i) {
            p.t = pivot + q * (p.t - pivot);
            p.r = (q * p.r).normalize();
        }
        if let Some(c) = model.children.get(i) {
            stack.extend(c);
        }
    }
}

fn translate_subtree(model: &SkinnedModel, pose: &mut [Trs], root: usize, d: Vec3) {
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        if let Some(p) = pose.get_mut(i) {
            p.t += d;
        }
        if let Some(c) = model.children.get(i) {
            stack.extend(c);
        }
    }
}

/// Two-bone IK: bends `a`-`b`-`c` (thigh, shin, foot) so `c` reaches `target`, keeping the
/// knee in the plane it already bends in, and the foot's own orientation.
fn two_bone(model: &SkinnedModel, pose: &mut [Trs], (a, b, c): (usize, usize, usize), target: Vec3, bend_hint: Vec3) {
    let (pa, pb, pc) = (pose[a].t, pose[b].t, pose[c].t);
    let foot_rot = pose[c].r;
    let (l1, l2) = ((pb - pa).length(), (pc - pb).length());
    if l1 < 1e-4 || l2 < 1e-4 {
        return;
    }
    let to = target - pa;
    let d = to.length().clamp((l1 - l2).abs() + 1e-3, l1 + l2 - 1e-3);
    let n = to.normalize_or_zero();
    if n == Vec3::ZERO {
        return;
    }
    // Which way the knee points: as it does now, or the hint (forward) for a straight leg.
    let mut k = (pb - pa) - n * (pb - pa).dot(n);
    if k.length_squared() < 1e-6 {
        k = bend_hint - n * bend_hint.dot(n);
    }
    let k = k.normalize_or_zero();
    let cos_a = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
    let knee = pa + n * (l1 * cos_a) + k * (l1 * (1.0 - cos_a * cos_a).sqrt());
    let q1 = Quat::from_rotation_arc((pb - pa).normalize(), (knee - pa).normalize());
    rotate_subtree(model, pose, a, pa, q1);
    let (nb, nc) = (pose[b].t, pose[c].t);
    let q2 = Quat::from_rotation_arc((nc - nb).normalize_or_zero(), (pa + n * d - nb).normalize_or_zero());
    rotate_subtree(model, pose, b, nb, q2);
    // The foot keeps the clip's orientation.
    let fix = (foot_rot * pose[c].r.inverse()).normalize();
    let pc = pose[c].t;
    rotate_subtree(model, pose, c, pc, fix);
}

/// Applies IK to a model-space `pose`. `world` is the object's transform; `ground(from, len)`
/// traces straight down and returns the hit height (world space).
pub(crate) fn apply(
    model: &SkinnedModel,
    pose: &mut [Trs],
    world: Mat4,
    ik: &CharacterIk,
    look_target: Option<Vec3>,
    ground: Option<&dyn Fn(Vec3, f32) -> Option<f32>>,
    state: &mut IkState,
    dt: f32,
) {
    let Some(h) = &model.humanoid else { return };
    let inv = world.inverse();
    let ease = (dt * 8.0).clamp(0.0, 1.0);
    // ---- feet
    if let (true, Some(ground), Some(hips)) = (ik.feet, ground, h.hips) {
        let mut targets = [None, None];
        let mut drop = 0.0f32;
        for s in 0..2 {
            let (Some(_), Some(_), Some(c)) = (h.thigh[s], h.shin[s], h.foot[s]) else { continue };
            let rest_height = model.nodes[c].rest.w_axis.y;
            let foot = pose[c].t;
            let lift = (foot.y - rest_height).max(0.0);
            let w = world.transform_point3(foot.with_y(rest_height));
            let up = world.transform_vector3(h.up).normalize_or(Vec3::Y);
            let reach = ik.max_step * world.transform_vector3(Vec3::Y).length().max(1e-3);
            let Some(hit_y) = ground(w + up * reach, reach * 2.0) else { continue };
            // The ground under this foot, as a height in model space.
            let g = inv.transform_point3(Vec3::new(w.x, hit_y, w.z)).y.clamp(-ik.max_step, ik.max_step);
            drop = drop.min(g);
            targets[s] = Some(Vec3::new(foot.x, g + rest_height + lift, foot.z));
        }
        state.hips_drop += (drop - state.hips_drop) * ease;
        if state.hips_drop.abs() > 1e-4 {
            translate_subtree(model, pose, hips, h.up * state.hips_drop);
        }
        for s in 0..2 {
            if let (Some(t), Some(a), Some(b), Some(c)) = (targets[s], h.thigh[s], h.shin[s], h.foot[s]) {
                two_bone(model, pose, (a, b, c), t, h.forward);
            }
        }
    }
    // ---- look
    let Some(head) = h.head else { return };
    let head_pos = pose[head].t;
    let rest_rot = Trs::from_mat(model.nodes[head].rest).r;
    let facing = (pose[head].r * rest_rot.inverse() * h.forward).normalize_or(h.forward);
    let want = look_target.map(|t| (inv.transform_point3(t) - head_pos).normalize_or_zero()).filter(|d| *d != Vec3::ZERO);
    let wanted = want.filter(|d| d.angle_between(facing).to_degrees() < ik.look_limit + 25.0);
    let target_amount = if wanted.is_some() { ik.look_weight.clamp(0.0, 1.0) } else { 0.0 };
    state.look_amount += (target_amount - state.look_amount) * (dt * 4.0).clamp(0.0, 1.0);
    if let Some(d) = wanted {
        // Within the limit.
        let angle = d.angle_between(facing);
        let limit = ik.look_limit.to_radians();
        let d = if angle > limit { facing.slerp_to(d, limit / angle) } else { d };
        let prev = state.look_dir.unwrap_or(facing);
        state.look_dir = Some(prev.slerp_to(d, (dt * 6.0).clamp(0.0, 1.0)));
    }
    if state.look_amount < 1e-3 {
        return;
    }
    let dir = state.look_dir.unwrap_or(facing);
    let full = Quat::from_rotation_arc(facing, dir);
    let turn = Quat::IDENTITY.slerp(full, state.look_amount);
    let mut chain: Vec<(usize, f32)> = [(h.chest, 0.2), (h.neck, 0.3)].into_iter().filter_map(|(b, k)| Some((b?, k))).collect();
    let used: f32 = chain.iter().map(|c| c.1).sum();
    chain.push((head, 1.0 - used));
    for (bone, k) in chain {
        let pivot = pose[bone].t;
        rotate_subtree(model, pose, bone, pivot, Quat::IDENTITY.slerp(turn, k));
    }
}

trait SlerpTo {
    fn slerp_to(self, to: Vec3, k: f32) -> Vec3;
}

impl SlerpTo for Vec3 {
    fn slerp_to(self, to: Vec3, k: f32) -> Vec3 {
        Quat::IDENTITY.slerp(Quat::from_rotation_arc(self.normalize_or(Vec3::Z), to.normalize_or(Vec3::Z)), k) * self
    }
}
