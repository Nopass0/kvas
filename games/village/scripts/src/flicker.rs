//! Flicker script.

use kvas_engine::prelude::*;

/// A lantern: dark by day, it fades on at dusk, flickers like a flame at night and fades out at dawn.
pub struct Flicker {
    base: f32,
    amount: f32,
}

impl Default for Flicker {
    fn default() -> Self {
        Self { base: 1.0, amount: 0.15 }
    }
}

/// 0 in daylight, 1 at night, with smooth dusk (17:30-19:30) and dawn (4:30-6:30) transitions.
fn darkness(hour: f32) -> f32 {
    let smooth = |a: f32, b: f32, x: f32| {
        let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    if hour >= 12.0 { smooth(17.5, 19.5, hour) } else { 1.0 - smooth(4.5, 6.5, hour) }
}

impl Script for Flicker {
    fn start(&mut self, ctx: &mut Ctx) {
        self.base = ctx.get::<Light>().map_or(1.0, |l| l.intensity);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let on = darkness(ctx.time_of_day());
        let t = ctx.elapsed() + ctx.entity.index() as f32 * 1.7;
        let noise = (t * 13.0).sin() * 0.5 + (t * 31.0).sin() * 0.3 + (t * 7.0).sin() * 0.2;
        let (base, amount) = (self.base, self.amount);
        if let Some(light) = ctx.get_mut::<Light>() {
            light.intensity = base * 1.4 * on * (1.0 + noise * amount);
        }
        if let Some(mesh) = ctx.get_mut::<MeshRenderer>() {
            mesh.emissive = 0.05 + 1.4 * on;
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("amount", &mut self.amount)]
    }
}
