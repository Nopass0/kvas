use kvas_engine::prelude::*;

script_params! {
    /// Car-style control with the IJKL keys: forward / back and steering.
    pub struct DriveIjkl {
        speed: f32 = 8.0,
        steer: f32 = 90.0,
    }
}

impl Script for DriveIjkl {
    fn update(&mut self, ctx: &mut Ctx) {
        let gas = (ctx.key_down(Key::I) as i32 - ctx.key_down(Key::K) as i32) as f32;
        let turn = (ctx.key_down(Key::J) as i32 - ctx.key_down(Key::L) as i32) as f32;
        ctx.rotate(Vec3::new(0.0, turn * self.steer * ctx.dt() * gas.signum().max(0.3), 0.0));
        ctx.translate_local(Vec3::NEG_Z * gas * self.speed * ctx.dt());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
