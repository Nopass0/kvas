use kvas_engine::prelude::*;

script_params! {
    /// Switches the object on and off irregularly like a broken lamp.
    pub struct FlickerSlow {
        interval: f32 = 1.0,
        chance_on: f32 = 0.6,
    }
}

impl Script for FlickerSlow {
    fn update(&mut self, ctx: &mut Ctx) {
        let me = ctx.entity;
        if ctx.every(self.interval) {
            let on = ctx.chance(self.chance_on);
            ctx.set_visible(me, on);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
