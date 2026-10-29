use kvas_engine::prelude::*;

script_params! {
    /// Sets the cloud cover to 0.0.
    pub struct CloudsClear {
        amount: f32 = 0.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for CloudsClear {
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
