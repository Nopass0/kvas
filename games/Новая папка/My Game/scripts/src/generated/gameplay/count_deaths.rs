use kvas_engine::prelude::*;

script_params! {
    /// Increments the shared variable `deaths` each time Space is pressed (a test counter).
    pub struct CountDeaths {
        step: f32 = 1.0,
    }
}

impl Script for CountDeaths {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::Space) {
            let value = ctx.add_global("deaths", self.step);
            ctx.hud_message(&format!("deaths: {value}"), 1.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
