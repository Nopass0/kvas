use kvas_engine::prelude::*;

script_params! {
    /// Follows the object named `target` with an offset (9.0, 2.5, 0.0), smoothing 5.0.
    pub struct FollowSideSmooth {
        target: String = "Player".into(),
        offset: Vec3 = Vec3::new(9.0, 2.5, 0.0),
        smoothing: f32 = 5.0,
    }
}

impl Script for FollowSideSmooth {
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
