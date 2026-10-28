use kvas_engine::prelude::*;

script_params! {
    /// Shows or hides the object when you press Space.
    pub struct ToggleSpace {
        visible: Local<bool> = Local(true),
    }
}

impl Script for ToggleSpace {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::Space) {
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
