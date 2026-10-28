//! Sound: the [`AudioSource`] component and the [`AudioBus`] that scripts talk to.
//!
//! The engine itself plays nothing. Scripts and components only queue requests on the
//! scene's bus; a shell (the editor or the player) hands them to the `kvas_audio` backend
//! once per frame. That keeps the simulation deterministic and testable without a sound card.
//!
//! ```
//! use kvas_engine::audio::AudioBus;
//! let mut bus = AudioBus::default();
//! let id = bus.play("sounds/click.wav", 0.8);
//! bus.stop(id);
//! assert_eq!(bus.drain().len(), 2);
//! ```

use crate::components::Component;
use crate::reflect::{AssetKind, Field};
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Handle of a sound started through the bus; use it to stop or adjust that sound later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SoundId(pub u64);

/// One queued request for the audio backend.
#[derive(Clone, Debug)]
pub enum AudioCmd {
    /// Start a one-shot or looping sound.
    Play {
        /// Handle assigned by the bus.
        id: SoundId,
        /// Asset path of the clip (wav, ogg, mp3 or flac).
        clip: String,
        /// Linear gain, 1 = unchanged.
        volume: f32,
        /// Playback speed / pitch multiplier, 1 = unchanged.
        pitch: f32,
        /// Stereo position for non-spatial sounds: -1 left, 0 centre, +1 right.
        pan: f32,
        /// Repeat until stopped.
        looped: bool,
        /// World position for a 3D sound; `None` plays it in 2D.
        position: Option<Vec3>,
        /// Distance below which a 3D sound is at full volume.
        min_distance: f32,
        /// Distance above which a 3D sound is silent.
        max_distance: f32,
    },
    /// Stop one sound.
    Stop(SoundId),
    /// Stop every sound started through the bus.
    StopAll,
    /// Change the gain of a running sound.
    SetVolume(SoundId, f32),
}

/// Pending audio requests and the master volume. Lives in the scene, is never saved.
#[derive(Clone, Debug)]
pub struct AudioBus {
    /// Gain applied to everything, 0..1.
    pub master_volume: f32,
    /// Silences every sound without losing their state.
    pub muted: bool,
    next: u64,
    commands: Vec<AudioCmd>,
}

impl Default for AudioBus {
    fn default() -> Self {
        Self { master_volume: 1.0, muted: false, next: 1, commands: Vec::new() }
    }
}

impl AudioBus {
    fn push_play(&mut self, clip: &str, volume: f32, pitch: f32, pan: f32, looped: bool, position: Option<Vec3>) -> SoundId {
        let id = SoundId(self.next);
        self.next += 1;
        self.commands.push(AudioCmd::Play {
            id,
            clip: clip.to_string(),
            volume: volume.max(0.0),
            pitch: if pitch.is_finite() && pitch > 0.0 { pitch } else { 1.0 },
            pan: pan.clamp(-1.0, 1.0),
            looped,
            position,
            min_distance: 2.0,
            max_distance: 40.0,
        });
        id
    }

    /// Plays a clip with every option at once (used by the editor's sound panel).
    pub fn play_with(&mut self, clip: &str, volume: f32, pitch: f32, pan: f32, looped: bool) -> SoundId {
        self.push_play(clip, volume, pitch, pan, looped, None)
    }

    /// Plays a clip in 2D at the given volume.
    pub fn play(&mut self, clip: &str, volume: f32) -> SoundId {
        self.push_play(clip, volume, 1.0, 0.0, false, None)
    }

    /// Plays a looping clip in 2D (music, ambience). Stop it with [`Self::stop`].
    pub fn play_loop(&mut self, clip: &str, volume: f32) -> SoundId {
        self.push_play(clip, volume, 1.0, 0.0, true, None)
    }

    /// Plays a clip with a stereo position: `pan` -1 is fully left, +1 fully right.
    pub fn play_panned(&mut self, clip: &str, volume: f32, pan: f32) -> SoundId {
        self.push_play(clip, volume, 1.0, pan, false, None)
    }

    /// Plays a clip with a speed / pitch change (`pitch` 2.0 is an octave higher and twice as fast).
    pub fn play_pitched(&mut self, clip: &str, volume: f32, pitch: f32) -> SoundId {
        self.push_play(clip, volume, pitch, 0.0, false, None)
    }

    /// Plays a clip at a world position: it gets quieter with distance and pans left/right.
    pub fn play_at(&mut self, clip: &str, volume: f32, position: Vec3) -> SoundId {
        self.push_play(clip, volume, 1.0, 0.0, false, Some(position))
    }

    /// Stops one sound.
    pub fn stop(&mut self, id: SoundId) {
        self.commands.push(AudioCmd::Stop(id));
    }

    /// Stops all sounds.
    pub fn stop_all(&mut self) {
        self.commands.push(AudioCmd::StopAll);
    }

    /// Changes the gain of a running sound (fade-outs, ducking).
    pub fn set_volume(&mut self, id: SoundId, volume: f32) {
        self.commands.push(AudioCmd::SetVolume(id, volume.max(0.0)));
    }

    /// Takes the queued requests; the backend calls this once per frame.
    pub fn drain(&mut self) -> Vec<AudioCmd> {
        std::mem::take(&mut self.commands)
    }
}

