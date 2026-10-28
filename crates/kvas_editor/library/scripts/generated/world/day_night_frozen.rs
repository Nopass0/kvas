use kvas_engine::prelude::*;

script_params! {
    /// Sets the day length: 0.0 game hours per second.
    pub struct DayNightFrozen {
        hours_per_second: f32 = 0.0,
        start_hour: f32 = 9.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for DayNightFrozen {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.set_time_of_day(self.start_hour);
            ctx.set_day_speed(self.hours_per_second);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
