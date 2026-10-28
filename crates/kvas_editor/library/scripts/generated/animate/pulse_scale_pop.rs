use kvas_engine::prelude::*;

script_params! {
    /// Scales the object smoothly between a minimum and a maximum.
    pub struct PulseScalePop {
        min_scale: f32 = 0.5,
        max_scale: f32 = 2.0,
        speed: f32 = 1.5,
        base: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for PulseScalePop {
    fn update(&mut self, ctx: &mut Ctx) {
        let base = *self.base.0.get_or_insert(ctx.transform().scale);
        let k = 0.5 + 0.5 * (ctx.elapsed() * self.speed).sin();
        ctx.transform_mut().scale = base * (self.min_scale + (self.max_scale - self.min_scale) * k);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
