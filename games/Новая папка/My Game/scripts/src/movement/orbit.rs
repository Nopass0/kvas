use kvas_engine::prelude::*;

script_params! {
    /// Circles around a point and keeps looking at it: product shots, menus, cutscenes.
    pub struct Orbit {
        center: Vec3 = Vec3::ZERO,
        radius: f32 = 10.0,
        height: f32 = 4.0,
        degrees_per_second: f32 = 12.0,
    }
}

impl Script for Orbit {
    fn update(&mut self, ctx: &mut Ctx) {
        let a = (ctx.elapsed() * self.degrees_per_second).to_radians();
        let pos = self.center + Vec3::new(a.cos() * self.radius, self.height, a.sin() * self.radius);
        let d = self.center - pos;
        let t = ctx.transform_mut();
        t.position = pos;
        t.rotation = Vec3::new((d.y / d.length().max(1e-4)).asin().to_degrees(), (-d.x).atan2(-d.z).to_degrees(), 0.0);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
