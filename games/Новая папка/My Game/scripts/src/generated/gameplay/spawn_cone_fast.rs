use kvas_engine::prelude::*;

script_params! {
    /// Drops a physics cone every 0.4 s above the object (at most `max_alive` at a time).
    pub struct SpawnConeFast {
        interval: f32 = 0.4,
        max_alive: i32 = 60,
        height: f32 = 4.0,
        spread: f32 = 2.0,
        alive: Local<Vec<Entity>> = Local(Vec::new()),
    }
}

impl Script for SpawnConeFast {
    fn update(&mut self, ctx: &mut Ctx) {
        self.alive.retain(|e| ctx.is_alive(*e));
        if !ctx.every(self.interval) || self.alive.len() as i32 >= self.max_alive {
            return;
        }
        let at = ctx.position() + Vec3::new(ctx.random_range(-self.spread, self.spread), self.height, ctx.random_range(-self.spread, self.spread));
        let e = ctx.spawn_primitive("cone", "cone", at);
        ctx.scene.world.insert(e, RigidBody::default());
        ctx.scene.world.insert(e, Collider::default());
        self.alive.push(e);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
