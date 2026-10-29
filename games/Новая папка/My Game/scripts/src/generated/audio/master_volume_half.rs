use kvas_engine::prelude::*;

script_params! {
    /// Sets the master volume to 0.5.
    pub struct MasterVolumeHalf {
        volume: f32 = 0.5,
        done: Local<bool> = Local(false),
    }
}

impl Script for MasterVolumeHalf {
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
