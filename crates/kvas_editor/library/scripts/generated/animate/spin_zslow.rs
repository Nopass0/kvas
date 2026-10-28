use kvas_engine::prelude::*;

script_params! {
    /// Rotates around the Z axis (20 degrees per second by default).
    pub struct SpinZSlow {
        degrees_per_second: f32 = 20.0,
    }
}

impl Script for SpinZSlow {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = Vec3::Z * self.degrees_per_second * ctx.dt();
        ctx.rotate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
