use kvas_engine::prelude::*;

script_params! {
    /// Teleports the object back to where it started when it falls below y = -5.0.
    pub struct KillZoneShallow {
        min_y: f32 = -5.0,
        start: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for KillZoneShallow {
    fn update(&mut self, ctx: &mut Ctx) {
        let start = *self.start.0.get_or_insert(ctx.position());
        if ctx.position().y < self.min_y {
            ctx.teleport(start, true);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
