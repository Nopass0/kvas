use kvas_engine::prelude::*;

script_params! {
    /// Applies the `retro` post-processing look when the game starts.
    pub struct LookRetro {
        done: Local<bool> = Local(false),
    }
}

impl Script for LookRetro {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.post_preset("retro");
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
