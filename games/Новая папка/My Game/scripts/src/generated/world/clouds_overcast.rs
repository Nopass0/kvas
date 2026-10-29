use kvas_engine::prelude::*;

script_params! {
    /// Sets the cloud cover to 1.0.
    pub struct CloudsOvercast {
        amount: f32 = 1.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for CloudsOvercast {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.set_clouds(self.amount);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
