use kvas_engine::prelude::*;

script_params! {
    /// Hold the right mouse button to zoom the camera in (a field-of-view change), release to return.
    pub struct ZoomOnKey {
        zoomed_fov: f32 = 30.0,
        speed: f32 = 8.0,
        normal: Local<Option<f32>> = Local(None),
    }
}

impl Script for ZoomOnKey {
    fn update(&mut self, ctx: &mut Ctx) {
        let normal = *self.normal.0.get_or_insert(ctx.get::<Camera>().map_or(60.0, |c| c.fov));
        let goal = if ctx.mouse_down(MouseButton::Right) { self.zoomed_fov } else { normal };
        let k = 1.0 - (-self.speed * ctx.dt()).exp();
        if let Some(cam) = ctx.get_mut::<Camera>() {
            cam.fov += (goal - cam.fov) * k;
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
