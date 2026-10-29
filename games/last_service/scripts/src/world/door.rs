//! A hinged door. Put it on the hinge entity: its children (the leaf model and its collider)
//! swing with it around the vertical axis.
//!
//! Public variables (Inspector, or `ctx.set_var(door, "Door", "locked", true.into())`):
//! `open`, `locked`, `angle` (how far it opens, degrees; negative swings the other way),
//! `speed` (degrees per second), `name_en` / `name_ru` (shown in the prompt).
//!
//! Signals it understands: `use` (toggle, the player's E), `open`, `close`.

use crate::s_shared::{message, sfx};
use kvas_engine::prelude::*;

pub struct Door {
    pub open: bool,
    pub locked: bool,
    pub angle: f32,
    pub speed: f32,
    pub name_en: String,
    pub name_ru: String,
    current: f32,
    base_yaw: Option<f32>,
}

impl Default for Door {
    fn default() -> Self {
        Self { open: false, locked: false, angle: 95.0, speed: 160.0, name_en: "door".into(), name_ru: "дверь".into(), current: 0.0, base_yaw: None }
    }
}

impl Door {
    fn set_open(&mut self, ctx: &mut Ctx, open: bool) {
        if self.open == open {
            return;
        }
        self.open = open;
        let at = ctx.world_position();
        ctx.play_sound_at(if open { "audio/sfx/sfx_door_open.wav" } else { "audio/sfx/sfx_door_close.wav" }, 0.7, at);
    }
}

impl Script for Door {
    fn start(&mut self, ctx: &mut Ctx) {
        self.base_yaw = Some(ctx.transform().rotation.y);
        // A door that starts open is drawn open right away.
        self.current = if self.open { self.angle } else { 0.0 };
    }

    fn update(&mut self, ctx: &mut Ctx) {
        for s in ctx.signals("use") {
            let _ = s;
            if self.locked {
                sfx(ctx, "sfx_latch", 0.35);
                message(ctx, "Locked.", "Заперто.");
            } else {
                let open = !self.open;
                self.set_open(ctx, open);
            }
        }
        if !ctx.signals("open").is_empty() && !self.locked {
            self.set_open(ctx, true);
        }
        if !ctx.signals("close").is_empty() {
            self.set_open(ctx, false);
        }
        let target = if self.open { self.angle } else { 0.0 };
        let step = self.speed * ctx.dt();
        let diff = target - self.current;
        if diff.abs() > 0.01 {
            // Eases in at the end, as a heavy door does.
            self.current += diff.clamp(-step, step) * (0.35 + 0.65 * (diff.abs() / self.angle.abs().max(1.0)).min(1.0));
            if (target - self.current).abs() < 0.2 {
                self.current = target;
            }
            let base = self.base_yaw.unwrap_or(0.0);
            ctx.transform_mut().rotation.y = base + self.current;
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::bool("open", &mut self.open),
            Field::bool("locked", &mut self.locked),
            Field::f32("angle", &mut self.angle),
            Field::f32("speed", &mut self.speed),
            Field::text("name_en", &mut self.name_en),
            Field::text("name_ru", &mut self.name_ru),
        ]
    }
}
