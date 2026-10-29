use kvas_engine::prelude::*;

script_params! {
    /// Starts a music or ambience loop when the game starts.
    pub struct AmbientLoop {
        clip: SoundRef = SoundRef(String::new()),
        volume: f32 = 0.5,
    }
}

impl Script for AmbientLoop {
    fn start(&mut self, ctx: &mut Ctx) {
        if !self.clip.0.is_empty() {
            ctx.play_music(&self.clip.0, self.volume);
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
