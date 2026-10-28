use kvas_engine::prelude::*;

script_params! {
    /// Moves up and down along Z like a buoy.
    pub struct BobZSmall {
        amplitude: f32 = 0.1,
        speed: f32 = 2.0,
        origin: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for BobZSmall {
    fn update(&mut self, ctx: &mut Ctx) {
        let origin = *self.origin.0.get_or_insert(ctx.transform().position);
        let wave = (ctx.elapsed() * self.speed).sin() * self.amplitude;
        ctx.transform_mut().position = origin + Vec3::Z * wave;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
