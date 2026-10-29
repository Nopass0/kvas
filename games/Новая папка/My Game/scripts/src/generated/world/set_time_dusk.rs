use kvas_engine::prelude::*;

script_params! {
    /// Sets the time of day to 18.5 h and freezes it.
    pub struct SetTimeDusk {
        hour: f32 = 18.5,
        done: Local<bool> = Local(false),
    }
}

impl Script for SetTimeDusk {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.set_time_of_day(self.hour);
            ctx.set_day_speed(0.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
