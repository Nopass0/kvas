use kvas_engine::prelude::*;

script_params! {
    /// Applies the `crisp` post-processing look when the game starts.
    pub struct LookCrisp {
        done: Local<bool> = Local(false),
    }
}

impl Script for LookCrisp {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.post_preset("crisp");
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
