use kvas_engine::prelude::*;

script_params! {
    /// Sets the time of day to 20.0 h and freezes it.
    pub struct SetTimeEvening {
        hour: f32 = 20.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for SetTimeEvening {
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
