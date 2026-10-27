//! Explorer script.

use kvas_engine::prelude::*;

script_params! {
    /// Third-person ball: WASD relative to the camera, Space to jump, Shift to sprint.
    pub struct Explorer {
        force: f32 = 16.0,
        jump: f32 = 6.5,
        camera: EntityRef = EntityRef("Main Camera".into()),
        water_level: f32 = 0.0,
        /// Below this height the explorer is put back on the island.
        respawn_below: f32 = -4.0,
        spawn: Vec3 = Vec3::new(0.0, 2.0, 9.0),
        was_in_water: Local<bool> = Local(false),
        was_grounded: Local<bool> = Local(true),
        wish: Local<Vec3> = Local(Vec3::ZERO),
    }
}

impl Script for Explorer {
    fn update(&mut self, ctx: &mut Ctx) {
        let input = ctx.action_vector("MoveLeft", "MoveRight", "MoveBack", "MoveForward");
        let (forward, right) = match self.camera.resolve(ctx).and_then(|c| ctx.get_of::<Transform>(c).copied()) {
            Some(t) => (Vec3::new(t.forward().x, 0.0, t.forward().z).normalize_or_zero(), Vec3::new(t.right().x, 0.0, t.right().z).normalize_or_zero()),
            None => (Vec3::NEG_Z, Vec3::X),
        };
        let boost = if ctx.action_down("Sprint") { 1.6 } else { 1.0 };
        self.wish.0 = (right * input.x + forward * input.y).clamp_length_max(1.0) * boost;

        let pos = ctx.world_position();
        let grounded = ctx.raycast(pos, Vec3::NEG_Y, 0.65).is_some();
        if ctx.action_pressed("Jump") && grounded {
            ctx.add_impulse(Vec3::Y * self.jump);
        }
        if grounded && !self.was_grounded.0 {
            ctx.play_sound_at("sounds/land.wav", 0.5, pos);
        }
        self.was_grounded.0 = grounded;

        // Splash when the ball drops into the sea.
        let in_water = pos.y < self.water_level + 0.2 && !grounded;
        if in_water && !self.was_in_water.0 {
            ctx.play_sound_at("sounds/splash.wav", 0.9, pos);
            ctx.hud_message("Splash!", 1.0);
        }
        self.was_in_water.0 = in_water;

        if pos.y < self.respawn_below {
            ctx.teleport(self.spawn, true);
        }
    }

    fn fixed_update(&mut self, ctx: &mut Ctx) {
        let impulse = self.wish.0 * self.force * ctx.dt();
        ctx.add_impulse(impulse);
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
