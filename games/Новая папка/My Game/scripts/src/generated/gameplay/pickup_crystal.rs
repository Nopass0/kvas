use kvas_engine::prelude::*;

script_params! {
    /// A pickup worth 25 point(s): spins, and is collected by the object named Player. Adds to the shared variable `score`.
    pub struct PickupCrystal {
        points: f32 = 25.0,
        radius: f32 = 1.0,
        spin: f32 = 120.0,
        sound: SoundRef = SoundRef(String::new()),
    }
}

impl Script for PickupCrystal {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.rotate(Vec3::new(0.0, self.spin * ctx.dt(), 0.0));
        let Some(player) = ctx.find("Player") else { return };
        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            ctx.add_global("score", self.points);
            if !self.sound.0.is_empty() {
                let at = ctx.position();
                ctx.play_sound_at(&self.sound.0, 1.0, at);
            }
            ctx.despawn_self();
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
