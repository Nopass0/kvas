use kvas_engine::prelude::*;

script_params! {
    /// Adds a jump (Space) to a physics body: only when something is under its feet.
    pub struct Jumper {
        jump_speed: f32 = 6.0,
        feet_offset: f32 = 0.55,
    }
}

impl Script for Jumper {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.action_pressed("Jump") {
            return;
        }
        let feet = ctx.world_position() - Vec3::Y * self.feet_offset;
        if ctx.raycast(feet, Vec3::NEG_Y, 0.25).is_some() {
            let v = ctx.velocity();
            ctx.set_velocity(Vec3::new(v.x, self.jump_speed, v.z));
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
