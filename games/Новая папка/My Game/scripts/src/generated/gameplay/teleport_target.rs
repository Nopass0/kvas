use kvas_engine::prelude::*;

script_params! {
    /// Moves the object to the object named Target when T is pressed.
    pub struct TeleportTarget {
        target: String = "Target".into(),
    }
}

impl Script for TeleportTarget {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::T) {
            if let Some(p) = ctx.find(&self.target).and_then(|e| ctx.position_of(e)) {
                ctx.teleport(p, true);
            }
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
