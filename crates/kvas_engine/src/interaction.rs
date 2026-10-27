//! Puzzle building blocks that need no code: `Switch` (button, lever, pressure plate) and `Slider`
//! (door, lift, gate). They talk through a shared `channel` name: a slider opens while any switch
//! on its channel is active. Scripts press buttons with `press()`.

use crate::choice_enum;
use crate::components::{Component, MeshRenderer, RigidBody, Transform};
use crate::hierarchy;
use crate::reflect::Field;
use glam::Vec3;
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};

choice_enum!(
    /// Momentary Button, persistent Toggle, or body-detecting pressure Plate.
    SwitchKind { Button, Toggle, Plate } default Button);

/// Button: active for `hold_time` seconds after being pressed. Toggle: flips on every press.
/// Plate: active while a physics object (a crate, the player) stands on it.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Switch {
    /// Momentary button, persistent toggle or proximity pressure plate.
    pub kind: SwitchKind,
    /// Shared channel name matched by sliders; empty emits no channel.
    pub channel: String,
    /// Duration in simulation seconds for a momentary button.
    pub hold_time: f32,
    /// Text shown on screen when the player looks at it (empty = not interactive by looking).
    pub prompt: String,
    /// RGBA MeshRenderer tint while active.
    pub color_on: [f32; 4],
    /// RGBA MeshRenderer tint while inactive.
    pub color_off: [f32; 4],
    #[serde(skip)]
    #[doc(hidden)]
    pub active: bool,
    #[serde(skip)]
    #[doc(hidden)]
    pub timer: f32,
}

impl Default for Switch {
    fn default() -> Self {
        Self {
            kind: SwitchKind::Button,
            channel: "door".into(),
            hold_time: 4.0,
            prompt: "Press E".into(),
            color_on: [0.2, 1.0, 0.35, 1.0],
            color_off: [1.0, 0.18, 0.12, 1.0],
            active: false,
            timer: 0.0,
        }
    }
}

impl Component for Switch {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::choice("kind", &mut self.kind),
            Field::text("channel", &mut self.channel),
            Field::f32("hold_time", &mut self.hold_time),
            Field::text("prompt", &mut self.prompt),
            Field::color("color_on", &mut self.color_on),
            Field::color("color_off", &mut self.color_off),
        ]
    }
}

/// Moves its entity between its start position and `start + open_offset` while its channel is active.
/// A door with collision needs a Kinematic RigidBody so physics follows it.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Slider {
    /// Channel opened while any matching switch is active.
    pub channel: String,
    /// Parent-local displacement from the initial position when fully open.
    pub open_offset: Vec3,
    /// Metres per second.
    pub speed: f32,
    #[serde(skip)]
    #[doc(hidden)]
    pub origin: Option<Vec3>,
    /// 0 = closed, 1 = fully open.
    #[serde(skip)]
    #[doc(hidden)]
    pub amount: f32,
}

impl Default for Slider {
    fn default() -> Self {
        Self { channel: "door".into(), open_offset: Vec3::new(0.0, 3.0, 0.0), speed: 2.0, origin: None, amount: 0.0 }
    }
}

impl Component for Slider {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::text("channel", &mut self.channel),
            Field::vec3("open_offset", &mut self.open_offset),
            Field::f32("speed", &mut self.speed),
        ]
    }
}

/// Presses a button / flips a toggle (what the player does with the use key).
pub fn press(world: &mut World, e: Entity) {
    if let Some(s) = world.get_mut::<Switch>(e) {
        match s.kind {
            SwitchKind::Button => s.timer = s.hold_time,
            SwitchKind::Toggle => s.active = !s.active,
            SwitchKind::Plate => {}
        }
    }
}

/// Is any physics object resting on the plate (its footprint, from just under to 1.5 m above its top)?
fn plate_pressed(world: &World, plate: Entity, bodies: &[(Entity, Vec3)]) -> bool {
    let m = hierarchy::global_matrix(world, plate);
    let inv = m.inverse();
    let top = m.w_axis.y + m.y_axis.length() * 0.5;
    bodies.iter().any(|&(e, p)| {
        let local = inv.transform_point3(p);
        e != plate && local.x.abs() <= 0.5 && local.z.abs() <= 0.5 && p.y > top - 0.2 && p.y < top + 1.5
    })
}

