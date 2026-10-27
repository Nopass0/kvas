use kvas_engine::prelude::*;

script_params! {
    /// Shows a message and an objective when the game starts.
    pub struct WelcomeMessage {
        message: String = "Welcome!".into(),
        objective: String = String::new(),
        seconds: f32 = 4.0,
    }
}

impl Script for WelcomeMessage {
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.hud_message(&self.message, self.seconds);
        if !self.objective.is_empty() {
            ctx.hud_objective(&self.objective);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
