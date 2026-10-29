use kvas_engine::prelude::*;

script_params! {
    /// Follows the object named `target` with an offset (1.2, 1.9, 3.0), smoothing 14.0.
    pub struct FollowShoulderTight {
        target: String = "Player".into(),
        offset: Vec3 = Vec3::new(1.2, 1.9, 3.0),
        smoothing: f32 = 14.0,
    }
}

impl Script for FollowShoulderTight {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find(&self.target).and_then(|e| ctx.position_of(e)) else { return };
        let wanted = player + self.offset;
        let here = ctx.position();
        let next = here.lerp(wanted, 1.0 - (-self.smoothing * ctx.dt()).exp());
        ctx.set_position(next);
        ctx.look_at(player);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
