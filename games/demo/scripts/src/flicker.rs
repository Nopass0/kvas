//! Flicker script.

use kvas_engine::prelude::*;

/// Makes a point light flicker like a flame.
#[derive(Default)]
pub struct Flicker {
    base: f32,
    amount: f32,
}

impl Script for Flicker {
    fn start(&mut self, ctx: &mut Ctx) {
        self.base = ctx.get::<Light>().map_or(1.0, |l| l.intensity);
        if self.amount == 0.0 {
            self.amount = 0.3;
        }
    }
    fn update(&mut self, ctx: &mut Ctx) {
        let t = ctx.elapsed();
        let noise = (t * 13.0).sin() * 0.5 + (t * 31.0).sin() * 0.3 + (t * 7.0).sin() * 0.2;
        let (base, amount) = (self.base, self.amount);
        if let Some(light) = ctx.get_mut::<Light>() {
            light.intensity = base * (1.0 + noise * amount);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("amount", &mut self.amount)]
    }
}
