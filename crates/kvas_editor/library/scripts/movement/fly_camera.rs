use kvas_engine::prelude::*;

script_params! {
    /// Free-fly camera: WASD, mouse look, Space / Ctrl for up and down, Shift to go faster.
    pub struct FlyCamera {
        speed: f32 = 6.0,
        sensitivity: f32 = 0.12,
        yaw: Local<f32> = Local(0.0),
        pitch: Local<f32> = Local(0.0),
    }
}

impl Script for FlyCamera {
    fn start(&mut self, ctx: &mut Ctx) {
        let r = ctx.transform().rotation;
        self.pitch.0 = r.x;
        self.yaw.0 = r.y;
        ctx.lock_cursor(true);
    }
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        let look = ctx.input.mouse_delta;
        self.yaw.0 -= look.x * self.sensitivity;
        self.pitch.0 = (self.pitch.0 - look.y * self.sensitivity).clamp(-89.0, 89.0);
        let input = ctx.action_vector("MoveLeft", "MoveRight", "MoveBack", "MoveForward");
        let vertical = (ctx.key_down(Key::Space) as i32 - ctx.key_down(Key::Ctrl) as i32) as f32;
        let speed = self.speed * if ctx.action_down("Sprint") { 3.0 } else { 1.0 };
        let (yaw, pitch) = (self.yaw.0.to_radians(), self.pitch.0.to_radians());
        let forward = Vec3::new(-yaw.sin() * pitch.cos(), pitch.sin(), -yaw.cos() * pitch.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let step = (forward * input.y + right * input.x + Vec3::Y * vertical) * speed * dt;
        let (pitch_deg, yaw_deg) = (self.pitch.0, self.yaw.0);
        let t = ctx.transform_mut();
        t.position += step;
        t.rotation = Vec3::new(pitch_deg, yaw_deg, 0.0);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
