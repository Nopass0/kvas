use kvas_engine::prelude::*;

script_params! {
    /// Sends the signal `open` when the Player is near and presses Enter.
    pub struct ButtonEnter {
        channel: String = "open".into(),
        radius: f32 = 2.0,
    }
}

impl Script for ButtonEnter {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        if ctx.key_pressed(Key::Enter) && ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            let _ = ctx.emit(&self.channel, json!({}));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
