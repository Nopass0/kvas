//! The shaders on Lida's camera, driven by the game: tape (VHS) and PS1 dither from the options,
//! the fear pulse from the Counter's distance (`ls.fear`), blood when caught.
//! Effect slots on the camera: 0 VHS, 1 fear pulse, 2 blood, 3 PS1 dither, 4 memory haze.

use crate::s_shared::*;
use kvas_engine::prelude::*;

#[derive(Default)]
pub struct CameraFx;

impl Script for CameraFx {
    fn update(&mut self, ctx: &mut Ctx) {
        let set = settings(ctx);
        let m = mode(ctx);
        let fear = if m == Mode::Playing { ctx.global_f32(FEAR, 0.0) } else { 0.0 };
        let dead = if m == Mode::Dead { (mode_time(ctx) * 1.2).min(1.0) } else { 0.0 };
        let Some(c) = ctx.get_mut::<Camera>() else { return };
        let fx = &mut c.effects;
        if let Some(e) = fx.get_mut(0) {
            e.enabled = set.vhs;
        }
        if let Some(e) = fx.get_mut(1) {
            e.enabled = fear > 0.05 && !set.reduced_motion;
            e.params = [fear, 90.0 + fear * 60.0, 0.0, 0.0];
        }
        if let Some(e) = fx.get_mut(2) {
            e.enabled = dead > 0.0;
            e.params = [dead, 0.0, 0.0, 0.0];
        }
        if let Some(e) = fx.get_mut(3) {
            e.enabled = set.ps1;
        }
    }
}
