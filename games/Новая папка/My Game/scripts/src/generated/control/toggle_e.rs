use kvas_engine::prelude::*;

script_params! {
    /// Shows or hides the object when you press E.
    pub struct ToggleE {
        visible: Local<bool> = Local(true),
    }
}

impl Script for ToggleE {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::E) {
            *self.visible = !*self.visible;
            let me = ctx.entity;
            let on = *self.visible;
            ctx.set_visible(me, on);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
