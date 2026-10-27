use kvas_engine::prelude::*;

script_params! {
    /// Rotates the object continuously (degrees per second around each axis).
    pub struct Spinner {
        degrees_per_second: Vec3 = Vec3::new(0.0, 90.0, 0.0),
    }
}

impl Script for Spinner {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = self.degrees_per_second * ctx.dt();
        ctx.transform_mut().rotation += step;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
