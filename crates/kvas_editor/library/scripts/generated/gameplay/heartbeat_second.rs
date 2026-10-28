use kvas_engine::prelude::*;

script_params! {
    /// Broadcasts the signal `tick_1` every 1.0 s.
    pub struct HeartbeatSecond {
        interval: f32 = 1.0,
    }
}

impl Script for HeartbeatSecond {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.every(self.interval) {
            let _ = ctx.emit("tick_1", json!({ "elapsed": ctx.elapsed() }));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
