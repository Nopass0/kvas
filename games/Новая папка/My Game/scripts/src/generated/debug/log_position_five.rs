use kvas_engine::prelude::*;

script_params! {
    /// Prints the object position every 5.0 s.
    pub struct LogPositionFive {
        interval: f32 = 5.0,
    }
}

impl Script for LogPositionFive {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.watch_every(self.interval, "position", ctx.position());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
