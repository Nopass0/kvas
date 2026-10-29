//! Procedural clips for humanoid rigs that come without animations: `Idle`, `Walk`, `Run`,
//! `Talk`, `Wave`, `Sit`, `Wipe`, `Look`.
//!
//! Bones are recognised by name (`hips`/`pelvis`, `spine`, `chest`, `neck`, `head`,
//! `upper_arm`, `forearm`, `hand`, `thigh`, `shin`, `foot`, with sides written as `.L`, `_left`,
//! `Left...`). The character's own axes come from the rest pose (up, right from shoulder to
//! shoulder, forward), so a bend is "forward" whatever the bone axes of the file are. Arms in a
//! T- or A-pose are brought down first. Each clip is a function of time baked at 30 fps.

use crate::skeletal::{Clip, SkinnedModel, Trs};
use glam::{Mat4, Quat, Vec3};

const L: usize = 0;
const R: usize = 1;

/// Bone indices (into [`SkinnedModel::nodes`]) of the recognised parts of a humanoid; sides
/// are `[left, right]`. Used by procedural clips, IK and ragdolls.
#[derive(Default, Clone, Debug)]
pub struct HumanoidBones {
    /// Hips or pelvis: the root of the legs and the spine.
    pub hips: Option<usize>,
    /// Lower spine.
    pub spine: Option<usize>,
    /// Upper spine / chest.
    pub chest: Option<usize>,
    /// Neck.
    pub neck: Option<usize>,
    /// Head.
    pub head: Option<usize>,
    /// Upper arms.
    pub upper_arm: [Option<usize>; 2],
    /// Forearms.
    pub forearm: [Option<usize>; 2],
    /// Hands.
    pub hand: [Option<usize>; 2],
    /// Thighs.
    pub thigh: [Option<usize>; 2],
    /// Shins.
    pub shin: [Option<usize>; 2],
    /// Feet.
    pub foot: [Option<usize>; 2],
    /// The character's up axis in model space.
    pub up: Vec3,
    /// The character's right (from its left shoulder to its right one).
    pub right: Vec3,
    /// The direction it faces.
    pub forward: Vec3,
}

type Rig = HumanoidBones;

/// Finds the humanoid bones of a model by name (see the module docs); `None` when there are no
/// hips and two complete legs.
pub fn find_bones(model: &SkinnedModel) -> Option<HumanoidBones> {
    let mut rig = Rig::of(model)?;
    let f = frame(model, &rig);
    (rig.up, rig.right, rig.forward) = (f.up, f.right, f.forward);
    Some(rig)
}

