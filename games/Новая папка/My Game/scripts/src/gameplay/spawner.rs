use kvas_engine::prelude::*;

script_params! {
    /// Drops a new physics primitive at this object every few seconds, and removes it later.
    pub struct Spawner {
        interval: f32 = 1.5,
        primitive: String = "cube".into(),
        lifetime: f32 = 12.0,
        timer: Local<f32> = Local(0.0),
        count: Local<u32> = Local(0),
    }
}

impl Script for Spawner {
    fn update(&mut self, ctx: &mut Ctx) {
        self.timer.0 += ctx.dt();
        if self.timer.0 < self.interval {
            return;
        }
        self.timer.0 = 0.0;
        self.count.0 += 1;
        let at = ctx.world_position();
        let e = ctx.spawn_primitive(&format!("Spawned {}", self.count.0), &self.primitive, at);
        ctx.insert(e, Collider::default());
        ctx.insert(e, RigidBody::default());
        ctx.despawn_after(e, self.lifetime);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
