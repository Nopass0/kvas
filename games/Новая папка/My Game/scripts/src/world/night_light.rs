use kvas_engine::prelude::*;

script_params! {
    /// A lamp that switches on at dusk, flickers like a flame and goes out at dawn.
    pub struct NightLight {
        intensity: f32 = 2.5,
        flicker: f32 = 0.15,
    }
}

fn darkness(hour: f32) -> f32 {
    let smooth = |a: f32, b: f32, x: f32| {
        let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    if hour >= 12.0 { smooth(17.5, 19.5, hour) } else { 1.0 - smooth(4.5, 6.5, hour) }
}

impl Script for NightLight {
    fn update(&mut self, ctx: &mut Ctx) {
        let on = darkness(ctx.time_of_day());
        let t = ctx.elapsed() + ctx.entity.index() as f32 * 1.7;
        let noise = (t * 13.0).sin() * 0.5 + (t * 31.0).sin() * 0.3 + (t * 7.0).sin() * 0.2;
        let level = self.intensity * on * (1.0 + noise * self.flicker);
        if let Some(light) = ctx.get_mut::<Light>() {
            light.intensity = level;
        }
        if let Some(mesh) = ctx.get_mut::<MeshRenderer>() {
            mesh.emissive = 0.05 + 1.4 * on;
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
