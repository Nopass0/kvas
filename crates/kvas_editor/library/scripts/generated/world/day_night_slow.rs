use kvas_engine::prelude::*;

script_params! {
    /// Sets the day length: 0.05 game hours per second.
    pub struct DayNightSlow {
        hours_per_second: f32 = 0.05,
        start_hour: f32 = 9.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for DayNightSlow {
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
