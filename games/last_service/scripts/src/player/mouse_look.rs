//! First-person view on the `Eyes` camera: the mouse turns it, walking bobs it, crouching lowers
//! it. The camera's own rotation is the state, so the director can turn the view (loading a save,
//! cut-scenes) just by setting the transform.

use crate::s_shared::*;
use kvas_engine::prelude::*;

pub struct MouseLook {
    pub height: f32,
    pub crouch_drop: f32,
    bob: f32,
}

impl Default for MouseLook {
    fn default() -> Self {
        Self { height: 0.62, crouch_drop: 0.55, bob: 0.0 }
    }
}

impl Script for MouseLook {
    fn update(&mut self, ctx: &mut Ctx) {
        if mode(ctx) != Mode::Playing {
            return;
        }
        let dt = ctx.dt().min(0.1);
        let set = settings(ctx);
        let body = player_state(ctx);
        let locked = ctx.input.cursor_locked();
        if !locked && ctx.mouse_pressed(MouseButton::Left) {
            ctx.lock_cursor(true);
        }
        let delta = if locked { ctx.input.mouse_delta * set.sensitivity } else { Vec2::ZERO };
        if body.moving && !set.reduced_motion {
            self.bob += dt * if body.sprint { 11.0 } else { 7.5 };
        }
        let bob = if set.reduced_motion { 0.0 } else { self.bob.sin() * 0.035 };
        let roll = if body.moving && !set.reduced_motion { (self.bob * 0.5).sin() * 0.6 } else { 0.0 };
        let (height, drop) = (self.height, self.crouch_drop);
        let t = ctx.transform_mut();
        t.rotation.y -= delta.x;
        t.rotation.x = (t.rotation.x - delta.y).clamp(-85.0, 85.0);
        t.rotation.z = roll;
        t.position.y = height - body.crouch * drop + bob;
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("height", &mut self.height), Field::f32("crouch_drop", &mut self.crouch_drop)]
    }
}
