use kvas_engine::prelude::*;

script_params! {
    /// Rotates around the X axis (20 degrees per second by default).
    pub struct SpinXSlow {
        degrees_per_second: f32 = 20.0,
    }
}

impl Script for SpinXSlow {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = Vec3::X * self.degrees_per_second * ctx.dt();
        ctx.rotate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
