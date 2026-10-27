//! FpsPlayer script.

use kvas_engine::prelude::*;

script_params! {
    /// First-person player on a capsule body: WASD, mouse look, Space to jump, Shift to sprint,
    /// Ctrl to crouch, Esc to free or recapture the mouse. Put it on the body (capsule collider +
    /// rigid body with "lock rotation") and make the camera a child object named "Camera".
    pub struct FpsPlayer {
        walk_speed: f32 = 4.2,
        sprint_speed: f32 = 7.5,
        crouch_speed: f32 = 2.2,
        jump_speed: f32 = 5.2,
        /// How quickly the player reaches the wanted speed (higher = snappier).
        acceleration: f32 = 14.0,
        look_sensitivity: f32 = 0.11,
        /// Camera height above the body centre in metres.
        eye_height: f32 = 0.7,
        camera: EntityRef = EntityRef("Camera".into()),
        yaw: Local<f32> = Local(0.0),
        pitch: Local<f32> = Local(0.0),
        crouch: Local<f32> = Local(0.0),
        bob: Local<f32> = Local(0.0),
        captured: Local<bool> = Local(true),
    }
}

impl Script for FpsPlayer {
    fn start(&mut self, ctx: &mut Ctx) {
        // Hide the cursor and use the mouse for looking as soon as the game starts.
        ctx.lock_cursor(true);
        ctx.hud_crosshair(true);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        if ctx.key_pressed(Key::Escape) {
            self.captured.0 = !self.captured.0;
            ctx.lock_cursor(self.captured.0);
        }
        if self.captured.0 {
            let look = ctx.input.mouse_delta;
            self.yaw.0 -= look.x * self.look_sensitivity;
            self.pitch.0 = (self.pitch.0 - look.y * self.look_sensitivity).clamp(-85.0, 85.0);
        }

        // Movement relative to where the player looks.
        let input = ctx.action_vector("MoveLeft", "MoveRight", "MoveBack", "MoveForward");
        let crouching = ctx.key_down(Key::Ctrl);
        let speed = if crouching { self.crouch_speed } else if ctx.action_down("Sprint") { self.sprint_speed } else { self.walk_speed };
        let yaw = self.yaw.0.to_radians();
        let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let wish = (forward * input.y + right * input.x).clamp_length_max(1.0) * speed;
        let velocity = ctx.velocity();
        let k = 1.0 - (-self.acceleration * dt).exp();
        let horizontal = Vec3::new(velocity.x, 0.0, velocity.z).lerp(wish, k);

        // Standing on something? A short ray just below the capsule tells.
        let feet = ctx.world_position() - Vec3::Y * 0.95;
        let grounded = ctx.raycast(feet, Vec3::NEG_Y, 0.25).is_some();
        let mut vertical = velocity.y;
        if grounded && ctx.action_pressed("Jump") {
            vertical = self.jump_speed;
        }
        ctx.set_velocity(Vec3::new(horizontal.x, vertical, horizontal.z));

        // Camera: look angles, smooth crouch and a little head bob while walking.
        self.crouch.0 += ((crouching as i32 as f32) - self.crouch.0) * (1.0 - (-10.0 * dt).exp());
        let moving = (horizontal.length() / self.walk_speed).min(1.5) * grounded as i32 as f32;
        self.bob.0 += horizontal.length() * dt * 2.2;
        let height = self.eye_height - 0.45 * self.crouch.0 + self.bob.0.sin() * 0.035 * moving;
        let (pitch, yaw_deg) = (self.pitch.0, self.yaw.0);
        if let Some(cam) = self.camera.resolve(ctx) {
            if let Some(t) = ctx.get_mut_of::<Transform>(cam) {
                t.position = Vec3::new(0.0, height, 0.0);
                t.rotation = Vec3::new(pitch, yaw_deg, 0.0);
            }
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
