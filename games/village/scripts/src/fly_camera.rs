//! FlyCamera script.

use kvas_engine::prelude::*;

script_params! {
    /// Fly around: WASD to move, Shift to go faster, Space / Ctrl for up / down, mouse to look.
    pub struct FlyCamera {
        speed: f32 = 6.0,
        sensitivity: f32 = 0.12,
        yaw: Local<f32> = Local(0.0),
        pitch: Local<f32> = Local(-6.0),
    }
}

impl Script for FlyCamera {
    fn start(&mut self, ctx: &mut Ctx) {
        let r = ctx.transform().rotation;
        self.pitch.0 = r.x;
        self.yaw.0 = r.y;
        ctx.lock_cursor(true);
        ctx.set_time_of_day(16.5);
        ctx.set_day_speed(0.3);
        ctx.hud_message("WASD - fly, mouse - look, Shift - faster, T / Y - change the hour; the day cycles by itself", 6.0);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        let look = ctx.input.mouse_delta;
        self.yaw.0 -= look.x * self.sensitivity;
        self.pitch.0 = (self.pitch.0 - look.y * self.sensitivity).clamp(-89.0, 89.0);
        let move_input = ctx.action_vector("MoveLeft", "MoveRight", "MoveBack", "MoveForward");
        let vertical = (ctx.key_down(Key::Space) as i32 - ctx.key_down(Key::Ctrl) as i32) as f32;
        let speed = self.speed * if ctx.action_down("Sprint") { 3.0 } else { 1.0 };
        let (yaw, pitch) = (self.yaw.0.to_radians(), self.pitch.0.to_radians());
        let forward = Vec3::new(-yaw.sin() * pitch.cos(), pitch.sin(), -yaw.cos() * pitch.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let step = (forward * move_input.y + right * move_input.x + Vec3::Y * vertical) * speed * dt;
        let (yaw_deg, pitch_deg) = (self.yaw.0, self.pitch.0);
        let t = ctx.transform_mut();
        t.position += step;
        t.rotation = Vec3::new(pitch_deg, yaw_deg, 0.0);

        // Time of day on keys, to see the village at dawn, noon, dusk and night.
        let hours = (ctx.key_down(Key::Y) as i32 - ctx.key_down(Key::T) as i32) as f32 * 3.0 * dt;
        if hours != 0.0 {
            let now = ctx.time_of_day();
            ctx.set_time_of_day(now + hours);
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
