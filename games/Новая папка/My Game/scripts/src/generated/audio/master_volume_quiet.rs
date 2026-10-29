use kvas_engine::prelude::*;

script_params! {
    /// Sets the master volume to 0.2.
    pub struct MasterVolumeQuiet {
        volume: f32 = 0.2,
        done: Local<bool> = Local(false),
    }
}

impl Script for MasterVolumeQuiet {
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
