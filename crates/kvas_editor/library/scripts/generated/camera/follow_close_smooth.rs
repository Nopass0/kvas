use kvas_engine::prelude::*;

script_params! {
    /// Follows the object named `target` with an offset (0.0, 1.8, 3.5), smoothing 5.0.
    pub struct FollowCloseSmooth {
        target: String = "Player".into(),
        offset: Vec3 = Vec3::new(0.0, 1.8, 3.5),
        smoothing: f32 = 5.0,
    }
}

impl Script for FollowCloseSmooth {
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
