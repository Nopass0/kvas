use kvas_engine::prelude::*;

script_params! {
    /// Prints the frame rate every 1.0 s.
    pub struct LogFpsSecond {
        interval: f32 = 1.0,
    }
}

impl Script for LogFpsSecond {
    fn update(&mut self, ctx: &mut Ctx) {
        let fps = ctx.fps();
        ctx.watch_every(self.interval, "fps", fps);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
