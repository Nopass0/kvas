use kvas_engine::prelude::*;

script_params! {
    /// Puts the object back at its start after 3.0 s without moving (stuck objects).
    pub struct Respawn {
        seconds: f32 = 3.0,
        start: Local<Option<Vec3>> = Local(None),
        last: Local<Option<Vec3>> = Local(None),
        still: Local<f32> = Local(0.0),
    }
}

impl Script for Respawn {
    fn update(&mut self, ctx: &mut Ctx) {
        let start = *self.start.0.get_or_insert(ctx.position());
        let here = ctx.position();
        let moved = self.last.0.is_none_or(|p| p.distance(here) > 0.01);
        *self.last.0.get_or_insert(here) = here;
        *self.still = if moved { 0.0 } else { *self.still + ctx.dt() };
        if *self.still > self.seconds {
            ctx.teleport(start, true);
            *self.still = 0.0;
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
