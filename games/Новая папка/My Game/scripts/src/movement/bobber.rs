use kvas_engine::prelude::*;

script_params! {
    /// Floats up and down like a pickup or a buoy.
    pub struct Bobber {
        height: f32 = 0.25,
        speed: f32 = 2.0,
        base_y: Local<Option<f32>> = Local(None),
    }
}

impl Script for Bobber {
    fn update(&mut self, ctx: &mut Ctx) {
        let base = *self.base_y.0.get_or_insert(ctx.transform().position.y);
        let y = base + (ctx.elapsed() * self.speed).sin() * self.height;
        ctx.transform_mut().position.y = y;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
