use kvas_engine::prelude::*;

script_params! {
    /// Runs the sky clock. Clouds thicken towards evening. Keys T / Y change the hour by hand.
    pub struct DayNightCycle {
        hours_per_second: f32 = 0.1,
        start_hour: f32 = 9.0,
    }
}

impl Script for DayNightCycle {
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.set_time_of_day(self.start_hour);
        ctx.set_day_speed(self.hours_per_second);
    }
    fn update(&mut self, ctx: &mut Ctx) {
        let hour = ctx.time_of_day();
        let evening = ((hour - 15.0) / 5.0).clamp(0.0, 1.0);
        ctx.set_clouds(0.3 + 0.4 * evening);
        let nudge = (ctx.key_down(Key::Y) as i32 - ctx.key_down(Key::T) as i32) as f32 * 3.0 * ctx.dt();
        if nudge != 0.0 {
            ctx.set_time_of_day(hour + nudge);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
