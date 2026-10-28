use kvas_engine::prelude::*;

script_params! {
    /// Prints the object position every 0.1 s.
    pub struct LogPositionTenth {
        interval: f32 = 0.1,
    }
}

impl Script for LogPositionTenth {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.watch_every(self.interval, "position", ctx.position());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
