//! Spins its object around one local axis (fan blades, a ceiling fan, a radar).
//! `speed` is in degrees per second; `axis` is "x", "y" or "z"; `spin_up` seconds to full speed.

use kvas_engine::prelude::*;

pub struct Spinner {
    pub speed: f32,
    pub axis: String,
    pub spin_up: f32,
    current: f32,
}

impl Default for Spinner {
    fn default() -> Self {
        Self { speed: 360.0, axis: "z".into(), spin_up: 2.0, current: 0.0 }
    }
}

impl Script for Spinner {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        let rate = if self.spin_up > 0.0 { self.speed.abs() / self.spin_up } else { f32::MAX };
        self.current += (self.speed - self.current).clamp(-rate * dt, rate * dt);
        let step = self.current * dt;
        let r = &mut ctx.transform_mut().rotation;
        match self.axis.as_str() {
            "x" => r.x = (r.x + step) % 360.0,
            "y" => r.y = (r.y + step) % 360.0,
            _ => r.z = (r.z + step) % 360.0,
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("speed", &mut self.speed), Field::text("axis", &mut self.axis), Field::f32("spin_up", &mut self.spin_up)]
    }
}
