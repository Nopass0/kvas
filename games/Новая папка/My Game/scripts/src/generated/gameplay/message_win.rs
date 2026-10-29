use kvas_engine::prelude::*;

script_params! {
    /// Shows the message "You win!" once when the game starts.
    pub struct MessageWin {
        text: String = "You win!".into(),
        seconds: f32 = 6.0,
        shown: Local<bool> = Local(false),
    }
}

impl Script for MessageWin {
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
