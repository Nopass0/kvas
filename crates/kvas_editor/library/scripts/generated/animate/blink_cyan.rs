use kvas_engine::prelude::*;

script_params! {
    /// Blinks between the original colour and a flash colour.
    pub struct BlinkCyan {
        flash_r: f32 = 0.1,
        flash_g: f32 = 0.9,
        flash_b: f32 = 1.0,
        hertz: f32 = 1.0,
        original: Local<Option<[f32; 4]>> = Local(None),
    }
}

impl Script for BlinkCyan {
    fn update(&mut self, ctx: &mut Ctx) {
        let me = ctx.entity;
        let base = *self.original.0.get_or_insert_with(|| ctx.get::<MeshRenderer>().map_or([1.0; 4], |m| m.color));
        let on = (ctx.elapsed() * self.hertz).fract() < 0.5;
        let c = if on { [self.flash_r, self.flash_g, self.flash_b, 1.0] } else { base };
        ctx.set_color(me, c);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
