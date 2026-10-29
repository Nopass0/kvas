use kvas_engine::prelude::*;

script_params! {
    /// Slides 2.0 m along Y when it receives the signal `open` and back on `close`.
    pub struct SlideDoorYShort {
        channel_open: String = "open".into(),
        channel_close: String = "close".into(),
        distance: f32 = 2.0,
        speed: f32 = 3.0,
        origin: Local<Option<Vec3>> = Local(None),
        open: Local<bool> = Local(false),
    }
}

impl Script for SlideDoorYShort {
    fn update(&mut self, ctx: &mut Ctx) {
        let origin = *self.origin.0.get_or_insert(ctx.transform().position);
        if !ctx.signals(&self.channel_open).is_empty() {
            *self.open = true;
        }
        if !ctx.signals(&self.channel_close).is_empty() {
            *self.open = false;
        }
        let goal = if *self.open { origin + Vec3::Y * self.distance } else { origin };
        let here = ctx.transform().position;
        ctx.transform_mut().position = here + (goal - here).clamp_length_max(self.speed * ctx.dt());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
