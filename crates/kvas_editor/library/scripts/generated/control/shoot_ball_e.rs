use kvas_engine::prelude::*;

script_params! {
    /// Throws a physics ball forward with E.
    pub struct ShootBallE {
        speed: f32 = 18.0,
        lifetime: f32 = 6.0,
    }
}

impl Script for ShootBallE {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.key_pressed(Key::E) {
            return;
        }
        let dir = ctx.forward();
        let at = ctx.position() + dir * 1.2;
        let ball = ctx.spawn_primitive("Ball", "sphere", at);
        ctx.scene.world.insert(ball, RigidBody::default());
        ctx.scene.world.insert(ball, Collider::default());
        ctx.set_velocity_of(ball, dir * self.speed);
        ctx.despawn_after(ball, self.lifetime);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
