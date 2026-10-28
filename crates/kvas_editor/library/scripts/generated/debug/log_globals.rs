use kvas_engine::prelude::*;

script_params! {
    /// Prints the shared variable `score` every 2 s.
    pub struct LogGlobals {
        key: String = "score".into(),
    }
}

impl Script for LogGlobals {
    fn update(&mut self, ctx: &mut Ctx) {
        let value = ctx.global_f32(&self.key, 0.0);
        ctx.watch_every(2.0, &self.key, value);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
