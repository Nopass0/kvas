use kvas_engine::prelude::*;

script_params! {
    /// Removes the object 2 seconds after the game starts.
    pub struct DespawnAfter2s {
        seconds: f32 = 2.0,
        armed: Local<bool> = Local(false),
    }
}

impl Script for DespawnAfter2s {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.armed {
            *self.armed = true;
            let me = ctx.entity;
            ctx.despawn_after(me, self.seconds);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
