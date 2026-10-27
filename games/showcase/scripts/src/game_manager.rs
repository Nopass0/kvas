//! GameManager script.

use kvas_engine::prelude::*;

script_params! {
    /// Rules, music, sky and screen look of the game.
    pub struct GameManager {
        total: i32 = 8,
        /// Game hours per real second; the sun sets while you play.
        day_speed: f32 = 0.05,
        music: SoundRef = SoundRef("music/island.m4a".into()),
        collected: Local<i32> = Local(0),
        finished: Local<bool> = Local(false),
    }
}

impl Script for GameManager {
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.play_music(&self.music.0, 0.35);
        ctx.set_time_of_day(15.0);
        ctx.set_day_speed(self.day_speed);
        ctx.set_clouds(0.35);
        ctx.post_preset("cinematic");
        ctx.post().motion_blur = 0.25;
        ctx.shadows().softness = 2.0;
        ctx.lock_cursor(false);
        ctx.hud_objective(&format!("Gems: 0 / {}", self.total));
        ctx.hud_message("Collect every gem before the sun sets", 4.0);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let picked = ctx.signals("gem").len() as i32;
        if picked > 0 {
            self.collected.0 += picked;
            ctx.hud_objective(&format!("Gems: {} / {}", self.collected.0, self.total));
        }
        if !self.finished.0 && self.collected.0 >= self.total {
            self.finished.0 = true;
            ctx.play_sound("sounds/win.wav", 0.9);
            ctx.hud_message("You found all the treasure!", 6.0);
            ctx.post().bloom = 0.9;
            ctx.post().lens_flare = 0.6;
            ctx.set_day_speed(0.0);
            ctx.set_time_of_day(18.0);
        }
        if !self.finished.0 && ctx.is_night() {
            ctx.hud_message("Night has fallen... the sea keeps the rest", 5.0);
            ctx.set_day_speed(0.0);
            ctx.set_time_of_day(21.0);
            self.finished.0 = true;
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
