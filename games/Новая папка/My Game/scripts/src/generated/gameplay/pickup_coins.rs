use kvas_engine::prelude::*;

script_params! {
    /// Adds 5 to the shared variable `coins` when the Player touches it.
    pub struct PickupCoins {
        amount: f32 = 5.0,
        radius: f32 = 1.3,
    }
}

impl Script for PickupCoins {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            ctx.add_global("coins", self.amount);
            ctx.hud_message(&format!("+{} coins", self.amount), 1.0);
            ctx.despawn_self();
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
