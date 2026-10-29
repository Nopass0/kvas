use kvas_engine::prelude::*;

script_params! {
    /// Shows the message "Find the exit" once when the game starts.
    pub struct MessageObjective {
        text: String = "Find the exit".into(),
        seconds: f32 = 5.0,
        shown: Local<bool> = Local(false),
    }
}

impl Script for MessageObjective {
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
