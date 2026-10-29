use kvas_engine::prelude::*;

script_params! {
    /// Adds 50 to the shared variable `energy` when the Player touches it.
    pub struct PickupEnergy {
        amount: f32 = 50.0,
        radius: f32 = 1.3,
    }
}

impl Script for PickupEnergy {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            ctx.add_global("energy", self.amount);
            ctx.hud_message(&format!("+{} energy", self.amount), 1.0);
            ctx.despawn_self();
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
