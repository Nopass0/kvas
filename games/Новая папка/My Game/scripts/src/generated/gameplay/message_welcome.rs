use kvas_engine::prelude::*;

script_params! {
    /// Shows the message "Welcome!" once when the game starts.
    pub struct MessageWelcome {
        text: String = "Welcome!".into(),
        seconds: f32 = 3.0,
        shown: Local<bool> = Local(false),
    }
}

impl Script for MessageWelcome {
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
