//! Sound output for the editor and the player: plays what the scene's [`AudioBus`] and its
//! [`AudioSource`] components ask for, with stereo panning and distance falloff.

use glam::Vec3;
use kvas_engine::assets::Assets;
use kvas_engine::audio::{cone_gain, distance_gain, stereo_pan, AudioCmd, AudioSource};
use kvas_engine::prelude::*;
use kvas_engine::scene::Scene;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source, SpatialSink};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
pub mod edit;

struct Voice {
    sink: SpatialSink,
    volume: f32,
    position: Option<Vec3>,
    min: f32,
    max: f32,
    pan: f32,
    /// Looping 2D clip started as music (follows the music volume).
    music: bool,
    /// Direction the source points (world), and its beam angle in degrees (360 = omni).
    forward: Vec3,
    cone: f32,
}

/// Decodes a clip; some container quirks make the decoder panic instead of failing, so that is caught.
fn decode(bytes: Arc<[u8]>) -> Result<Decoder<Cursor<Arc<[u8]>>>, String> {
    std::panic::catch_unwind(|| Decoder::new(Cursor::new(bytes))).map_err(|_| "unsupported container".to_string())?.map_err(|e| e.to_string())
}

/// Owns the output device. Create it once; call [`AudioEngine::update`] every frame while a game runs.
pub struct AudioEngine {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    voices: HashMap<u64, Voice>,
    sources: HashMap<Entity, (u64, String)>,
    cache: HashMap<String, Option<Arc<[u8]>>>,
    next_source_id: u64,
    /// Room echo for new sounds (from the scene's `AudioBus`).
    reverb: (f32, f32),
    /// Problems worth showing in the console (missing clips, undecodable files).
    pub warnings: Vec<String>,
}

impl AudioEngine {
    /// Opens the default output device; `None` when there is no sound card.
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Self { _stream: stream, handle, voices: HashMap::new(), sources: HashMap::new(), cache: HashMap::new(), next_source_id: 1 << 62, reverb: (0.0, 45.0), warnings: Vec::new() })
    }

    fn bytes(&mut self, assets: &Assets, clip: &str) -> Option<Arc<[u8]>> {
        if let Some(hit) = self.cache.get(clip) {
            return hit.clone();
        }
        let data = assets.read(clip).ok().map(|b| Arc::<[u8]>::from(b));
        if data.is_none() {
            self.warnings.push(format!("Sound not found: {clip}"));
        }
        self.cache.insert(clip.to_string(), data.clone());
        data
    }

    #[allow(clippy::too_many_arguments)]
    fn start(&mut self, assets: &Assets, id: u64, clip: &str, volume: f32, pitch: f32, pan: f32, looped: bool, position: Option<Vec3>, min: f32, max: f32) {
        let Some(bytes) = self.bytes(assets, clip) else { return };
        let decoder = match decode(bytes) {
            Ok(d) => d,
            Err(e) => {
                self.warnings.push(format!("Cannot decode {clip}: {e}"));
                self.cache.insert(clip.to_string(), None);
                return;
            }
        };
        let Ok(sink) = SpatialSink::try_new(&self.handle, [pan, 0.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]) else { return };
        let source = decoder.speed(pitch);
        let music = looped && position.is_none();
        let (wet, room) = self.reverb;
        if wet > 0.01 && !music {
            // Room echo: three reflections at prime-ish spacings, each quieter than the last.
            let ms = |k: f32| std::time::Duration::from_millis((room.clamp(5.0, 400.0) * k) as u64);
            let wet = wet.clamp(0.0, 1.0);
            let echo = source.buffered().reverb(ms(1.0), 0.45 * wet).buffered().reverb(ms(1.73), 0.32 * wet).buffered().reverb(ms(2.91), 0.22 * wet);
            if looped {
                sink.append(echo.buffered().repeat_infinite());
            } else {
                sink.append(echo);
            }
        } else if looped {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }
        sink.set_volume(volume);
        self.voices.insert(id, Voice { sink, volume, position, min, max, pan, music, forward: Vec3::NEG_Z, cone: 360.0 });
    }

    /// Stops everything (call when the game stops).
    pub fn stop_all(&mut self) {
        self.voices.clear();
        self.sources.clear();
    }

    /// Handles queued requests and drives [`AudioSource`] components.
    pub fn update(&mut self, scene: &mut Scene, assets: &Assets) {
        self.run(scene, assets, true);
    }

    /// Only handles queued requests (editor previews); components stay silent.
    pub fn update_requests(&mut self, scene: &mut Scene, assets: &Assets) {
        self.run(scene, assets, false);
    }

    fn run(&mut self, scene: &mut Scene, assets: &Assets, components: bool) {
        let (listener, right) = listener(&scene.world);
        self.reverb = (scene.audio.reverb, scene.audio.reverb_room_ms);
        let commands = scene.audio.drain();
        for cmd in commands {
            match cmd {
                AudioCmd::Play { id, clip, volume, pitch, pan, looped, position, min_distance, max_distance } => {
                    self.start(assets, id.0, &clip, volume, pitch, pan, looped, position, min_distance, max_distance);
                }
                AudioCmd::Stop(id) => {
                    self.voices.remove(&id.0);
                }
                AudioCmd::StopAll => self.voices.clear(),
                AudioCmd::SetVolume(id, v) => {
                    if let Some(voice) = self.voices.get_mut(&id.0) {
                        voice.volume = v;
                    }
                }
            }
        }

        // Components: start new ones, restart on trigger, drop the voices of removed ones.
        let entities = if components { scene.world.entities_with::<AudioSource>() } else { Vec::new() };
        self.sources.retain(|e, (id, _)| {
            let keep = entities.contains(e);
            if !keep {
                self.voices.remove(id);
            }
            keep
        });
        for e in entities {
            let matrix = hierarchy::global_matrix(&scene.world, e);
            let position = matrix.w_axis.truncate();
            let forward = matrix.z_axis.truncate();
            let Some(src) = scene.world.get_mut::<AudioSource>(e) else { continue };
            let snapshot = src.clone();
            let trigger = std::mem::take(&mut src.trigger);
            let known = self.sources.get(&e).cloned();
            let changed_clip = known.as_ref().is_some_and(|(_, clip)| *clip != snapshot.clip);
            let finished = known.as_ref().is_some_and(|(id, _)| !self.voices.contains_key(id));
            let should_start = !snapshot.clip.is_empty() && (trigger || changed_clip || (known.is_none() && snapshot.play_on_start) || (finished && snapshot.looping));
            if should_start {
                if let Some((id, _)) = &known {
                    self.voices.remove(id);
                }
                let id = self.next_source_id;
                self.next_source_id += 1;
                let pos = snapshot.spatial.then_some(position);
                self.start(assets, id, &snapshot.clip, snapshot.volume, snapshot.pitch, snapshot.pan, snapshot.looping, pos, snapshot.min_distance, snapshot.max_distance);
                self.sources.insert(e, (id, snapshot.clip.clone()));
            } else if let Some((id, _)) = known {
                if let Some(voice) = self.voices.get_mut(&id) {
                    voice.volume = snapshot.volume;
                    voice.position = snapshot.spatial.then_some(position);
                    voice.min = snapshot.min_distance;
                    voice.max = snapshot.max_distance;
                    voice.pan = snapshot.pan;
                    voice.forward = -forward;
                    voice.cone = snapshot.cone_angle;
                }
            }
        }

        // Apply spatialisation and gain, drop what finished.
        let master = if scene.audio.muted { 0.0 } else { scene.audio.master_volume };
        self.voices.retain(|_, v| !v.sink.empty());
        for v in self.voices.values() {
            let (gain, pan) = match v.position {
                Some(p) => (distance_gain((p - listener).length(), v.min, v.max) * cone_gain(v.forward, listener - p, v.cone), stereo_pan(listener, right, p)),
                None => (1.0, v.pan),
            };
            let bus = if v.music { scene.audio.music_volume } else { scene.audio.effects_volume };
            v.sink.set_volume(v.volume * gain * master * bus.clamp(0.0, 2.0));
            v.sink.set_emitter_position([pan, 0.0, 0.0]);
        }
    }
}

