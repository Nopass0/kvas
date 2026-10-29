use kvas_engine::prelude::*;

script_params! {
    /// Scales the object smoothly between a minimum and a maximum.
    pub struct PulseScaleBreathing {
        min_scale: f32 = 0.95,
        max_scale: f32 = 1.05,
        speed: f32 = 1.0,
        base: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for PulseScaleBreathing {
    fn update(&mut self, ctx: &mut Ctx) {
        let base = *self.base.0.get_or_insert(ctx.transform().scale);
        let k = 0.5 + 0.5 * (ctx.elapsed() * self.speed).sin();
        ctx.transform_mut().scale = base * (self.min_scale + (self.max_scale - self.min_scale) * k);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
