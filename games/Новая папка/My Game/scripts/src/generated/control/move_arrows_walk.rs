use kvas_engine::prelude::*;

script_params! {
    /// Moves the object on the ground with the ARROWS keys (3.0 m/s). Not physics based.
    pub struct MoveArrowsWalk {
        speed: f32 = 3.0,
    }
}

impl Script for MoveArrowsWalk {
    fn update(&mut self, ctx: &mut Ctx) {
        let mut dir = Vec3::ZERO;
        if ctx.key_down(Key::ArrowUp) { dir.z -= 1.0; }
        if ctx.key_down(Key::ArrowDown) { dir.z += 1.0; }
        if ctx.key_down(Key::ArrowLeft) { dir.x -= 1.0; }
        if ctx.key_down(Key::ArrowRight) { dir.x += 1.0; }
        let step = dir.normalize_or_zero() * self.speed * ctx.dt();
        ctx.translate(step);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