fn norm(s: &str) -> String {
    let s = s.to_lowercase().replace("mixamorig", "");
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn find(names: &[String], bases: &[&str]) -> Option<usize> {
    bases.iter().find_map(|b| names.iter().position(|n| n == b))
}

fn find_side(names: &[String], bases: &[&str], side: usize) -> Option<usize> {
    let (short, long) = if side == L { ("l", "left") } else { ("r", "right") };
    bases.iter().find_map(|b| {
        names.iter().position(|n| {
            *n == format!("{b}{short}") || *n == format!("{b}{long}") || *n == format!("{long}{b}") || *n == format!("{short}{b}") || n.ends_with(&format!("{long}{b}"))
        })
    })
}

impl Rig {
    fn of(model: &SkinnedModel) -> Option<Rig> {
        let names: Vec<String> = model.nodes.iter().map(|n| norm(&n.name)).collect();
        let side = |bases: &[&str]| [find_side(&names, bases, L), find_side(&names, bases, R)];
        let rig = Rig {
            up: Vec3::Y,
            right: Vec3::NEG_X,
            forward: Vec3::Z,
            hips: find(&names, &["hips", "pelvis", "hip"]),
            spine: find(&names, &["spine", "spine01", "spine1"]),
            chest: find(&names, &["chest", "spine2", "spine02", "upperchest", "spine03"]),
            neck: find(&names, &["neck", "neck01"]),
            head: find(&names, &["head"]),
            upper_arm: side(&["upperarm", "arm"]),
            forearm: side(&["forearm", "lowerarm"]),
            hand: side(&["hand"]),
            thigh: side(&["thigh", "upperleg", "upleg"]),
            shin: side(&["shin", "calf", "lowerleg", "leg"]),
            foot: side(&["foot"]),
        };
        let legs = rig.thigh.iter().chain(&rig.shin).all(Option::is_some);
        (rig.hips.is_some() && legs).then_some(rig)
    }
}

/// The character's axes and the rest-pose fix for the arms.
struct Frame {
    up: Vec3,
    right: Vec3,
    forward: Vec3,
    arm_down: [Quat; 2],
}

fn pos(model: &SkinnedModel, i: Option<usize>) -> Option<Vec3> {
    Some(model.nodes.get(i?)?.rest.w_axis.truncate())
}

fn frame(model: &SkinnedModel, rig: &Rig) -> Frame {
    let up = Vec3::Y;
    let across = match (pos(model, rig.upper_arm[R]).zip(pos(model, rig.upper_arm[L])), pos(model, rig.thigh[R]).zip(pos(model, rig.thigh[L]))) {
        (Some((r, l)), _) => r - l,
        (None, Some((r, l))) => r - l,
        _ => Vec3::NEG_X,
    };
    let right = (across - up * across.dot(up)).normalize_or(Vec3::NEG_X);
    let forward = up.cross(right).normalize_or(Vec3::Z);
    // Arms hanging a little away from the body, whatever the rest pose is.
    let mut arm_down = [Quat::IDENTITY; 2];
    for s in [L, R] {
        if let (Some(a), Some(b)) = (pos(model, rig.upper_arm[s]), pos(model, rig.forearm[s])) {
            let dir = (b - a).normalize_or_zero();
            let out = if s == L { -right } else { right };
            let target = (-up * 12f32.to_radians().cos() + out * 12f32.to_radians().sin() + forward * 0.05).normalize();
            if dir != Vec3::ZERO {
                arm_down[s] = Quat::from_rotation_arc(dir, target);
            }
        }
    }
    Frame { up, right, forward, arm_down }
}

/// Bone rotations (model space) and a hips offset for one moment of a clip.
#[derive(Default)]
struct Pose {
    rot: Vec<(usize, Quat)>,
    hips_offset: Vec3,
}

impl Pose {
    fn set(&mut self, bone: Option<usize>, q: Quat) {
        if let Some(b) = bone {
            self.rot.push((b, q));
        }
    }
}

fn deg(axis: Vec3, d: f32) -> Quat {
    Quat::from_axis_angle(axis, d.to_radians())
}

const TAU: f32 = std::f32::consts::TAU;

/// One moment of a procedural clip.
fn pose(name: &str, t: f32, rig: &Rig, f: &Frame) -> Pose {
    let mut p = Pose::default();
    let (rt, up, fw) = (f.right, f.up, f.forward);
    // Swing about the right axis: positive moves a hanging limb forward, a torso backwards.
    let swing = |d: f32| deg(rt, d);
    let turn = |d: f32| deg(up, d);
    let roll = |d: f32| deg(fw, d);
    let breathe = (TAU * t / 4.0).sin();
    let arms_idle = |p: &mut Pose, sway: f32| {
        for s in [L, R] {
            p.set(rig.upper_arm[s], swing(2.0 * sway) * f.arm_down[s]);
            p.set(rig.forearm[s], swing(10.0));
        }
    };
    match name {
        "Idle" => {
            p.set(rig.chest, swing(-1.5 * breathe));
            p.set(rig.spine, roll(1.2 * (TAU * t / 8.0).sin()));
            p.set(rig.head, turn(8.0 * (TAU * t / 8.0).sin()) * swing(-3.0 + 2.0 * (TAU * t / 5.0).sin()));
            arms_idle(&mut p, breathe);
            p.hips_offset = up * (-0.004 * breathe);
        }
        "Walk" | "Run" => {
            let run = name == "Run";
            let (period, k) = if run { (0.7, 1.6) } else { (1.1, 1.0) };
            let ph = TAU * t / period;
            let s = ph.sin();
            let c = ph.cos();
            p.set(rig.thigh[L], swing(24.0 * k * s));
            p.set(rig.thigh[R], swing(-24.0 * k * s));
            p.set(rig.shin[L], swing(-(8.0 + 32.0 * k * c.max(0.0))));
            p.set(rig.shin[R], swing(-(8.0 + 32.0 * k * (-c).max(0.0))));
            p.set(rig.foot[L], swing(8.0 * s));
            p.set(rig.foot[R], swing(-8.0 * s));
            p.set(rig.upper_arm[L], swing(-18.0 * k * s) * f.arm_down[L]);
            p.set(rig.upper_arm[R], swing(18.0 * k * s) * f.arm_down[R]);
            p.set(rig.forearm[L], swing(15.0 * k));
            p.set(rig.forearm[R], swing(15.0 * k));
            p.set(rig.hips, turn(5.0 * s));
            p.set(rig.spine, swing(if run { -8.0 } else { -2.0 }));
            p.set(rig.chest, turn(-7.0 * s));
            p.set(rig.head, turn(2.0 * s));
            p.hips_offset = up * (0.022 * k * (2.0 * ph).cos());
        }
        "Talk" => {
            let g = (TAU * t * 0.9).sin();
            p.set(rig.chest, swing(-1.5 * breathe));
            p.set(rig.head, swing(-4.0 * (TAU * t * 1.3).sin()) * turn(6.0 * (TAU * t / 3.0).sin()));
            p.set(rig.neck, swing(-2.0 * (TAU * t * 1.3).sin()));
            p.set(rig.upper_arm[L], f.arm_down[L]);
            p.set(rig.forearm[L], swing(12.0));
            p.set(rig.upper_arm[R], swing(18.0 + 6.0 * g) * f.arm_down[R]);
            p.set(rig.forearm[R], swing(40.0 + 18.0 * g) * turn(-10.0 * g));
            p.set(rig.hand[R], roll(15.0 * g));
        }
        "Wave" => {
            let w = (TAU * t * 1.5).sin();
            p.set(rig.upper_arm[L], f.arm_down[L]);
            p.set(rig.forearm[L], swing(10.0));
            // Right arm up to the side, the forearm waving.
            p.set(rig.upper_arm[R], roll(-150.0) * f.arm_down[R]);
            p.set(rig.forearm[R], roll(25.0 * w));
            p.set(rig.head, turn(-6.0));
            p.set(rig.chest, swing(-1.0 * breathe));
        }
        "Sit" => {
            for s in [L, R] {
                p.set(rig.thigh[s], swing(88.0));
                p.set(rig.shin[s], swing(-88.0));
                p.set(rig.upper_arm[s], swing(32.0) * f.arm_down[s]);
                p.set(rig.forearm[s], swing(48.0));
            }
            p.set(rig.spine, swing(-4.0));
            p.set(rig.chest, swing(-1.5 * breathe));
            p.set(rig.head, swing(-6.0) * turn(5.0 * (TAU * t / 6.0).sin()));
        }
        "Wipe" => {
            let w = TAU * t / 1.2;
            p.set(rig.spine, swing(-14.0));
            p.set(rig.chest, swing(-6.0) * turn(4.0 * w.sin()));
            p.set(rig.head, swing(-16.0));
            p.set(rig.upper_arm[R], turn(16.0 * w.sin()) * swing(50.0 + 8.0 * w.cos()) * f.arm_down[R]);
            p.set(rig.forearm[R], swing(55.0 + 10.0 * w.sin()));
            p.set(rig.upper_arm[L], swing(35.0) * f.arm_down[L]);
            p.set(rig.forearm[L], swing(60.0));
        }
        "Look" => {
            let a = (TAU * t / 6.0).sin();
            p.set(rig.chest, turn(10.0 * a) * swing(-1.5 * breathe));
            p.set(rig.head, turn(35.0 * a) * swing(-2.0));
            arms_idle(&mut p, breathe);
        }
        _ => {}
    }
    p
}

/// Forward kinematics: the rest pose with model-space rotations applied at each bone.
fn evaluate(model: &SkinnedModel, order: &[usize], rest_local: &[Mat4], rig: &Rig, p: &Pose) -> Vec<Trs> {
    let n = model.nodes.len();
    let mut delta = vec![Quat::IDENTITY; n];
    for (b, q) in &p.rot {
        delta[*b] = *q * delta[*b];
    }
    let mut world = vec![Mat4::IDENTITY; n];
    for &i in order {
        let rest_rot = Trs::from_mat(model.nodes[i].rest).r;
        let local = Mat4::from_quat(rest_rot.inverse() * delta[i] * rest_rot);
        let parent = model.nodes[i].parent.map_or(Mat4::IDENTITY, |p| world[p]);
        world[i] = parent * rest_local[i] * local;
        if Some(i) == rig.hips {
            world[i] = Mat4::from_translation(p.hips_offset) * world[i];
        }
    }
    world.into_iter().map(Trs::from_mat).collect()
}

/// Adds the procedural clips a humanoid rig lacks.
pub fn add_clips(model: &mut SkinnedModel) {
    let Some(rig) = Rig::of(model) else { return };
    let f = frame(model, &rig);
    // Parents before children.
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
    let rest_local: Vec<Mat4> = model.nodes.iter().map(|n| n.parent.map_or(n.rest, |p| model.nodes[p].rest.inverse() * n.rest)).collect();
    let fps = 30.0;
    for (name, duration) in [("Idle", 8.0), ("Walk", 1.1), ("Run", 0.7), ("Talk", 6.0), ("Wave", 2.0), ("Sit", 6.0), ("Wipe", 2.4), ("Look", 6.0)] {
        if model.clip(name).is_some() {
            continue;
        }
        let count = (duration * fps) as usize + 1;
        let frames = (0..count).map(|k| evaluate(model, &order, &rest_local, &rig, &pose(name, k as f32 / fps, &rig, &f))).collect();
        model.clips.push(Clip { name: name.to_string(), duration, fps, frames });
    }
}
