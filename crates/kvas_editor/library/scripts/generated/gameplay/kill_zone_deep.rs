use kvas_engine::prelude::*;

script_params! {
    /// Teleports the object back to where it started when it falls below y = -100.0.
    pub struct KillZoneDeep {
        min_y: f32 = -100.0,
        start: Local<Option<Vec3>> = Local(None),
    }
}

impl Script for KillZoneDeep {
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
