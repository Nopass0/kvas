use kvas_engine::prelude::*;

script_params! {
    /// Sets the time of day to 9.0 h and freezes it.
    pub struct SetTimeMorning {
        hour: f32 = 9.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for SetTimeMorning {
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
