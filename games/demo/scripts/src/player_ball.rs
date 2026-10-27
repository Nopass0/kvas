//! PlayerBall script.

use kvas_engine::prelude::*;

/// WASD / arrows roll the ball relative to the camera, Space jumps when grounded.
pub struct PlayerBall {
    force: f32,
    jump: f32,
    camera: String,
    wish: Vec3,
}

impl Default for PlayerBall {
    fn default() -> Self {
        Self { force: 14.0, jump: 6.0, camera: "Main Camera".into(), wish: Vec3::ZERO }
    }
}

impl Script for PlayerBall {
    fn update(&mut self, ctx: &mut Ctx) {
        let x = ctx.action_axis("MoveLeft", "MoveRight");
        let z = ctx.action_axis("MoveForward", "MoveBack");
        // Movement is relative to where the camera looks, flattened onto the ground.
        let (forward, right) = match ctx.find(&self.camera).and_then(|c| ctx.get_of::<Transform>(c)) {
            Some(t) => (Vec3::new(t.forward().x, 0.0, t.forward().z).normalize_or_zero(), Vec3::new(t.right().x, 0.0, t.right().z).normalize_or_zero()),
            None => (Vec3::NEG_Z, Vec3::X),
        };
        self.wish = (right * x - forward * z).clamp_length_max(1.0);

        if ctx.action_pressed("Jump") {
            let feet = ctx.world_position() - Vec3::Y * 0.55;
            if ctx.raycast(feet, Vec3::NEG_Y, 0.2).is_some() {
                ctx.add_impulse(Vec3::Y * self.jump);
            }
        }
    }
    fn fixed_update(&mut self, ctx: &mut Ctx) {
        let impulse = self.wish * self.force * ctx.dt();
        ctx.add_impulse(impulse);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("force", &mut self.force), Field::f32("jump", &mut self.jump), Field::entity("camera", &mut self.camera)]
    }
}
