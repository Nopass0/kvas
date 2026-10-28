use kvas_engine::prelude::*;

script_params! {
    /// Sets the exposure to 1.5.
    pub struct ExposureBright {
        exposure: f32 = 1.5,
        done: Local<bool> = Local(false),
    }
}

impl Script for ExposureBright {
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
