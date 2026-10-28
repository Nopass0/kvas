use kvas_engine::prelude::*;

script_params! {
    /// Plays a looping 2D sound panned to the centre.
    pub struct StereoLoopCentre {
        sound: SoundRef = SoundRef(String::new()),
        volume: f32 = 0.6,
        started: Local<bool> = Local(false),
    }
}

impl Script for StereoLoopCentre {
    fn update(&mut self, ctx: &mut Ctx) {
        if !*self.started && !self.sound.0.is_empty() {
            *self.started = true;
            ctx.play_sound_panned(&self.sound.0, self.volume, 0.0);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
