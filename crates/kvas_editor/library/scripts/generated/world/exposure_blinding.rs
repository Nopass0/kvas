use kvas_engine::prelude::*;

script_params! {
    /// Sets the exposure to 3.0.
    pub struct ExposureBlinding {
        exposure: f32 = 3.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for ExposureBlinding {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.set_exposure(self.exposure);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
