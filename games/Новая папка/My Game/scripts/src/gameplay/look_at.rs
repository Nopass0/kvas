use kvas_engine::prelude::*;

script_params! {
    /// Turns the object towards another object (turrets, eyes, signs).
    pub struct LookAt {
        target: EntityRef = EntityRef("Player".into()),
        turn_speed: f32 = 6.0,
    }
}

impl Script for LookAt {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(target) = self.target.resolve(ctx) else { return };
        let goal = hierarchy::global_matrix(&ctx.scene.world, target).w_axis.truncate();
        let d = goal - ctx.world_position();
        if d.length() < 1e-3 {
            return;
        }
        let yaw = (-d.x).atan2(-d.z).to_degrees();
        let k = 1.0 - (-self.turn_speed * ctx.dt()).exp();
        let t = ctx.transform_mut();
        let delta = (yaw - t.rotation.y + 540.0).rem_euclid(360.0) - 180.0;
        t.rotation.y += delta * k;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
