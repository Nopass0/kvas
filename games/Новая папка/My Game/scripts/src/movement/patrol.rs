use kvas_engine::prelude::*;

script_params! {
    /// Moves back and forth along an axis (a moving platform, a patrolling enemy).
    pub struct Patrol {
        direction: Vec3 = Vec3::new(1.0, 0.0, 0.0),
        distance: f32 = 4.0,
        speed: f32 = 1.0,
        origin: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for Patrol {
    fn update(&mut self, ctx: &mut Ctx) {
        let origin = *self.origin.0.get_or_insert(ctx.transform().position);
        let offset = self.direction.normalize_or_zero() * (ctx.elapsed() * self.speed).sin() * self.distance;
        ctx.transform_mut().position = origin + offset;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
