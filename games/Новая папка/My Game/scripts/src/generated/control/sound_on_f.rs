use kvas_engine::prelude::*;

script_params! {
    /// Plays a sound when F is pressed.
    pub struct SoundOnF {
        sound: SoundRef = SoundRef(String::new()),
        volume: f32 = 1.0,
    }
}

impl Script for SoundOnF {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::F) && !self.sound.0.is_empty() {
            let at = ctx.position();
            ctx.play_sound_at(&self.sound.0, self.volume, at);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
