//! Spinner script.

use kvas_engine::prelude::*;

/// Rotates the entity at a constant speed (degrees per second per axis).
pub struct Spinner {
    speed: Vec3,
}

impl Default for Spinner {
    fn default() -> Self {
        Self { speed: Vec3::new(0.0, 45.0, 0.0) }
    }
}

impl Script for Spinner {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        ctx.transform_mut().rotation += self.speed * dt;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::vec3("speed", &mut self.speed)]
    }
}
