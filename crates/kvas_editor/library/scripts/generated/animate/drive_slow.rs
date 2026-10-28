use kvas_engine::prelude::*;

script_params! {
    /// Moves forward (local -Z) at a constant speed in metres per second.
    pub struct DriveSlow {
        speed: f32 = 1.0,
    }
}

impl Script for DriveSlow {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.translate_local(Vec3::NEG_Z * self.speed * ctx.dt());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
