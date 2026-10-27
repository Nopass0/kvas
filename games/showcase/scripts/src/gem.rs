//! Gem script.

use kvas_engine::prelude::*;

script_params! {
    /// A collectible: spins, bobs and is picked up when the explorer gets close.
    pub struct Gem {
        pickup_radius: f32 = 1.3,
        spin: f32 = 120.0,
        bob: f32 = 0.2,
        clip: SoundRef = SoundRef("sounds/gem.wav".into()),
        base_y: Local<Option<f32>> = Local(None),
    }
}

impl Script for Gem {
    fn update(&mut self, ctx: &mut Ctx) {
        let (dt, time) = (ctx.dt(), ctx.elapsed());
        let base = *self.base_y.0.get_or_insert(ctx.transform().position.y);
        let t = ctx.transform_mut();
        t.rotation.y += self.spin * dt;
        t.position.y = base + (time * 2.0).sin() * self.bob;

        let Some(player) = ctx.find("Explorer") else { return };
        let here = ctx.world_position();
        let there = hierarchy::global_matrix(&ctx.scene.world, player).w_axis.truncate();
        if here.distance(there) < self.pickup_radius {
            ctx.play_sound_at(&self.clip.0, 1.0, here);
            let _ = ctx.emit("gem", json!({}));
            let me = ctx.entity;
            ctx.despawn(me);
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
