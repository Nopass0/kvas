use kvas_engine::prelude::*;

script_params! {
    /// Moves the object on the ground with the WASD keys (3.0 m/s). Not physics based.
    pub struct MoveWasdWalk {
        speed: f32 = 3.0,
    }
}

impl Script for MoveWasdWalk {
    fn update(&mut self, ctx: &mut Ctx) {
        let mut dir = Vec3::ZERO;
        if ctx.key_down(Key::W) { dir.z -= 1.0; }
        if ctx.key_down(Key::S) { dir.z += 1.0; }
        if ctx.key_down(Key::A) { dir.x -= 1.0; }
        if ctx.key_down(Key::D) { dir.x += 1.0; }
        let step = dir.normalize_or_zero() * self.speed * ctx.dt();
        ctx.translate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
