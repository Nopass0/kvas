use kvas_engine::prelude::*;

script_params! {
    /// Car-style control with the ARROWS keys: forward / back and steering.
    pub struct DriveArrows {
        speed: f32 = 8.0,
        steer: f32 = 90.0,
    }
}

impl Script for DriveArrows {
    fn update(&mut self, ctx: &mut Ctx) {
        let gas = (ctx.key_down(Key::ArrowUp) as i32 - ctx.key_down(Key::ArrowDown) as i32) as f32;
        let turn = (ctx.key_down(Key::ArrowLeft) as i32 - ctx.key_down(Key::ArrowRight) as i32) as f32;
        ctx.rotate(Vec3::new(0.0, turn * self.steer * ctx.dt() * gas.signum().max(0.3), 0.0));
        ctx.translate_local(Vec3::NEG_Z * gas * self.speed * ctx.dt());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
