use kvas_engine::prelude::*;

script_params! {
    /// Car-style control with the WASD keys: forward / back and steering.
    pub struct DriveWasd {
        speed: f32 = 8.0,
        steer: f32 = 90.0,
    }
}

impl Script for DriveWasd {
    fn update(&mut self, ctx: &mut Ctx) {
        let gas = (ctx.key_down(Key::W) as i32 - ctx.key_down(Key::S) as i32) as f32;
        let turn = (ctx.key_down(Key::A) as i32 - ctx.key_down(Key::D) as i32) as f32;
        ctx.rotate(Vec3::new(0.0, turn * self.steer * ctx.dt() * gas.signum().max(0.3), 0.0));
        ctx.translate_local(Vec3::NEG_Z * gas * self.speed * ctx.dt());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
