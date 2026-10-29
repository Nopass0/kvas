use kvas_engine::prelude::*;

script_params! {
    /// Sets the cloud cover to 0.25.
    pub struct CloudsFew {
        amount: f32 = 0.25,
        done: Local<bool> = Local(false),
    }
}

impl Script for CloudsFew {
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
