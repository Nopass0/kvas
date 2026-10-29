use kvas_engine::prelude::*;

script_params! {
    /// Shows the message "Careful!" once when the game starts.
    pub struct MessageWarning {
        text: String = "Careful!".into(),
        seconds: f32 = 2.0,
        shown: Local<bool> = Local(false),
    }
}

impl Script for MessageWarning {
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
