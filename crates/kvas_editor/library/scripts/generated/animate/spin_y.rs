use kvas_engine::prelude::*;

script_params! {
    /// Rotates around the Y axis (90 degrees per second by default).
    pub struct SpinY {
        degrees_per_second: f32 = 90.0,
    }
}

impl Script for SpinY {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = Vec3::Y * self.degrees_per_second * ctx.dt();
        ctx.rotate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
