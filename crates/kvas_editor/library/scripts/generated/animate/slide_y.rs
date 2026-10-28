use kvas_engine::prelude::*;

script_params! {
    /// Slides back and forth along Y.
    pub struct SlideY {
        distance: f32 = 3.0,
        speed: f32 = 1.5,
        origin: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for SlideY {
    fn update(&mut self, ctx: &mut Ctx) {
        let origin = *self.origin.0.get_or_insert(ctx.transform().position);
        let k = 0.5 + 0.5 * (ctx.elapsed() * self.speed * 0.5).sin();
        ctx.transform_mut().position = origin + Vec3::Y * self.distance * k;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
