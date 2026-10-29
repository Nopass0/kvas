use kvas_engine::prelude::*;

script_params! {
    /// Sends a signal once when the Player comes close.
    pub struct ProximitySignal {
        channel: String = "proximity".into(),
        radius: f32 = 4.0,
        fired: Local<bool> = Local(false),
    }
}

impl Script for ProximitySignal {
    fn update(&mut self, ctx: &mut Ctx) {
        if *self.fired {
            return;
        }
        let Some(player) = ctx.find("Player") else { return };
        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            *self.fired = true;
            let _ = ctx.emit(&self.channel, json!({ "from": "proximity" }));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
