use kvas_engine::prelude::*;

script_params! {
    /// Rotates around the X axis (90 degrees per second by default).
    pub struct SpinX {
        degrees_per_second: f32 = 90.0,
    }
}

impl Script for SpinX {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = Vec3::X * self.degrees_per_second * ctx.dt();
        ctx.rotate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
