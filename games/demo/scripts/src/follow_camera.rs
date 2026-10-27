//! FollowCamera script.

use kvas_engine::prelude::*;

/// Smoothly follows a target entity and looks at it.
pub struct FollowCamera {
    target: String,
    offset: Vec3,
    smoothing: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self { target: "Player".into(), offset: Vec3::new(0.0, 5.0, 9.0), smoothing: 4.0 }
    }
}

impl Script for FollowCamera {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(target) = ctx.find(&self.target) else { return };
        let goal = hierarchy::global_matrix(&ctx.scene.world, target).w_axis.truncate();
        let k = 1.0 - (-self.smoothing * ctx.dt()).exp();
        let offset = self.offset;
        let t = ctx.transform_mut();
        t.position = t.position.lerp(goal + offset, k);
        let d = goal - t.position;
        if d.length() > 1e-3 {
            let yaw = (-d.x).atan2(-d.z);
            let pitch = (d.y / d.length()).asin();
            t.rotation = Vec3::new(pitch.to_degrees(), yaw.to_degrees(), 0.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::entity("target", &mut self.target), Field::vec3("offset", &mut self.offset), Field::f32("smoothing", &mut self.smoothing)]
    }
}
