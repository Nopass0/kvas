use kvas_engine::prelude::*;

script_params! {
    /// Broadcasts the signal `tick_5` every 5.0 s.
    pub struct HeartbeatFive {
        interval: f32 = 5.0,
    }
}

impl Script for HeartbeatFive {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.every(self.interval) {
            let _ = ctx.emit("tick_5", json!({ "elapsed": ctx.elapsed() }));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
