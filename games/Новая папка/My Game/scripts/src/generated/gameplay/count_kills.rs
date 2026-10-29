use kvas_engine::prelude::*;

script_params! {
    /// Increments the shared variable `kills` each time Space is pressed (a test counter).
    pub struct CountKills {
        step: f32 = 1.0,
    }
}

impl Script for CountKills {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::Space) {
            let value = ctx.add_global("kills", self.step);
            ctx.hud_message(&format!("kills: {value}"), 1.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
