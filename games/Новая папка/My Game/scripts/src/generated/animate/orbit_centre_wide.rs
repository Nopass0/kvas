use kvas_engine::prelude::*;

script_params! {
    /// Circles around the point where the object started (or an object named in `center`).
    pub struct OrbitCentreWide {
        radius: f32 = 10.0,
        degrees_per_second: f32 = 20.0,
        center: String = "".into(),
        start: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for OrbitCentreWide {
    fn update(&mut self, ctx: &mut Ctx) {
        let start = *self.start.0.get_or_insert(ctx.position());
        let center = ctx.find(&self.center).and_then(|e| ctx.position_of(e)).unwrap_or(start);
        let angle = (ctx.elapsed() * self.degrees_per_second).to_radians();
        let at = center + Vec3::new(angle.cos(), 0.0, angle.sin()) * self.radius + Vec3::Y * (start.y - center.y);
        ctx.set_position(at);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
