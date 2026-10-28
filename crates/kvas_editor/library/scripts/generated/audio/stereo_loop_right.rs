use kvas_engine::prelude::*;

script_params! {
    /// Plays a looping 2D sound panned to the right.
    pub struct StereoLoopRight {
        sound: SoundRef = SoundRef(String::new()),
        volume: f32 = 0.6,
        started: Local<bool> = Local(false),
    }
}

impl Script for StereoLoopRight {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.started && !self.sound.0.is_empty() {
            *self.started = true;
            ctx.play_sound_panned(&self.sound.0, self.volume, 1.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
