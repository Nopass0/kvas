use kvas_engine::prelude::*;

script_params! {
    /// Bobs up and down while slowly turning: pickups, crystals, floating signs.
    pub struct HoverLevitate {
        amplitude: f32 = 0.8,
        speed: f32 = 0.6,
        degrees_per_second: f32 = 10.0,
        origin_y: Local<Option<f32>> = Local(None),
    }
}

impl Script for HoverLevitate {
    fn update(&mut self, ctx: &mut Ctx) {
        let y0 = *self.origin_y.0.get_or_insert(ctx.transform().position.y);
        let t = ctx.elapsed();
        let dt = ctx.dt();
        let tr = ctx.transform_mut();
        tr.position.y = y0 + (t * self.speed).sin() * self.amplitude;
        tr.rotation.y += self.degrees_per_second * dt;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
