use kvas_engine::prelude::*;

script_params! {
    /// Counts "collected" messages and shows the score; announces the win.
    pub struct ScoreCounter {
        goal: i32 = 5,
        label: String = "Collected".into(),
        win_text: String = "You win!".into(),
        score: Local<i32> = Local(0),
    }
}

impl Script for ScoreCounter {
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.hud_objective(&format!("{}: 0 / {}", self.label, self.goal));
    }
    fn update(&mut self, ctx: &mut Ctx) {
        let n = ctx.signals("collected").len() as i32;
        if n > 0 {
            self.score.0 += n;
            ctx.hud_objective(&format!("{}: {} / {}", self.label, self.score.0, self.goal));
            if self.score.0 >= self.goal {
                ctx.hud_message(&self.win_text, 6.0);
            }
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
