//! Lida's body: walking, running, crouching and footsteps. The view direction comes from the
//! `Eyes` camera (see `MouseLook`); this script only moves the capsule.
//! Publishes `ls.player` (crouch, sprint, moving) for the camera, the Counter's hearing and the HUD.

use crate::s_shared::*;
use kvas_engine::prelude::*;

pub struct PlayerController {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub crouch_speed: f32,
    wish: Vec3,
    sprint: bool,
    crouch: f32,
    step_t: f32,
    eyes: Option<Entity>,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self { walk_speed: 2.8, run_speed: 4.6, crouch_speed: 1.5, wish: Vec3::ZERO, sprint: false, crouch: 0.0, step_t: 0.0, eyes: None }
    }
}

impl Script for PlayerController {
    fn start(&mut self, ctx: &mut Ctx) {
        self.eyes = ctx.find("Eyes");
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt().min(0.1);
        if mode(ctx) != Mode::Playing {
            self.wish = Vec3::ZERO;
            return;
        }
        let yaw = self.eyes.and_then(|e| ctx.get_of::<Transform>(e)).map_or(0.0, |t| t.rotation.y).to_radians();
        let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let (x, z) = (ctx.action_axis("MoveLeft", "MoveRight"), ctx.action_axis("MoveBack", "MoveForward"));
        self.wish = (forward * z + right * x).normalize_or_zero();
        self.sprint = ctx.action_down("Sprint") && self.crouch < 0.5;
        let want_crouch = if ctx.action_down("Crouch") { 1.0 } else { 0.0 };
        self.crouch += (want_crouch - self.crouch) * (dt * 8.0).min(1.0);
        let moving = self.wish != Vec3::ZERO && ctx.velocity().with_y(0.0).length() > 0.5;
        if moving {
            self.step_t -= dt * if self.sprint { 1.6 } else { 1.0 };
            if self.step_t <= 0.0 {
                self.step_t = 0.5;
                let vol = if self.crouch > 0.5 { 0.12 } else if self.sprint { 0.55 } else { 0.3 };
                let p = ctx.world_position();
                let inside = p.x > 0.0 && p.x < 12.0 && p.z < 0.0 && p.z > -15.0;
                sfx(ctx, if inside { "sfx_step_tile" } else { "sfx_step_grass" }, vol);
            }
        }
        let state = PlayerState { crouch: self.crouch, sprint: self.sprint, moving };
        ctx.set_global(PLAYER, serde_json::to_value(state).unwrap_or_default());
    }

    fn fixed_update(&mut self, ctx: &mut Ctx) {
        let v = ctx.velocity();
        let speed = if self.crouch > 0.5 { self.crouch_speed } else if self.sprint { self.run_speed } else { self.walk_speed };
        let target = self.wish * speed;
        let mut h = Vec3::new(v.x, 0.0, v.z);
        h += (target - h) * 0.3;
        ctx.set_velocity(Vec3::new(h.x, v.y.min(0.0), h.z));
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("walk_speed", &mut self.walk_speed), Field::f32("run_speed", &mut self.run_speed), Field::f32("crouch_speed", &mut self.crouch_speed)]
    }
}