/// The listener is the first camera in the scene: its position and right-hand direction.
fn listener(world: &World) -> (Vec3, Vec3) {
    match kvas_engine::components::active_camera(world) {
        Some(e) => {
            let m = hierarchy::global_matrix(world, e);
            (m.w_axis.truncate(), m.x_axis.truncate().normalize_or(Vec3::X))
        }
        None => (Vec3::ZERO, Vec3::X),
    }
}

/// What the sound panel shows about a clip.
#[derive(Clone, Debug, Default)]
pub struct ClipInfo {
    /// Peak amplitude (0..1) per slice of the clip, for drawing the waveform.
    pub peaks: Vec<f32>,
    /// Length in seconds.
    pub seconds: f32,
    /// 1 = mono, 2 = stereo.
    pub channels: u16,
    /// Samples per second.
    pub sample_rate: u32,
}

/// Decodes a clip and summarises it into `buckets` peaks. Can take a moment for long music files:
/// call it from a worker thread.
pub fn analyze(bytes: Arc<[u8]>, buckets: usize) -> Option<ClipInfo> {
    let decoder = decode(bytes).ok()?;
    let (channels, sample_rate) = (decoder.channels(), decoder.sample_rate());
    let samples: Vec<i16> = decoder.collect();
    if samples.is_empty() || channels == 0 {
        return None;
    }
    let buckets = buckets.max(8);
    let per = samples.len().div_ceil(buckets);
    let peaks = samples.chunks(per).map(|c| c.iter().map(|s| (*s as f32).abs()).fold(0.0, f32::max) / 32768.0).collect();
    Some(ClipInfo { peaks, seconds: samples.len() as f32 / (channels as f32 * sample_rate as f32), channels, sample_rate })
}

/// Reads and analyzes a project clip.
pub fn analyze_asset(assets: &Assets, clip: &str, buckets: usize) -> Option<ClipInfo> {
    let bytes = Arc::<[u8]>::from(assets.read(clip).ok()?);
    analyze(bytes, buckets)
}

