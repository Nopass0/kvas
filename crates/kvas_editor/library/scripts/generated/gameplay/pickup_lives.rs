use kvas_engine::prelude::*;

script_params! {
    /// Adds 1 to the shared variable `lives` when the Player touches it.
    pub struct PickupLives {
        amount: f32 = 1.0,
        radius: f32 = 1.3,
    }
}

impl Script for PickupLives {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            ctx.add_global("lives", self.amount);
            ctx.hud_message(&format!("+{} lives", self.amount), 1.0);
            ctx.despawn_self();
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
