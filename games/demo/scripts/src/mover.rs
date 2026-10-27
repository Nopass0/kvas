//! Mover script.

use kvas_engine::prelude::*;

/// Moves a (kinematic) body back and forth along an axis.
pub struct Mover {
    axis: Vec3,
    amplitude: f32,
    speed: f32,
    origin: Vec3,
}

impl Default for Mover {
    fn default() -> Self {
        Self { axis: Vec3::X, amplitude: 4.0, speed: 1.0, origin: Vec3::ZERO }
    }
}

impl Script for Mover {
    fn start(&mut self, ctx: &mut Ctx) {
        self.origin = ctx.transform().position;
    }
    fn update(&mut self, ctx: &mut Ctx) {
        let t = ctx.elapsed() * self.speed;
        ctx.transform_mut().position = self.origin + self.axis * (t.sin() * self.amplitude);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::vec3("axis", &mut self.axis), Field::f32("amplitude", &mut self.amplitude), Field::f32("speed", &mut self.speed)]
    }
}
