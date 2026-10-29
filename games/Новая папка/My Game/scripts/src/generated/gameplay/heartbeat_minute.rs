use kvas_engine::prelude::*;

script_params! {
    /// Broadcasts the signal `tick_60` every 60.0 s.
    pub struct HeartbeatMinute {
        interval: f32 = 60.0,
    }
}

impl Script for HeartbeatMinute {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.every(self.interval) {
            let _ = ctx.emit("tick_60", json!({ "elapsed": ctx.elapsed() }));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
