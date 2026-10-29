//! The title-screen camera: a slow, breathing look at the lit diner from the road.

use crate::s_shared::*;
use kvas_engine::prelude::*;

#[derive(Default)]
pub struct MenuCamera {
    home: Option<Vec3>,
}

impl Script for MenuCamera {
    fn update(&mut self, ctx: &mut Ctx) {
        let home = *self.home.get_or_insert(ctx.transform().position);
        if mode(ctx) != Mode::Menu {
            return;
        }
        let t = ctx.elapsed();
        let vhs = settings(ctx).vhs;
        if let Some(e) = ctx.get_mut::<Camera>().and_then(|c| c.effects.get_mut(0)) {
            e.enabled = vhs;
        }
        ctx.transform_mut().position = home + Vec3::new((t * 0.05).sin() * 0.6, (t * 0.31).sin() * 0.03, 0.0);
    }
}
