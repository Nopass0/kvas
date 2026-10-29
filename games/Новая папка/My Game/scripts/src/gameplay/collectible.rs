use kvas_engine::prelude::*;

script_params! {
    /// A pickup: spins, bobs, and is collected when the player (an object named "Player") gets close.
    /// Sends a "collected" message that a game manager can count.
    pub struct Collectible {
        radius: f32 = 1.3,
        spin: f32 = 120.0,
        bob: f32 = 0.2,
        sound: SoundRef = SoundRef(String::new()),
        base_y: Local<Option<f32>> = Local(None),
    }
}

impl Script for Collectible {
    fn update(&mut self, ctx: &mut Ctx) {
        let (dt, time) = (ctx.dt(), ctx.elapsed());
        let base = *self.base_y.0.get_or_insert(ctx.transform().position.y);
        let t = ctx.transform_mut();
        t.rotation.y += self.spin * dt;
        t.position.y = base + (time * 2.0).sin() * self.bob;
        let Some(player) = ctx.find("Player") else { return };
        let there = hierarchy::global_matrix(&ctx.scene.world, player).w_axis.truncate();
        let here = ctx.world_position();
        if here.distance(there) < self.radius {
            if !self.sound.0.is_empty() {
                ctx.play_sound_at(&self.sound.0, 1.0, here);
            }
            let _ = ctx.emit("collected", json!({}));
            let me = ctx.entity;
            ctx.despawn(me);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
