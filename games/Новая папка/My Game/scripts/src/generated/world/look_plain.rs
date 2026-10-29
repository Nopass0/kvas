use kvas_engine::prelude::*;

script_params! {
    /// Applies the `none` post-processing look when the game starts.
    pub struct LookPlain {
        done: Local<bool> = Local(false),
    }
}

impl Script for LookPlain {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.post_preset("none");
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
