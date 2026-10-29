use kvas_engine::prelude::*;

script_params! {
    /// Applies the `cinematic` post-processing look when the game starts.
    pub struct LookCinematic {
        done: Local<bool> = Local(false),
    }
}

impl Script for LookCinematic {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.post_preset("cinematic");
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
