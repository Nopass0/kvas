use kvas_engine::prelude::*;

script_params! {
    /// Shakes the camera for a moment when you press C (impact feeling).
    pub struct ShakeOnKeyQuake {
        strength: f32 = 1.0,
        duration: f32 = 0.4,
        time_left: Local<f32> = Local(0.0),
        origin: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for ShakeOnKeyQuake {
    fn update(&mut self, ctx: &mut Ctx) {
        let origin = *self.origin.0.get_or_insert(ctx.transform().position);
        if ctx.key_pressed(Key::C) {
            *self.time_left = self.duration;
        }
        if *self.time_left > 0.0 {
            *self.time_left -= ctx.dt();
            let k = (*self.time_left / self.duration).clamp(0.0, 1.0) * self.strength;
            let jitter = Vec3::new(ctx.random_range(-k, k), ctx.random_range(-k, k), 0.0);
            ctx.transform_mut().position = origin + jitter;
        } else {
            ctx.transform_mut().position = origin;
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
