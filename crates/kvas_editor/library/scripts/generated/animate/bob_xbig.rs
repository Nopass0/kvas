use kvas_engine::prelude::*;

script_params! {
    /// Moves up and down along X like a buoy.
    pub struct BobXBig {
        amplitude: f32 = 1.0,
        speed: f32 = 1.0,
        origin: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for BobXBig {
    fn update(&mut self, ctx: &mut Ctx) {
        let origin = *self.origin.0.get_or_insert(ctx.transform().position);
        let wave = (ctx.elapsed() * self.speed).sin() * self.amplitude;
        ctx.transform_mut().position = origin + Vec3::X * wave;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
