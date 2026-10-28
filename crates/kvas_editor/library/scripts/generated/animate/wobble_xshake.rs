use kvas_engine::prelude::*;

script_params! {
    /// Swings around X by +-4 degrees.
    pub struct WobbleXShake {
        degrees: f32 = 4.0,
        speed: f32 = 14.0,
        base: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for WobbleXShake {
    fn update(&mut self, ctx: &mut Ctx) {
        let base = *self.base.0.get_or_insert(ctx.transform().rotation);
        ctx.transform_mut().rotation = base + Vec3::X * (ctx.elapsed() * self.speed).sin() * self.degrees;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
