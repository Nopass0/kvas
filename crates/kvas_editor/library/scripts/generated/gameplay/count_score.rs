use kvas_engine::prelude::*;

script_params! {
    /// Increments the shared variable `score` each time Space is pressed (a test counter).
    pub struct CountScore {
        step: f32 = 1.0,
    }
}

impl Script for CountScore {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::Space) {
            let value = ctx.add_global("score", self.step);
            ctx.hud_message(&format!("score: {value}"), 1.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
