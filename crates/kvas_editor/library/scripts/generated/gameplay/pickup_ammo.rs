use kvas_engine::prelude::*;

script_params! {
    /// Adds 10 to the shared variable `ammo` when the Player touches it.
    pub struct PickupAmmo {
        amount: f32 = 10.0,
        radius: f32 = 1.3,
    }
}

impl Script for PickupAmmo {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {
            ctx.add_global("ammo", self.amount);
            ctx.hud_message(&format!("+{} ammo", self.amount), 1.0);
            ctx.despawn_self();
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
