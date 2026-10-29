use kvas_engine::prelude::*;

script_params! {
    /// Prints the frame rate every 5.0 s.
    pub struct LogFpsFive {
        interval: f32 = 5.0,
    }
}

impl Script for LogFpsFive {
    fn update(&mut self, ctx: &mut Ctx) {
        let fps = ctx.fps();
        ctx.watch_every(self.interval, "fps", fps);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