/// Advances every switch and slider by `dt` seconds.
/// Invalid or negative deltas are ignored.
pub fn update(world: &mut World, dt: f32) {
    if !dt.is_finite() || dt < 0.0 { return; }
    let switches: Vec<Entity> = world.iter::<Switch>().map(|(e, _)| e).collect();
    if switches.is_empty() && world.iter::<Slider>().next().is_none() {
        return;
    }
    let bodies: Vec<(Entity, Vec3)> = world
        .iter::<RigidBody>()
        .filter(|(_, rb)| rb.kind == crate::components::BodyKind::Dynamic)
        .map(|(e, _)| (e, hierarchy::global_matrix(world, e).w_axis.truncate()))
        .collect();
    let mut active_channels: Vec<String> = Vec::new();
    for e in switches {
        let pressed_plate = world.get::<Switch>(e).is_some_and(|s| s.kind == SwitchKind::Plate) && plate_pressed(world, e, &bodies);
        let Some(s) = world.get_mut::<Switch>(e) else { continue };
        match s.kind {
            SwitchKind::Button => {
                s.timer = (s.timer - dt).max(0.0);
                s.active = s.timer > 0.0;
            }
            SwitchKind::Plate => s.active = pressed_plate,
            SwitchKind::Toggle => {}
        }
        if s.active && !s.channel.is_empty() {
            active_channels.push(s.channel.clone());
        }
        let color = if s.active { s.color_on } else { s.color_off };
        if let Some(m) = world.get_mut::<MeshRenderer>(e) {
            m.color = color;
        }
    }

    let sliders: Vec<Entity> = world.iter::<Slider>().map(|(e, _)| e).collect();
    for e in sliders {
        let Some(base) = world.get::<Transform>(e).map(|t| t.position) else { continue };
        let Some(s) = world.get_mut::<Slider>(e) else { continue };
        let origin = *s.origin.get_or_insert(base);
        let target = if active_channels.contains(&s.channel) { 1.0 } else { 0.0 };
        let length = s.open_offset.length().max(0.001);
        let step = s.speed * dt / length;
        s.amount += (target - s.amount).clamp(-step, step);
        let eased = s.amount * s.amount * (3.0 - 2.0 * s.amount);
        let position = origin + s.open_offset * eased;
        if let Some(t) = world.get_mut::<Transform>(e) {
            t.position = position;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{register_all, Collider};

    fn world() -> World {
        let mut w = World::default();
        register_all(&mut w);
        w
    }

    fn spawn(w: &mut World, name: &str, position: Vec3, scale: Vec3) -> Entity {
        let e = hierarchy::spawn_named(w, name);
        w.insert(e, Transform { position, scale, ..Default::default() });
        e
    }

    #[test]
    fn button_opens_a_door_and_closes_it_again() {
        let mut w = world();
        let button = spawn(&mut w, "Button", Vec3::ZERO, Vec3::ONE);
        w.insert(button, Switch { hold_time: 1.0, ..Default::default() });
        let door = spawn(&mut w, "Door", Vec3::new(5.0, 1.0, 0.0), Vec3::ONE);
        w.insert(door, Slider { open_offset: Vec3::new(0.0, 3.0, 0.0), speed: 3.0, ..Default::default() });

        update(&mut w, 0.1);
        assert_eq!(w.get::<Transform>(door).unwrap().position.y, 1.0, "closed until pressed");
        // Held for 1 s, speed 3 m/s: open for about a second, then closing again.
        press(&mut w, button);
        let mut open_y = 0.0f32;
        for _ in 0..15 {
            update(&mut w, 0.1);
            open_y = open_y.max(w.get::<Transform>(door).unwrap().position.y);
        }
        assert!(open_y > 3.5, "door rose to {open_y}");
        for _ in 0..80 {
            update(&mut w, 0.1);
        }
        assert!((w.get::<Transform>(door).unwrap().position.y - 1.0).abs() < 0.01, "door closes after the hold time");
    }

    #[test]
    fn plate_is_pressed_by_a_crate_and_toggle_stays() {
        let mut w = world();
        let plate = spawn(&mut w, "Plate", Vec3::new(0.0, 0.05, 0.0), Vec3::new(2.0, 0.1, 2.0));
        w.insert(plate, Switch { kind: SwitchKind::Plate, channel: "gate".into(), ..Default::default() });
        let gate = spawn(&mut w, "Gate", Vec3::ZERO, Vec3::ONE);
        w.insert(gate, Slider { channel: "gate".into(), open_offset: Vec3::X * 2.0, speed: 100.0, ..Default::default() });
        let crate_ = spawn(&mut w, "Crate", Vec3::new(5.0, 0.6, 0.0), Vec3::ONE);
        w.insert(crate_, RigidBody::default());
        w.insert(crate_, Collider::default());

        update(&mut w, 0.1);
        assert!(!w.get::<Switch>(plate).unwrap().active);
        w.get_mut::<Transform>(crate_).unwrap().position = Vec3::new(0.4, 0.6, -0.3);
        update(&mut w, 0.1);
        assert!(w.get::<Switch>(plate).unwrap().active, "a crate on the plate presses it");
        update(&mut w, 0.1);
        assert!((w.get::<Transform>(gate).unwrap().position.x - 2.0).abs() < 0.01);
        w.get_mut::<Transform>(crate_).unwrap().position = Vec3::new(3.0, 0.6, 0.0);
        update(&mut w, 0.1);
        assert!(!w.get::<Switch>(plate).unwrap().active, "stepping off releases it");

        let lever = spawn(&mut w, "Lever", Vec3::new(9.0, 0.0, 0.0), Vec3::ONE);
        w.insert(lever, Switch { kind: SwitchKind::Toggle, channel: "gate".into(), ..Default::default() });
        press(&mut w, lever);
        update(&mut w, 0.1);
        update(&mut w, 5.0);
        assert!(w.get::<Switch>(lever).unwrap().active, "toggles do not time out");
        press(&mut w, lever);
        assert!(!w.get::<Switch>(lever).unwrap().active);
    }
}
