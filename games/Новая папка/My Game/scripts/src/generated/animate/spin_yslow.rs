use kvas_engine::prelude::*;

script_params! {
    /// Rotates around the Y axis (20 degrees per second by default).
    pub struct SpinYSlow {
        degrees_per_second: f32 = 20.0,
    }
}

impl Script for SpinYSlow {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = Vec3::Y * self.degrees_per_second * ctx.dt();
        ctx.rotate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
