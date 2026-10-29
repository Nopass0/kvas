//! The flashlight in Lida's hand (a spot light under the camera). F turns it on once she has
//! picked it up; the old battery makes it waver. Publishes `ls.flash_on` (the Counter sees a lit
//! player from further away).

use crate::s_shared::*;
use kvas_engine::prelude::*;

pub struct Flashlight {
    pub intensity: f32,
    pub on: bool,
    had: bool,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self { intensity: 5.0, on: false, had: false }
    }
}

impl Script for Flashlight {
    fn update(&mut self, ctx: &mut Ctx) {
        let s = story(ctx);
        // Switched on the moment it is picked up.
        if s.flashlight && !self.had {
            self.on = true;
        }
        self.had = s.flashlight;
        if !s.flashlight {
            self.on = false;
        } else if mode(ctx) == Mode::Playing && ctx.action_pressed("Flashlight") {
            self.on = !self.on;
            sfx(ctx, "sfx_ui", 0.35);
        }
        let t = ctx.elapsed();
        let waver = if (t * 0.7).sin() > 0.995 { 0.4 } else { 0.96 + 0.04 * (t * 9.0).sin() };
        let on = self.on && !matches!(mode(ctx), Mode::Menu | Mode::Credits);
        let intensity = if on { self.intensity * waver } else { 0.0 };
        if let Some(l) = ctx.get_mut::<Light>() {
            l.intensity = intensity;
        }
        if ctx.global_bool(FLASH_ON, false) != on {
            ctx.set_global(FLASH_ON, on.into());
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("intensity", &mut self.intensity), Field::bool("on", &mut self.on)]
    }
}
