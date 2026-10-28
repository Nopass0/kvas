use kvas_engine::prelude::*;

script_params! {
    /// Turns the camera around the vertical axis for trailers and menus.
    pub struct CameraPanFast {
        degrees_per_second: f32 = 45.0,
    }
}

impl Script for CameraPanFast {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.rotate(Vec3::new(0.0, self.degrees_per_second * ctx.dt(), 0.0));
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
