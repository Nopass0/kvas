use kvas_engine::prelude::*;

script_params! {
    /// Rotates around the Y axis (360 degrees per second by default).
    pub struct SpinYFast {
        degrees_per_second: f32 = 360.0,
    }
}

impl Script for SpinYFast {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = Vec3::Y * self.degrees_per_second * ctx.dt();
        ctx.rotate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
