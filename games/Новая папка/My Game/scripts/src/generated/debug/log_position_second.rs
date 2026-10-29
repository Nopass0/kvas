use kvas_engine::prelude::*;

script_params! {
    /// Prints the object position every 1.0 s.
    pub struct LogPositionSecond {
        interval: f32 = 1.0,
    }
}

impl Script for LogPositionSecond {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.watch_every(self.interval, "position", ctx.position());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
