use kvas_engine::prelude::*;

script_params! {
    /// Moves the object on the ground with the IJKL keys (7.0 m/s). Not physics based.
    pub struct MoveIjklRun {
        speed: f32 = 7.0,
    }
}

impl Script for MoveIjklRun {
    fn update(&mut self, ctx: &mut Ctx) {
        let mut dir = Vec3::ZERO;
        if ctx.key_down(Key::I) { dir.z -= 1.0; }
        if ctx.key_down(Key::K) { dir.z += 1.0; }
        if ctx.key_down(Key::J) { dir.x -= 1.0; }
        if ctx.key_down(Key::L) { dir.x += 1.0; }
        let step = dir.normalize_or_zero() * self.speed * ctx.dt();
        ctx.translate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
