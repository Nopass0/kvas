use kvas_engine::prelude::*;

script_params! {
    /// Swings around X by +-25 degrees.
    pub struct WobbleXSway {
        degrees: f32 = 25.0,
        speed: f32 = 1.0,
        base: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for WobbleXSway {
    fn update(&mut self, ctx: &mut Ctx) {
        let base = *self.base.0.get_or_insert(ctx.transform().rotation);
        ctx.transform_mut().rotation = base + Vec3::X * (ctx.elapsed() * self.speed).sin() * self.degrees;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
