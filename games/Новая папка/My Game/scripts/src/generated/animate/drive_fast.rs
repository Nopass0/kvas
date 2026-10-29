use kvas_engine::prelude::*;

script_params! {
    /// Moves forward (local -Z) at a constant speed in metres per second.
    pub struct DriveFast {
        speed: f32 = 8.0,
    }
}

impl Script for DriveFast {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.translate_local(Vec3::NEG_Z * self.speed * ctx.dt());
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
