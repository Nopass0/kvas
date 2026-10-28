use kvas_engine::prelude::*;

script_params! {
    /// Shows the message "Press E to interact" once when the game starts.
    pub struct MessageHint {
        text: String = "Press E to interact".into(),
        seconds: f32 = 4.0,
        shown: Local<bool> = Local(false),
    }
}

impl Script for MessageHint {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.shown {
            *self.shown = true;
            ctx.hud_message(&self.text, self.seconds);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
