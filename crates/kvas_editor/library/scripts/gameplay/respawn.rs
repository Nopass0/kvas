use kvas_engine::prelude::*;

script_params! {
    /// Puts the object back at its start (or a chosen point) when it falls out of the world.
    pub struct Respawn {
        below_y: f32 = -20.0,
        start: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for Respawn {
    fn update(&mut self, ctx: &mut Ctx) {
        let pos = ctx.world_position();
        let start = *self.start.0.get_or_insert(pos);
        if pos.y < self.below_y {
            ctx.teleport(start, true);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
