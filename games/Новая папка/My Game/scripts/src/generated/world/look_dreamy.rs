use kvas_engine::prelude::*;

script_params! {
    /// Applies the `dreamy` post-processing look when the game starts.
    pub struct LookDreamy {
        done: Local<bool> = Local(false),
    }
}

impl Script for LookDreamy {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.post_preset("dreamy");
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
