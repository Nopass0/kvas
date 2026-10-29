use kvas_engine::prelude::*;

script_params! {
    /// Sets the master volume to 1.0.
    pub struct MasterVolumeFull {
        volume: f32 = 1.0,
        done: Local<bool> = Local(false),
    }
}

impl Script for MasterVolumeFull {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.done {
            *self.done = true;
            ctx.set_master_volume(self.volume);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
