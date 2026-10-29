use kvas_engine::prelude::*;

script_params! {
    /// Slowly changes the field of view between two values.
    pub struct FovBreathWide {
        min_fov: f32 = 70.0,
        max_fov: f32 = 100.0,
        speed: f32 = 0.5,
    }
}

impl Script for FovBreathWide {
    fn update(&mut self, ctx: &mut Ctx) {
        let k = 0.5 + 0.5 * (ctx.elapsed() * self.speed).sin();
        let fov = self.min_fov + (self.max_fov - self.min_fov) * k;
        if let Some(c) = ctx.get_mut::<Camera>() {
            c.fov = fov;
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
