use kvas_engine::prelude::*;

script_params! {
    /// Increments the shared variable `clicks` each time Space is pressed (a test counter).
    pub struct CountClicks {
        step: f32 = 1.0,
    }
}

impl Script for CountClicks {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::Space) {
            let value = ctx.add_global("clicks", self.step);
            ctx.hud_message(&format!("clicks: {value}"), 1.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
