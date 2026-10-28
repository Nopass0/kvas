use kvas_engine::prelude::*;

script_params! {
    /// Sets the exposure to 0.5.
    pub struct ExposureDark {
        exposure: f32 = 0.5,
        done: Local<bool> = Local(false),
    }
}

impl Script for ExposureDark {
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
