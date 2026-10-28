use kvas_engine::prelude::*;

script_params! {
    /// Adds 25 to the shared variable `health` when the Player touches it.
    pub struct PickupHealth {
        amount: f32 = 25.0,
        radius: f32 = 1.3,
    }
}

impl Script for PickupHealth {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            ctx.add_global("health", self.amount);
            ctx.hud_message(&format!("+{} health", self.amount), 1.0);
            ctx.despawn_self();
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
