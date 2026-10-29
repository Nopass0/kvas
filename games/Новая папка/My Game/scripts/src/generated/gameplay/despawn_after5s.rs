use kvas_engine::prelude::*;

script_params! {
    /// Removes the object 5 seconds after the game starts.
    pub struct DespawnAfter5s {
        seconds: f32 = 5.0,
        armed: Local<bool> = Local(false),
    }
}

impl Script for DespawnAfter5s {
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
