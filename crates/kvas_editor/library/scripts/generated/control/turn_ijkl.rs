use kvas_engine::prelude::*;

script_params! {
    /// Turns left and right with the IJKL keys.
    pub struct TurnIjkl {
        degrees_per_second: f32 = 120.0,
    }
}

impl Script for TurnIjkl {
    fn update(&mut self, ctx: &mut Ctx) {
        let turn = (ctx.key_down(Key::J) as i32 - ctx.key_down(Key::L) as i32) as f32;
        ctx.rotate(Vec3::new(0.0, turn * self.degrees_per_second * ctx.dt(), 0.0));
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
