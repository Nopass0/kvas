use kvas_engine::prelude::*;

script_params! {
    /// Jumps with Space when the body is close to the ground (a physics RigidBody is required).
    pub struct Jump {
        jump_speed: f32 = 6.5,
        ground_distance: f32 = 1.1,
    }
}

impl Script for Jump {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.key_pressed(Key::Space) {
            return;
        }
        let from = ctx.position();
        let grounded = ctx.raycast(from, Vec3::NEG_Y, self.ground_distance).is_some();
        if grounded {
            let mut v = ctx.velocity();
            v.y = self.jump_speed;
            ctx.set_velocity(v);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
