use kvas_engine::prelude::*;

script_params! {
    /// Follows the object named `target` with an offset (0.0, 3.0, 7.0), smoothing 14.0.
    pub struct FollowBehindTight {
        target: String = "Player".into(),
        offset: Vec3 = Vec3::new(0.0, 3.0, 7.0),
        smoothing: f32 = 14.0,
    }
}

impl Script for FollowBehindTight {
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
