use kvas_engine::prelude::*;

script_params! {
    /// Cycles the object colour through the rainbow.
    pub struct ColorCycleSlow {
        cycles_per_second: f32 = 0.05,
        saturation: f32 = 0.8,
        brightness: f32 = 1.0,
    }
}

impl Script for ColorCycleSlow {
    fn update(&mut self, ctx: &mut Ctx) {
        let me = ctx.entity;
        let h = (ctx.elapsed() * self.cycles_per_second).fract();
        let c = hsv_to_rgb(h, self.saturation, self.brightness);
        ctx.set_color(me, [c[0], c[1], c[2], 1.0]);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
