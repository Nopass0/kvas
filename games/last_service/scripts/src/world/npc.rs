//! A character who lives in the scene: busy with `work_clip` (Anna wipes the counter, Pavel
//! looks around), turns to Lida when she comes within `notice` metres, and plays `talk_clip`
//! while one of `speaker`'s lines is on screen (the `ls.say` subtitles).
//! The model needs an `Animator`; the director can change `work_clip` (Anna sits at the end).

use crate::s_shared::*;
use kvas_engine::prelude::*;

pub struct Npc {
    pub speaker: String,
    pub work_clip: String,
    pub idle_clip: String,
    pub talk_clip: String,
    pub notice: f32,
    pub turn_speed: f32,
    talking: f32,
    home_yaw: Option<f32>,
    player: Option<Entity>,
    eyes: Option<Entity>,
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            speaker: String::new(),
            work_clip: "Idle".into(),
            idle_clip: "Idle".into(),
            talk_clip: "Talk".into(),
            notice: 3.0,
            turn_speed: 140.0,
            talking: 0.0,
            home_yaw: None,
            player: None,
            eyes: None,
        }
    }
}

impl Script for Npc {
    fn start(&mut self, ctx: &mut Ctx) {
        self.player = ctx.find("Lida");
        self.eyes = ctx.find("Eyes");
        self.home_yaw = Some(ctx.transform().rotation.y);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt().min(0.1);
        for s in ctx.signals("ls.say") {
            if s.payload["who"].as_str() == Some(self.speaker.as_str()) {
                self.talking = self.talking.max(s.payload["time"].as_f64().unwrap_or(3.0) as f32);
            }
        }
        self.talking = (self.talking - dt).max(0.0);
        let me = ctx.world_position();
        let player = self.player.and_then(|p| ctx.position_of(p)).unwrap_or(me + Vec3::splat(100.0));
        let to = (player - me).with_y(0.0);
        let near = mode(ctx) == Mode::Playing && to.length() < self.notice;
        let sitting = self.work_clip == "Sit";
        let clip = if sitting {
            &self.work_clip
        } else if self.talking > 0.0 {
            &self.talk_clip
        } else if near {
            &self.idle_clip
        } else {
            &self.work_clip
        };
        let me_e = ctx.entity;
        ctx.play_animation(me_e, &clip.clone());
        // Eye contact while Lida is close or while talking (IK turns the head and chest).
        let eyes = self.eyes.map(|e| ctx.world_matrix_of(e).w_axis.truncate());
        ctx.look_at_point(me_e, if near || self.talking > 0.0 { eyes } else { None });
        // Turn to Lida while she is close or while talking, back to the work afterwards. A
        // seated character stays as the director placed her.
        if sitting {
            return;
        }
        let home = *self.home_yaw.get_or_insert(ctx.transform().rotation.y);
        let want = if near || self.talking > 0.0 { to.x.atan2(to.z).to_degrees() } else { home };
        let t = ctx.transform_mut();
        let diff = (want - t.rotation.y + 540.0).rem_euclid(360.0) - 180.0;
        let step = self.turn_speed * dt;
        t.rotation.y += diff.clamp(-step, step);
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::text("speaker", &mut self.speaker),
            Field::text("work_clip", &mut self.work_clip),
            Field::text("idle_clip", &mut self.idle_clip),
            Field::text("talk_clip", &mut self.talk_clip),
            Field::f32("notice", &mut self.notice),
            Field::f32("turn_speed", &mut self.turn_speed),
        ]
    }
}
