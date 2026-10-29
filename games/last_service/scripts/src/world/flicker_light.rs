//! A lamp on the diner's tired wiring. `base` is its full intensity, `power` (0..1) is set by the
//! game director when a power line is off or dimmed, `flicker` makes it stutter now and then.

use kvas_engine::prelude::*;

pub struct FlickerLight {
    pub base: f32,
    pub power: f32,
    pub flicker: bool,
    shown: f32,
}

impl Default for FlickerLight {
    fn default() -> Self {
        Self { base: 2.0, power: 1.0, flicker: false, shown: -1.0 }
    }
}

impl Script for FlickerLight {
    fn update(&mut self, ctx: &mut Ctx) {
        let t = ctx.elapsed();
        let seed = ctx.entity.index() as f32;
        let k = if !self.flicker {
            1.0
        } else if (t * 3.1 + seed).sin() > 0.985 {
            0.15
        } else {
            0.92 + 0.08 * (t * 13.0 + seed).sin()
        };
        let want = self.base * self.power.clamp(0.0, 1.0) * k;
        // Only touch the light when it changes (steady lamps cost nothing).
        if (want - self.shown).abs() > 1e-3 {
            self.shown = want;
            if let Some(l) = ctx.get_mut::<Light>() {
                l.intensity = want;
            }
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("base", &mut self.base), Field::f32("power", &mut self.power), Field::bool("flicker", &mut self.flicker)]
    }
}