/// A sound attached to an object: ambience, a radio, a machine hum.
///
/// With `spatial` on, volume follows the distance to the listener (the scene camera) and the
/// sound is panned left/right by where the object is; off, it plays flat with `pan`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSource {
    /// Clip asset path.
    pub clip: String,
    /// Linear gain.
    pub volume: f32,
    /// Speed / pitch multiplier.
    pub pitch: f32,
    /// Repeat the clip.
    pub looping: bool,
    /// Start when the game starts.
    pub play_on_start: bool,
    /// Treat the object position as the sound position.
    pub spatial: bool,
    /// Full-volume radius in metres (spatial only).
    pub min_distance: f32,
    /// Silent beyond this distance in metres (spatial only).
    pub max_distance: f32,
    /// Stereo position when not spatial, -1 left .. +1 right.
    pub pan: f32,
    /// Set to true from a script to (re)start the clip; the backend clears it.
    pub trigger: bool,
    /// Opening angle in degrees of the sound beam along the object's forward (-Z) axis.
    /// 360 = the same in every direction; a narrow cone is a loudspeaker that is quieter behind and to the sides.
    pub cone_angle: f32,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip: String::new(),
            volume: 1.0,
            pitch: 1.0,
            looping: true,
            play_on_start: true,
            spatial: true,
            min_distance: 2.0,
            max_distance: 40.0,
            pan: 0.0,
            trigger: false,
            cone_angle: 360.0,
        }
    }
}

impl Component for AudioSource {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::asset("clip", &mut self.clip, AssetKind::Audio),
            Field::f32("volume", &mut self.volume),
            Field::f32("pitch", &mut self.pitch),
            Field::bool("looping", &mut self.looping),
            Field::bool("play_on_start", &mut self.play_on_start),
            Field::bool("spatial", &mut self.spatial),
            Field::f32("min_distance", &mut self.min_distance),
            Field::f32("max_distance", &mut self.max_distance),
            Field::f32("pan", &mut self.pan),
            Field::f32("cone_angle", &mut self.cone_angle),
            Field::bool("trigger", &mut self.trigger),
        ]
    }
}

/// Gain for a 3D sound: 1 inside `min`, falling linearly to 0 at `max`.
pub fn distance_gain(distance: f32, min: f32, max: f32) -> f32 {
    if distance <= min {
        1.0
    } else if distance >= max || max <= min {
        0.0
    } else {
        let t = (distance - min) / (max - min);
        (1.0 - t) * (1.0 - t)
    }
}

/// Gain of a directional source: full inside the inner 60 % of the cone, fading to 10 % at its edge and beyond.
/// `forward` is where the source points, `to_listener` the direction from the source to the listener.
pub fn cone_gain(forward: Vec3, to_listener: Vec3, cone_degrees: f32) -> f32 {
    if cone_degrees >= 359.0 {
        return 1.0;
    }
    let (f, t) = (forward.normalize_or_zero(), to_listener.normalize_or_zero());
    if f == Vec3::ZERO || t == Vec3::ZERO {
        return 1.0;
    }
    let angle = f.dot(t).clamp(-1.0, 1.0).acos().to_degrees();
    let half = (cone_degrees * 0.5).max(1.0);
    let inner = half * 0.6;
    if angle <= inner {
        1.0
    } else if angle >= half {
        0.1
    } else {
        let k = (angle - inner) / (half - inner);
        1.0 - 0.9 * k * k * (3.0 - 2.0 * k)
    }
}

/// Stereo position (-1 left .. 1 right) of a source seen from a listener.
pub fn stereo_pan(listener: Vec3, listener_right: Vec3, source: Vec3) -> f32 {
    let to = source - listener;
    let d = to.length();
    if d < 1e-3 {
        return 0.0;
    }
    (to.dot(listener_right) / d).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falloff_is_one_inside_and_zero_outside() {
        assert_eq!(distance_gain(1.0, 2.0, 10.0), 1.0);
        assert_eq!(distance_gain(10.0, 2.0, 10.0), 0.0);
        assert!(distance_gain(6.0, 2.0, 10.0) < 0.5);
    }

    #[test]
    fn cone_is_loud_in_front_and_quiet_behind() {
        assert_eq!(cone_gain(Vec3::NEG_Z, Vec3::NEG_Z, 90.0), 1.0);
        assert_eq!(cone_gain(Vec3::NEG_Z, Vec3::Z, 90.0), 0.1);
        assert_eq!(cone_gain(Vec3::NEG_Z, Vec3::Z, 360.0), 1.0);
        let mid = cone_gain(Vec3::NEG_Z, Vec3::new(0.6, 0.0, -0.8), 90.0);
        assert!(mid < 1.0 && mid > 0.1);
    }

    #[test]
    fn pan_follows_side() {
        assert!(stereo_pan(Vec3::ZERO, Vec3::X, Vec3::new(5.0, 0.0, 0.0)) > 0.99);
        assert!(stereo_pan(Vec3::ZERO, Vec3::X, Vec3::new(-5.0, 0.0, 0.0)) < -0.99);
    }
}
