use kvas_engine::prelude::*;

script_params! {
    /// Broadcasts the signal `tick_10` every 10.0 s.
    pub struct HeartbeatTen {
        interval: f32 = 10.0,
    }
}

impl Script for HeartbeatTen {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.every(self.interval) {
            let _ = ctx.emit("tick_10", json!({ "elapsed": ctx.elapsed() }));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
