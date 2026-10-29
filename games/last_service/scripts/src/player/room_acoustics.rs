//! The room echo around Lida: a short, close reverb inside the diner, a long thin one outside.

use kvas_engine::prelude::*;

#[derive(Default)]
pub struct RoomAcoustics {
    inside: Option<bool>,
}

impl Script for RoomAcoustics {
    fn update(&mut self, ctx: &mut Ctx) {
        let p = ctx.world_position();
        let inside = p.x > 0.0 && p.x < 12.0 && p.z < 0.0 && p.z > -15.0;
        if self.inside != Some(inside) {
            self.inside = Some(inside);
            if inside {
                ctx.set_reverb(0.35, 38.0);
            } else {
                ctx.set_reverb(0.12, 110.0);
            }
        }
    }
}
