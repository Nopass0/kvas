use kvas_engine::prelude::*;

script_params! {
    /// Shakes the camera when a "shake" message arrives (send it with ctx.emit("shake", json!({}))),
    /// or when the shake key is pressed.
    pub struct CameraShake {
        strength: f32 = 0.15,
        duration: f32 = 0.4,
        left: Local<f32> = Local(0.0),
        rest: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for CameraShake {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.signals("shake").is_empty() || ctx.key_pressed(Key::K) {
            self.left.0 = self.duration;
            self.rest.0 = Some(ctx.transform().position);
        }
        if self.left.0 > 0.0 {
            self.left.0 -= ctx.dt();
            let fade = (self.left.0 / self.duration).clamp(0.0, 1.0);
            let t = ctx.elapsed() * 70.0;
            let jitter = Vec3::new((t).sin(), (t * 1.3).cos(), (t * 0.7).sin()) * self.strength * fade;
            if let Some(rest) = self.rest.0 {
                ctx.transform_mut().position = rest + jitter;
                if self.left.0 <= 0.0 {
                    ctx.transform_mut().position = rest;
                }
            }
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
