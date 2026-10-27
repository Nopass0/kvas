//! FollowCamera script.

use kvas_engine::prelude::*;

script_params! {
    /// Smoothly follows a target and looks at it.
    pub struct FollowCamera {
        target: EntityRef = EntityRef("Explorer".into()),
        offset: Vec3 = Vec3::new(0.0, 5.5, 10.0),
        smoothing: f32 = 3.5,
    }
}

impl Script for FollowCamera {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(target) = self.target.resolve(ctx) else { return };
        let goal = hierarchy::global_matrix(&ctx.scene.world, target).w_axis.truncate();
        let k = 1.0 - (-self.smoothing * ctx.dt()).exp();
        let offset = self.offset;
        let t = ctx.transform_mut();
        t.position = t.position.lerp(goal + offset, k);
        let d = goal - t.position;
        if d.length() > 1e-3 {
            t.rotation = Vec3::new((d.y / d.length()).asin().to_degrees(), (-d.x).atan2(-d.z).to_degrees(), 0.0);
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
