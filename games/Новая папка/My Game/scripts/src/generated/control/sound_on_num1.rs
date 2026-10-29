use kvas_engine::prelude::*;

script_params! {
    /// Plays a sound when Num1 is pressed.
    pub struct SoundOnNum1 {
        sound: SoundRef = SoundRef(String::new()),
        volume: f32 = 1.0,
    }
}

impl Script for SoundOnNum1 {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.key_pressed(Key::Num1) && !self.sound.0.is_empty() {
            let at = ctx.position();
            ctx.play_sound_at(&self.sound.0, self.volume, at);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
