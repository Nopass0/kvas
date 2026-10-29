//! Rust scripting. Scripts are ordinary Rust types compiled into a dynamic library next to the
//! project and hot-reloaded by the host. Host and library must be built by the same toolchain
//! from the same engine sources; `AbiInfo` is checked on load to catch a mismatch early.
//!
//! Note: statics are duplicated across the library boundary, so scripts must talk to the engine
//! only through `Ctx` (that is why logging goes through `Ctx::log`).

use crate::components::*;
use crate::hierarchy;
use crate::input::{Input, Key, MouseButton};
use crate::physics::{Physics, QueryOptions, RayHit};
use crate::reflect::{apply_props, apply_props_checked, read_props, Field};
use crate::scene::Scene;
use glam::Vec3;
use kvas_ecs::{Component as EcsComponent, Entity, World};
use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Simulation time supplied to one script callback.
#[derive(Clone, Copy, Default)]
pub struct Time {
    /// Callback interval in seconds (fixed interval during `fixed_update`).
    pub dt: f32,
    /// Accumulated scaled gameplay time in seconds.
    pub elapsed: f32,
    /// Render/update tick counter, shared by fixed steps of the same frame.
    pub frame: u64,
}

/// Implement this (plus `Default`) and list the type in `register_scripts!`.
/// Public fields are exposed in the editor by returning them from `fields()`.
pub trait Script {
    /// Called once before this instance's first gameplay callback.
    /// A hot reload creates a new instance and calls `start` again.
    fn start(&mut self, _ctx: &mut Ctx) {}
    /// Called once per accepted simulation frame before fixed physics steps.
    fn update(&mut self, _ctx: &mut Ctx) {}
    /// Called before every fixed physics step (60 Hz by default). Apply forces here.
    fn fixed_update(&mut self, _ctx: &mut Ctx) {}
    /// Editable/persistent properties. Missing saved properties retain defaults.
    fn fields(&mut self) -> Vec<Field<'_>> {
        Vec::new()
    }
}

/// Exported script factory used by [`crate::register_scripts!`].
pub struct ScriptDef {
    /// Type name exposed in the editor and serialized in [`ScriptSlot`].
    pub name: &'static str,
    /// Factory valid only while the owning native library remains loaded.
    pub create: fn() -> Box<dyn Script>,
    /// The script overrides `update`; when false it is never called per frame.
    pub update: bool,
    /// The script overrides `fixed_update`; when false it is never called per physics step.
    pub fixed: bool,
}

impl ScriptDef {
    /// Defines a host-linked script factory without a dynamic library.
    /// The factory and script type must live in permanent host code. Export
    /// reloadable script types with [`crate::register_scripts!`] instead.
    pub fn of<T: Script + Default + 'static>(name: &'static str) -> Self {
        Self { name, create: || Box::new(T::default()), update: true, fixed: true }
    }
}

/// Library registration payload; construct through [`crate::register_scripts!`].
pub struct ScriptLib {
    /// Host/library compatibility fingerprint.
    pub abi: AbiInfo,
    /// Ordered exported factories.
    pub scripts: Vec<ScriptDef>,
}

const ABI_VERSION: u32 = 3;

fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

/// Everything that must be identical in the host and in a scripts library for sharing engine
/// memory to be sound: compiler, engine sources, and the layout of the types scripts touch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct AbiInfo {
    version: u32,
    rustc: u64,
    sources: u64,
    layouts: [usize; 8],
}

impl AbiInfo {
    /// Fingerprint of this compiler, engine sources and script-visible layouts.
    /// The C-compatible probe is checked before exchanging Rust-owned containers.
    pub fn current() -> Self {
        use std::mem::{align_of, size_of};
        Self {
            version: ABI_VERSION,
            rustc: fnv(env!("KVAS_RUSTC_VERSION")),
            sources: fnv(&format!("{}|{}", env!("KVAS_SRC_HASH"), env!("KVAS_BUILD_CONFIG"))),
            layouts: [
                size_of::<World>(),
                size_of::<Scene>(),
                size_of::<Ctx>(),
                size_of::<Transform>(),
                size_of::<glam::Mat4>() * 16 + align_of::<glam::Mat4>(),
                size_of::<glam::Quat>() * 16 + align_of::<glam::Quat>(),
                size_of::<RigidBody>(),
                // Keep the C probe's fixed layout unchanged across engine versions.
                // Extra layout checks are folded into the final scalar slot.
                [
                    size_of::<Scripts>(),
                    align_of::<Scripts>(),
                    size_of::<Input>(),
                    align_of::<Input>(),
                    size_of::<serde_json::Value>(),
                    align_of::<serde_json::Value>(),
                    size_of::<crate::reflect::FieldRef<'static>>(),
                    size_of::<crate::mesh::MeshData>(),
                    align_of::<crate::mesh::MeshData>(),
                    size_of::<QueryOptions>(),
                ]
                .into_iter()
                .fold(0xcbf2_9ce4_8422_2325u64, |hash, value| (hash ^ value as u64).wrapping_mul(0x0100_0000_01b3))
                    as usize,
            ],
        }
    }
}

/// Last segment of a type path (`a::b::Spin` becomes `Spin`); used by [`register_scripts!`].
pub fn short_name(path: &'static str) -> &'static str {
    path.rsplit("::").next().unwrap_or(path).trim()
}

/// Exports the scripts of a library crate: `register_scripts!(Spinner, Player);`
#[macro_export]
macro_rules! register_scripts {
    // Generated by the project's build.rs: `Type => has_update, has_fixed_update; ...`.
    (@flags $($t:ty => $u:expr, $f:expr);* $(;)?) => {
        /// C-compatible compatibility probe checked before loading Rust registrations.
        #[no_mangle]
        pub extern "C" fn kvas_script_abi() -> $crate::script::AbiInfo {
            $crate::script::AbiInfo::current()
        }
        /// Native script registration entry point. Loaded by `ScriptHost` after ABI validation.
        #[no_mangle]
        pub fn kvas_script_lib() -> $crate::script::ScriptLib {
            $crate::script::ScriptLib {
                abi: $crate::script::AbiInfo::current(),
                scripts: vec![$($crate::script::ScriptDef {
                    name: $crate::script::short_name(stringify!($t)),
                    create: || Box::new(<$t as Default>::default()),
                    update: $u,
                    fixed: $f,
                }),*],
            }
        }
    };
    ($($t:ty),* $(,)?) => {
        /// C-compatible compatibility probe checked before loading Rust registrations.
        #[no_mangle]
        pub extern "C" fn kvas_script_abi() -> $crate::script::AbiInfo {
            $crate::script::AbiInfo::current()
        }
        /// Native script registration entry point. Loaded by `ScriptHost` after ABI validation.
        #[no_mangle]
        pub fn kvas_script_lib() -> $crate::script::ScriptLib {
            $crate::script::ScriptLib {
                abi: $crate::script::AbiInfo::current(),
                scripts: vec![$($crate::script::ScriptDef {
                    name: $crate::script::short_name(stringify!($t)),
                    create: || Box::new(<$t as Default>::default()),
                    update: true,
                    fixed: true,
                }),*],
            }
        }
    };
}

/// What a script sees each call.
pub struct Ctx<'a> {
    /// Entity owning this script slot; it may become stale if a script despawns it.
    pub entity: Entity,
    /// Mutable host scene. Register custom storages only in permanent host code.
    pub scene: &'a mut Scene,
    /// Frame snapshot and shared shell requests.
    pub input: &'a Input,
    /// Scaled callback time.
    pub time: Time,
    physics: &'a Physics,
    // Host function: rapier code must run in the host, not in a script library.
    raycast_fn: fn(&Physics, Vec3, Vec3, f32, QueryOptions) -> Option<RayHit>,
    overlap_fn: fn(&Physics, Vec3, f32) -> Vec<Entity>,
    touching_fn: fn(&Physics, Entity) -> Vec<Entity>,
    overlap_filtered_fn: fn(&Physics, Vec3, f32, QueryOptions) -> Vec<Entity>,
    log: &'a mut Vec<String>,
}

impl Ctx<'_> {
    /// Callback delta in seconds; fixed callbacks receive the configured fixed interval.
    pub fn dt(&self) -> f32 {
        self.time.dt
    }
    /// Scaled gameplay seconds since runtime creation.
    pub fn elapsed(&self) -> f32 {
        self.time.elapsed
    }
    /// Frames simulated so far (a counter, handy for "every N frames").
    pub fn frame(&self) -> u64 {
        self.time.frame
    }
    /// Frames per second implied by this callback's `dt` (0 when `dt` is 0).
    pub fn fps(&self) -> f32 {
        if self.time.dt > 0.0 { 1.0 / self.time.dt } else { 0.0 }
    }
    /// True once every `seconds` of game time (use inside `update`).
    pub fn every(&self, seconds: f32) -> bool {
        seconds > 0.0 && (self.time.elapsed % seconds) < self.time.dt
    }

    // ---- sky and time of day

    /// Current in-game hour, 0..24.
    pub fn time_of_day(&self) -> f32 {
        self.scene.settings.atmosphere.time
    }
    /// Jumps to an hour of the day (wraps into 0..24) and switches the procedural sky on.
    pub fn set_time_of_day(&mut self, hour: f32) {
        let sky = &mut self.scene.settings.atmosphere;
        sky.procedural = true;
        sky.time = hour.rem_euclid(24.0);
    }
    /// Game hours that pass per real second (0 freezes the sun).
    pub fn set_day_speed(&mut self, hours_per_second: f32) {
        self.scene.settings.atmosphere.speed = hours_per_second.max(0.0);
    }
    /// The whole sky configuration for fine control: clouds, stars, haze, aurora, moon.
    pub fn sky(&mut self) -> &mut crate::sky::SkySettings {
        &mut self.scene.settings.atmosphere
    }
    /// 0 = clear sky, 1 = overcast.
    pub fn set_clouds(&mut self, amount: f32) {
        self.scene.settings.atmosphere.clouds = amount.clamp(0.0, 1.0);
    }
    /// Whether it is night (sun below the horizon, roughly 20:00-05:00).
    pub fn is_night(&self) -> bool {
        let h = self.scene.settings.atmosphere.time;
        !(5.0..20.0).contains(&h)
    }

    // ---- look: post effects and shadows

    /// Screen effects for fine control: `ctx.post().bloom = 0.8; ctx.post().motion_blur = 0.5;`.
    pub fn post(&mut self) -> &mut crate::fx::PostSettings {
        &mut self.scene.settings.post
    }
    /// Sun shadow settings: `ctx.shadows().softness = 2.0;`.
    pub fn shadows(&mut self) -> &mut crate::fx::ShadowSettings {
        &mut self.scene.settings.shadows
    }
    /// Applies a ready-made look: "cinematic", "dreamy", "crisp", "retro" or anything else for none.
    pub fn post_preset(&mut self, name: &str) {
        self.scene.settings.post = crate::fx::PostSettings::preset(name);
    }
    /// Exposure multiplier (brightness) of the picture.
    pub fn set_exposure(&mut self, exposure: f32) {
        self.scene.settings.exposure = exposure.clamp(0.05, 8.0);
    }

    /// Adds a custom full-screen shader to the camera (a WGSL file with `fn effect(..)`) and returns its index.
    pub fn add_post_effect(&mut self, shader: &str) -> usize {
        let list = &mut self.scene.settings.post.effects;
        list.push(crate::fx::PostEffect { shader: shader.to_string(), ..Default::default() });
        list.len() - 1
    }
    /// Sets the four numbers a custom effect receives as `params`.
    pub fn set_post_effect_params(&mut self, index: usize, params: [f32; 4]) {
        if let Some(effect) = self.scene.settings.post.effects.get_mut(index) {
            effect.params = params;
        }
    }
    /// Switches a custom effect on or off.
    pub fn enable_post_effect(&mut self, index: usize, on: bool) {
        if let Some(effect) = self.scene.settings.post.effects.get_mut(index) {
            effect.enabled = on;
        }
    }
    /// Removes every custom camera effect.
    pub fn clear_post_effects(&mut self) {
        self.scene.settings.post.effects.clear();
    }

    // ---- audio

    /// Plays a clip (path inside `assets/`) in 2D, e.g. `ctx.play_sound("sounds/click.wav", 1.0)`.
    pub fn play_sound(&mut self, clip: &str, volume: f32) -> crate::audio::SoundId {
        self.scene.audio.play(clip, volume)
    }
    /// Plays a looping 2D clip (music). Keep the id to stop it with [`Self::stop_sound`].
    pub fn play_music(&mut self, clip: &str, volume: f32) -> crate::audio::SoundId {
        self.scene.audio.play_loop(clip, volume)
    }
    /// Plays a clip at a world position (distance falloff and left/right stereo).
    pub fn play_sound_at(&mut self, clip: &str, volume: f32, position: Vec3) -> crate::audio::SoundId {
        self.scene.audio.play_at(clip, volume, position)
    }
    /// Plays a clip with stereo position `pan` (-1 left .. 1 right).
    pub fn play_sound_panned(&mut self, clip: &str, volume: f32, pan: f32) -> crate::audio::SoundId {
        self.scene.audio.play_panned(clip, volume, pan)
    }
    /// Plays a clip with a pitch / speed multiplier.
    pub fn play_sound_pitched(&mut self, clip: &str, volume: f32, pitch: f32) -> crate::audio::SoundId {
        self.scene.audio.play_pitched(clip, volume, pitch)
    }
    /// Stops a sound started earlier.
    pub fn stop_sound(&mut self, id: crate::audio::SoundId) {
        self.scene.audio.stop(id);
    }
    /// Changes the volume of a running sound.
    pub fn set_sound_volume(&mut self, id: crate::audio::SoundId, volume: f32) {
        self.scene.audio.set_volume(id, volume);
    }
    /// Stops every sound.
    pub fn stop_all_sounds(&mut self) {
        self.scene.audio.stop_all();
    }
    /// Master volume for the whole game, 0..1.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.scene.audio.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Mutable host world; its built-in component storages already exist.
    pub fn world(&mut self) -> &mut World {
        &mut self.scene.world
    }
    /// Appends a message to the host log shown in the editor/player.
    pub fn log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
    }

    /// Borrows this entity's component; returns `None` when missing or despawned.
    pub fn get<T: EcsComponent>(&self) -> Option<&T> {
        self.scene.world.get::<T>(self.entity)
    }
    /// Mutably borrows this entity's component, if present.
    pub fn get_mut<T: EcsComponent>(&mut self) -> Option<&mut T> {
        self.scene.world.get_mut::<T>(self.entity)
    }
    /// Borrows another living entity's component, if present.
    pub fn get_of<T: EcsComponent>(&self, e: Entity) -> Option<&T> {
        self.scene.world.get::<T>(e)
    }
    /// Mutably borrows another living entity's component, if present.
    pub fn get_mut_of<T: EcsComponent>(&mut self, e: Entity) -> Option<&mut T> {
        self.scene.world.get_mut::<T>(e)
    }

    /// This entity's transform (panics only if a script removed it; the editor does not allow that).
    pub fn transform(&self) -> &Transform {
        self.get::<Transform>().expect("entity has no Transform")
    }
    /// Mutable local pose.
    /// # Panics
    /// Panics when this entity has no [`Transform`], including after self-despawn.
    pub fn transform_mut(&mut self) -> &mut Transform {
        self.get_mut::<Transform>().expect("entity has no Transform")
    }
    /// Current world position computed from the hierarchy, including pending local edits.
    pub fn world_position(&self) -> Vec3 {
        hierarchy::global_matrix(&self.scene.world, self.entity).w_axis.truncate()
    }

    /// Sets the material colour of an entity (linear-ish sRGB, 0..1).
    pub fn set_color(&mut self, e: Entity, rgba: [f32; 4]) {
        if let Some(m) = self.scene.world.get_mut::<MeshRenderer>(e) {
            m.color = rgba;
        }
    }
    /// Sets a texture by asset path or by a procedural key (`NoiseTexture::key()`).
    pub fn set_texture(&mut self, e: Entity, texture: &str) {
        if let Some(m) = self.scene.world.get_mut::<MeshRenderer>(e) {
            m.texture = texture.to_string();
        }
    }
    /// Replaces the mesh with a parametric shape (the mesh is rebuilt when parameters change).
    pub fn set_shape(&mut self, e: Entity, shape: Shape) {
        self.scene.world.insert(e, shape);
        if !self.scene.world.has::<MeshRenderer>(e) {
            self.scene.world.insert(e, MeshRenderer::default());
        }
    }
    /// Direct non-transient children in entity index order.
    pub fn children(&self, e: Entity) -> Vec<Entity> {
        hierarchy::children(&self.scene.world, e)
    }
    /// First exact entity-name match; names are not unique identifiers.
    pub fn find(&self, name: &str) -> Option<Entity> {
        hierarchy::find_by_name(&self.scene.world, name)
    }
    /// Every entity with this exact name.
    pub fn find_all(&self, name: &str) -> Vec<Entity> {
        self.scene.world.entities().into_iter().filter(|&e| hierarchy::name_of(&self.scene.world, e) == name).collect()
    }
    /// Entities that have component `T`.
    pub fn find_with<T: EcsComponent>(&self) -> Vec<Entity> {
        self.scene.world.iter::<T>().map(|(e, _)| e).collect()
    }
    /// Reads a public variable (a field listed in the script's `fields()`) of a script on any entity,
    /// e.g. `ctx.var(door, "DoorLogic", "locked")`. Returns JSON: numbers, bools, strings, arrays.
    pub fn var(&self, entity: Entity, script: &str, name: &str) -> Option<serde_json::Value> {
        self.scene.world.get::<Scripts>(entity)?.0.iter().find(|s| s.name == script)?.props.get(name).cloned()
    }
    /// Changes a public variable of a script on any entity; the script sees it before its next callback.
    /// Returns false when the entity has no such script.
    pub fn set_var(&mut self, entity: Entity, script: &str, name: &str, value: serde_json::Value) -> bool {
        match self.scene.world.get_mut::<Scripts>(entity).and_then(|s| s.0.iter_mut().find(|s| s.name == script)) {
            Some(slot) => {
                slot.props.insert(name.to_string(), value);
                true
            }
            None => false,
        }
    }
    /// Number variable of another script, see [`Ctx::var`].
    pub fn var_f32(&self, entity: Entity, script: &str, name: &str) -> Option<f32> {
        self.var(entity, script, name)?.as_f64().map(|v| v as f32)
    }
    /// Boolean variable of another script, see [`Ctx::var`].
    pub fn var_bool(&self, entity: Entity, script: &str, name: &str) -> Option<bool> {
        self.var(entity, script, name)?.as_bool()
    }

    /// Creates a named entity with an identity local transform.
    pub fn spawn(&mut self, name: &str) -> Entity {
        hierarchy::spawn_named(&mut self.scene.world, name)
    }
    /// Creates a visible primitive at a world-space position with no collider/body.
    /// Supported primitive names are listed in [`crate::mesh::PRIMITIVES`].
    pub fn spawn_primitive(&mut self, name: &str, primitive: &str, position: Vec3) -> Entity {
        let e = self.spawn(name);
        self.scene.world.insert(e, Transform::at(position));
        self.scene.world.insert(e, MeshRenderer { mesh: format!("primitive:{primitive}"), ..Default::default() });
        e
    }
    /// Destroys an entity and its subtree. Physics objects are removed on the next fixed step.
    pub fn despawn(&mut self, e: Entity) {
        hierarchy::despawn_recursive(&mut self.scene.world, e);
    }

    /// Accumulates a world-space impulse for this body to consume on the next physics step.
    /// Does nothing when no [`RigidBody`] is attached.
    pub fn add_impulse(&mut self, impulse: Vec3) {
        if let Some(rb) = self.get_mut::<RigidBody>() {
            rb.impulse += impulse;
        }
    }
    /// Requests a world-space linear velocity in metres/second for the next fixed step.
    pub fn set_velocity(&mut self, v: Vec3) {
        if let Some(rb) = self.get_mut::<RigidBody>() {
            rb.set_velocity = Some(v);
        }
    }
    /// Sets the spin of a rigid body (any entity); `Vec3::ZERO` stops it rotating.
    pub fn set_angular_velocity_of(&mut self, e: Entity, w: Vec3) {
        if let Some(rb) = self.get_mut_of::<RigidBody>(e) {
            rb.set_angular_velocity = Some(w);
        }
    }
    /// Sets the linear velocity of any rigid body.
    pub fn set_velocity_of(&mut self, e: Entity, v: Vec3) {
        if let Some(rb) = self.get_mut_of::<RigidBody>(e) {
            rb.set_velocity = Some(v);
        }
    }
    /// Last simulated linear velocity, or zero when no body is attached.
    pub fn velocity(&self) -> Vec3 {
        self.get::<RigidBody>().map_or(Vec3::ZERO, |rb| rb.velocity)
    }
    /// First solid collider hit by the ray; this entity's own collider and triggers are ignored.
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_distance: f32) -> Option<RayHit> {
        (self.raycast_fn)(self.physics, origin, dir, max_distance, QueryOptions { exclude: Some(self.entity), ..Default::default() })
    }

    /// Ray query with layer/sensor filters; defaults to excluding this script's body.
    /// An explicit `options.exclude` overrides that default owner exclusion.
    pub fn raycast_filtered(&self, origin: Vec3, direction: Vec3, max_distance: f32, mut options: QueryOptions) -> Option<RayHit> {
        options.exclude.get_or_insert(self.entity);
        (self.raycast_fn)(self.physics, origin, direction, max_distance, options)
    }

    /// Sphere query under explicit layer/sensor/exclusion filters. No implicit owner exclusion.
    pub fn overlap_sphere_filtered(&self, center: Vec3, radius: f32, options: QueryOptions) -> Vec<Entity> {
        (self.overlap_filtered_fn)(self.physics, center, radius, options)
    }

    /// Collider entities overlapping a world-space sphere in the last physics state.
    /// Includes triggers and this entity; results are sorted and deduplicated.
    pub fn overlap_sphere(&self, center: Vec3, radius: f32) -> Vec<Entity> {
        (self.overlap_fn)(self.physics, center, radius)
    }

    /// Active solid contacts and sensor overlaps for a collider entity.
    pub fn touching(&self, entity: Entity) -> Vec<Entity> {
        (self.touching_fn)(self.physics, entity)
    }

    /// Whether an entity handle still refers to a living object in this scene.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.scene.world.is_alive(entity)
    }

    /// Current world matrix, evaluated directly from local transforms.
    pub fn world_matrix_of(&self, entity: Entity) -> glam::Mat4 {
        hierarchy::global_matrix(&self.scene.world, entity)
    }

    /// Reparents while preserving world pose when representable as TRS.
    pub fn set_parent(&mut self, entity: Entity, parent: Option<Entity>) -> bool {
        hierarchy::set_parent(&mut self.scene.world, entity, parent)
    }

    /// Reparents while preserving local pose; rejects stale handles and cycles.
    pub fn set_parent_local(&mut self, entity: Entity, parent: Option<Entity>) -> bool {
        hierarchy::set_parent_local(&mut self.scene.world, entity, parent)
    }

    /// Adds a component of a host-registered type to a living entity.
    /// # Panics
    /// Panics for unregistered or conflicting component types; scripts use built-ins only.
    pub fn insert<T: EcsComponent>(&mut self, entity: Entity, component: T) {
        self.scene.world.insert(entity, component);
    }

    /// Removes a component, returning its former value if present.
    pub fn remove<T: EcsComponent>(&mut self, entity: Entity) -> Option<T> {
        self.scene.world.remove::<T>(entity)
    }

    /// Whether this entity currently has a component.
    pub fn has<T: EcsComponent>(&self) -> bool {
        self.scene.world.has::<T>(self.entity)
    }

    /// Adds a named script slot. The runtime creates its instance at the next sync.
    /// Returns false for a stale entity; duplicate names in different slots are allowed.
    pub fn attach_script(&mut self, entity: Entity, name: &str) -> bool {
        let Some(scripts) = self.scene.world.get_or_insert_with::<Scripts>(entity, Scripts::default) else {
            return false;
        };
        scripts.0.push(ScriptSlot { name: name.to_owned(), ..Default::default() });
        true
    }

    /// Removes every slot with this script name. Returns the number removed.
    pub fn detach_script(&mut self, entity: Entity, name: &str) -> usize {
        let Some(scripts) = self.scene.world.get_mut::<Scripts>(entity) else {
            return 0;
        };
        let before = scripts.0.len();
        scripts.0.retain(|s| s.name != name);
        before - scripts.0.len()
    }

    /// Snapshot of all living entities with an exact tag, in index order.
    pub fn tagged(&self, tag: &str) -> Vec<Entity> {
        crate::gameplay::tagged(&self.scene.world, tag)
    }

    /// Whether an entity contains this exact group label.
    pub fn has_tag(&self, entity: Entity, tag: &str) -> bool {
        self.get_of::<Tags>(entity).is_some_and(|tags| tags.contains(tag))
    }

    /// Adds a group label, creating Tags if needed. Invalid/duplicate labels or stale entities return false.
    pub fn add_tag(&mut self, entity: Entity, tag: &str) -> bool {
        if tag.is_empty() || tag.chars().any(char::is_whitespace) {
            return false;
        }
        self.scene.world.get_or_insert_with::<Tags>(entity, Tags::default).is_some_and(|tags| tags.insert(tag))
    }

    /// Removes a label without removing other tags; returns whether it existed.
    pub fn remove_tag(&mut self, entity: Entity, tag: &str) -> bool {
        self.get_mut_of::<Tags>(entity).is_some_and(|tags| tags.remove(tag))
    }

    /// Schedules/replaces subtree expiration after non-negative simulation seconds.
    /// Zero expires after update callbacks in the current/next accepted frame.
    /// Invalid durations or stale entities return false without changing components.
    pub fn despawn_after(&mut self, entity: Entity, seconds: f32) -> bool {
        let Some(lifetime) = Lifetime::new(seconds) else {
            return false;
        };
        if !self.is_alive(entity) {
            return false;
        }
        self.scene.world.insert(entity, lifetime);
        true
    }

    /// Sets inherited mesh visibility without changing scripts, lights or physics.
    /// Returns false for a stale entity.
    pub fn set_visible(&mut self, entity: Entity, visible: bool) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        self.scene.world.insert(entity, Visibility { visible });
        true
    }

    /// Effective mesh visibility, including hidden ancestors.
    pub fn is_visible(&self, entity: Entity) -> bool {
        hierarchy::is_visible(&self.scene.world, entity)
    }

    /// Starts/replaces a parent-local position tween from the current pose.
    /// Returns false for invalid targets/durations, stale entities or missing transforms.
    pub fn move_to(&mut self, entity: Entity, target: Vec3, seconds: f32, ease: crate::tween::Ease) -> bool {
        let Some(mut tween) = crate::tween::MoveTween::to(target, seconds) else {
            return false;
        };
        if !self.scene.world.has::<Transform>(entity) {
            return false;
        }
        tween.ease = ease;
        self.scene.world.insert(entity, tween);
        true
    }

    /// Moves an entity to a world-space point and queues physical teleportation.
    /// With `reset_velocity`, pending impulses and both velocities are cleared.
    /// For a dynamic body the final pose is integrated on the next fixed step;
    /// non-physical entities move immediately. Invalid/singular requests do nothing.
    pub fn teleport_of(&mut self, entity: Entity, position: Vec3, reset_velocity: bool) -> bool {
        if !hierarchy::set_world_position(&mut self.scene.world, entity, position) {
            return false;
        }
        if let Some(body) = self.get_mut_of::<RigidBody>(entity) {
            body.teleport = true;
            if reset_velocity {
                body.set_velocity = Some(Vec3::ZERO);
                body.set_angular_velocity = Some(Vec3::ZERO);
                body.impulse = Vec3::ZERO;
            }
        }
        true
    }

    /// Teleports this script's entity, optionally resetting its motion.
    pub fn teleport(&mut self, position: Vec3, reset_velocity: bool) -> bool {
        self.teleport_of(self.entity, position, reset_velocity)
    }

    /// Broadcasts JSON data on a channel for delivery before the next frame's callbacks.
    /// Sending is bounded and returns an explicit error rather than silently dropping data.
    pub fn emit(&mut self, channel: &str, payload: serde_json::Value) -> Result<u64, crate::signals::SignalError> {
        self.scene.signals.send(channel, payload, Some(self.entity), None)
    }

    /// Sends a directed message to a living entity for the next simulation frame.
    pub fn send_to(&mut self, target: Entity, channel: &str, payload: serde_json::Value) -> Result<u64, crate::signals::SignalError> {
        if !self.is_alive(target) {
            return Err(crate::signals::SignalError::TargetNotAlive);
        }
        self.scene.signals.send(channel, payload, Some(self.entity), Some(target))
    }

    /// Clones the current channel messages addressed to this entity or broadcast.
    /// Owned snapshots let callbacks mutate the scene while handling messages.
    /// Reading does not consume signals; use update for one-shot processing.
    pub fn signals(&self, channel: &str) -> Vec<crate::signals::Signal> {
        self.scene.signals.for_entity(self.entity).filter(|signal| signal.channel == channel).cloned().collect()
    }

    /// Whether the physical key is held this frame.
    pub fn key_down(&self, k: Key) -> bool {
        self.input.key_down(k)
    }
    /// Whether the physical key went down in this frame snapshot.
    pub fn key_pressed(&self, k: Key) -> bool {
        self.input.key_pressed(k)
    }
    /// Whether the physical key went up in this frame snapshot.
    pub fn key_released(&self, key: Key) -> bool {
        self.input.key_released(key)
    }

    /// Whether a physical pointer button is held.
    pub fn mouse_down(&self, b: MouseButton) -> bool {
        self.input.mouse_down(b)
    }

    /// Whether a physical pointer button went down this frame.
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.input.mouse_pressed(button)
    }
    /// Whether a physical pointer button went up this frame.
    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.input.mouse_released(button)
    }

    /// Hides the cursor and keeps it centred while true; `input.mouse_delta` then is the look movement.
    pub fn lock_cursor(&self, lock: bool) {
        self.input.lock_cursor(lock);
    }
    /// Switches to another scene once this frame ends, e.g. `ctx.load_scene("scenes/level2.scene.json")`.
    /// Objects and scripts of the current scene go away; the new scene starts with fresh scripts.
    pub fn load_scene(&self, path: &str) {
        self.input.request_scene(crate::input::SceneRequest::Load(path.to_string()));
    }
    /// Adds every object of another scene to this one once this frame ends (a level piece, a shared
    /// menu...). Its scripts start on the next frame; the scene settings stay as they are.
    pub fn add_scene(&self, path: &str) {
        self.input.request_scene(crate::input::SceneRequest::Add(path.to_string()));
    }
    /// Starts the running scene again from its file once this frame ends (restart a level).
    pub fn reload_scene(&self) {
        self.input.request_scene(crate::input::SceneRequest::Reload);
    }
    /// Asset path of the running scene, e.g. `scenes/main.scene.json` (empty if unknown).
    pub fn scene_path(&self) -> String {
        self.input.scene_path()
    }
    /// Shows the interaction hint on screen for this frame ("Press E to open").
    pub fn hud_prompt(&self, text: &str) {
        self.input.hud().prompt = text.to_string();
    }
    /// Displays a transient message for finite non-negative simulation seconds.
    pub fn hud_message(&self, text: &str, seconds: f32) {
        let mut h = self.input.hud();
        h.message = text.to_string();
        h.message_time = if seconds.is_finite() { seconds.max(0.0) } else { 0.0 };
    }
    /// Replaces persistent HUD objective text; empty clears it.
    pub fn hud_objective(&self, text: &str) {
        self.input.hud().objective = text.to_string();
    }
    /// Requests that the hosting shell show/hide the crosshair.
    pub fn hud_crosshair(&self, on: bool) {
        self.input.hud().crosshair = on;
    }

    /// Named input actions from Project Settings > Input.
    pub fn action_down(&self, name: &str) -> bool {
        self.input.action_down(name)
    }
    /// Whether any bound input went down this frame; missing actions return false.
    pub fn action_pressed(&self, name: &str) -> bool {
        self.input.action_pressed(name)
    }
    /// Whether the last held input bound to this action was released.
    pub fn action_released(&self, name: &str) -> bool {
        self.input.action_released(name)
    }
    /// Unit-length two-dimensional movement from four named actions.
    pub fn action_vector(&self, left: &str, right: &str, down: &str, up: &str) -> glam::Vec2 {
        self.input.action_vector(left, right, down, up)
    }
    /// Digital axis: positive minus negative action, in `[-1, 1]`.
    pub fn action_axis(&self, negative: &str, positive: &str) -> f32 {
        self.input.action_axis(negative, positive)
    }
    /// Changes an action's bindings while the game runs, e.g. `ctx.rebind("Jump", &["Space", "W"])`.
    pub fn rebind(&self, name: &str, inputs: &[&str]) {
        self.input.rebind(name, inputs);
    }
}

/// Loaded native script factories. Instances created by this host keep the library
/// loaded even if the host is dropped, so their callback vtables remain valid.
pub struct ScriptHost {
    defs: Vec<ScriptDef>,
    library: Option<Arc<LoadedLibrary>>,
}

struct LoadedLibrary {
    library: Option<libloading::Library>,
    copy: PathBuf,
}

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        // Windows releases the file lock only after unloading the module.
        self.library.take();
        let _ = std::fs::remove_file(&self.copy);
    }
}

struct RetainedScript {
    // Field order matters: script destructor runs while library code is resident.
    script: Box<dyn Script>,
    _library: Option<Arc<LoadedLibrary>>,
}

impl Script for RetainedScript {
    fn start(&mut self, ctx: &mut Ctx) {
        self.script.start(ctx);
    }
    fn update(&mut self, ctx: &mut Ctx) {
        self.script.update(ctx);
    }
    fn fixed_update(&mut self, ctx: &mut Ctx) {
        self.script.fixed_update(ctx);
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.script.fields()
    }
}

static LOAD_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl ScriptHost {
    /// Creates a host for permanently linked Rust scripts, useful for embedding
    /// and headless tests. Rejects empty or duplicate script names.
    /// Factories must belong to permanent host code, not an unloadable library.
    ///
    /// ```
    /// use kvas_engine::script::{Script, ScriptDef, ScriptHost};
    /// #[derive(Default)] struct Behaviour;
    /// impl Script for Behaviour {}
    /// let host = ScriptHost::from_defs(vec![ScriptDef::of::<Behaviour>("Behaviour")]).unwrap();
    /// assert!(host.create("Behaviour").is_some());
    /// ```
    pub fn from_defs(defs: Vec<ScriptDef>) -> Result<Self, String> {
        validate_defs(&defs)?;
        Ok(Self { defs, library: None })
    }

    /// Loads a library. It is copied first so the original stays writable for the next build.
    pub fn load(path: &Path) -> Result<ScriptHost, String> {
        let ext = path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        let copy = std::env::temp_dir().join(format!(
            "kvas_scripts_{}_{}.{ext}",
            std::process::id(),
            LOAD_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::copy(path, &copy).map_err(|e| format!("{}: {e}", path.display()))?;
        // SAFETY: loading arbitrary native code is inherently trusted; the project's own scripts are.
        let lib = match unsafe { libloading::Library::new(&copy) } {
            Ok(lib) => lib,
            Err(e) => {
                let _ = std::fs::remove_file(&copy);
                return Err(format!("cannot load scripts: {e}"));
            }
        };
        let library = Arc::new(LoadedLibrary { library: Some(lib), copy });
        let lib = library.library.as_ref().expect("loaded library");
        let probe: libloading::Symbol<extern "C" fn() -> AbiInfo> = unsafe { lib.get(b"kvas_script_abi") }
            .map_err(|_| "scripts library has no ABI probe; rebuild it with the current engine".to_string())?;
        if probe() != AbiInfo::current() {
            return Err("scripts were built against a different engine/compiler build; rebuild them".into());
        }
        let entry: libloading::Symbol<fn() -> ScriptLib> =
            unsafe { lib.get(b"kvas_script_lib") }.map_err(|_| "scripts library does not call register_scripts!".to_string())?;
        // Check the ABI before running any code that touches engine types.
        let lib_fn: fn() -> ScriptLib = *entry;
        let script_lib = lib_fn();
        if script_lib.abi != AbiInfo::current() {
            return Err("scripts were built against a different engine/compiler build; rebuild them".into());
        }
        validate_defs(&script_lib.scripts)?;
        Ok(ScriptHost { defs: script_lib.scripts, library: Some(library) })
    }

    /// Exported type names borrowed for the host lifetime (not falsely `'static`).
    /// Use [`Self::names_owned`] to retain names across hot reload or host drop.
    pub fn names(&self) -> Vec<&str> {
        self.defs.iter().map(|d| d.name).collect()
    }

    /// Copies exported names so they can be retained after the library is unloaded.
    pub fn names_owned(&self) -> Vec<String> {
        self.names().into_iter().map(str::to_owned).collect()
    }

    /// Whether a script has `update` / `fixed_update` (unknown names: both true).
    pub fn callbacks(&self, name: &str) -> (bool, bool) {
        self.defs.iter().find(|d| d.name == name).map_or((true, true), |d| (d.update, d.fixed))
    }

    /// Constructs a script by exact type name and retains the library for its lifetime.
    /// Returns `None` for unknown names. User factory panics propagate to the caller;
    /// [`ScriptRuntime::sync`] catches them when constructing gameplay instances.
    pub fn create(&self, name: &str) -> Option<Box<dyn Script>> {
        self.defs
            .iter()
            .find(|d| d.name == name)
            .map(|d| Box::new(RetainedScript { script: (d.create)(), _library: self.library.clone() }) as Box<dyn Script>)
    }
}

/// Gameplay callback phase dispatched by [`ScriptRuntime`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Per-frame callback.
    Update,
    /// Before a fixed physics step.
    Fixed,
}

struct Instance {
    entity: Entity,
    slot: usize,
    name: String,
    script: Box<dyn Script>,
    started: bool,
    failed: bool,
    /// Which per-frame callbacks the script has; the others are skipped entirely.
    update: bool,
    fixed: bool,
    /// Property values last exchanged with the `Scripts` component (so outside edits are noticed).
    synced: serde_json::Map<String, serde_json::Value>,
}

/// Native script instance lifecycle and per-instance panic isolation.
/// A panic disables only the affected instance until it is recreated or reloaded.
#[derive(Default)]
pub struct ScriptRuntime {
    // Instances must be dropped before the host (their vtables live in the library).
    instances: Vec<Instance>,
    host: Option<ScriptHost>,
    failed_creations: HashSet<(Entity, usize, String)>,
}

fn panic_text(p: Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| p.downcast_ref::<String>().cloned()).unwrap_or_else(|| "panic".into())
}

impl ScriptRuntime {
    /// Current native host, if a scripts library is loaded.
    pub fn host(&self) -> Option<&ScriptHost> {
        self.host.as_ref()
    }

    /// Number of instantiated attached scripts, including disabled instances.
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Disabled callback instances plus slots whose initialization failed.
    pub fn failed_count(&self) -> usize {
        self.instances.iter().filter(|i| i.failed).count() + self.failed_creations.len()
    }

    /// Drops all instances and failure records while retaining the host.
    /// The next sync reconstructs scripts from serialized slot properties.
    pub fn restart_instances(&mut self) {
        self.instances.clear();
        self.failed_creations.clear();
    }

    /// Replaces the library (hot reload). All instances are recreated by the next `sync`.
    pub fn set_host(&mut self, host: Option<ScriptHost>) {
        self.instances.clear();
        self.failed_creations.clear();
        self.host = host;
    }

    /// Creates instances for newly attached scripts and drops those of removed entities.
    pub fn sync(&mut self, scene: &Scene, log: &mut Vec<String>) {
        let attached = |entity: Entity, slot: usize, name: &str| {
            scene.world.get::<Scripts>(entity).and_then(|s| s.0.get(slot)).is_some_and(|s| s.name == name)
        };
        self.instances.retain(|i| attached(i.entity, i.slot, &i.name));
        self.failed_creations.retain(|(entity, slot, name)| attached(*entity, *slot, name));
        let have: HashSet<(Entity, usize)> = self.instances.iter().map(|i| (i.entity, i.slot)).collect();
        for (entity, scripts) in scene.world.iter::<Scripts>() {
            for (slot, s) in scripts.0.iter().enumerate() {
                if have.contains(&(entity, slot)) {
                    continue;
                }
                let Some(host) = &self.host else { continue };
                let key = (entity, slot, s.name.clone());
                if self.failed_creations.contains(&key) {
                    continue;
                }
                let created = catch_unwind(AssertUnwindSafe(|| {
                    let mut script = host.create(&s.name)?;
                    let mut fields = script.fields();
                    log.extend(
                        apply_props_checked(&mut fields, &s.props).into_iter().map(|warning| format!("script '{}': {warning}", s.name)),
                    );
                    drop(fields);
                    Some(script)
                }));
                match created {
                    Ok(Some(script)) => {
                        let (update, fixed) = host.callbacks(&s.name);
                        self.instances.push(Instance {
                            entity,
                            slot,
                            name: s.name.clone(),
                            script,
                            started: false,
                            failed: false,
                            update,
                            fixed,
                            synced: s.props.clone(),
                        });
                    }
                    Ok(None) => {
                        self.failed_creations.insert(key);
                        log.push(format!("script '{}' not found (is the scripts library built?)", s.name));
                    }
                    Err(p) => {
                        self.failed_creations.insert(key);
                        log.push(format!("script '{}' initialization panicked and was disabled: {}", s.name, panic_text(p)));
                    }
                }
            }
        }
    }

    /// Runs attached instances in creation order, catching callback panics.
    /// Slots removed earlier in this frame do not receive further callbacks.
    pub fn run(&mut self, phase: Phase, scene: &mut Scene, physics: &Physics, input: &Input, time: Time, log: &mut Vec<String>) {
        for inst in &mut self.instances {
            let wanted = match phase {
                Phase::Update => inst.update,
                Phase::Fixed => inst.fixed,
            };
            // Nothing to call: no per-frame work at all for this instance.
            if inst.failed || (inst.started && !wanted) {
                continue;
            }
            let Some(slot) = scene.world.get::<Scripts>(inst.entity).and_then(|s| s.0.get(inst.slot)).filter(|s| s.name == inst.name) else {
                continue;
            };
            // Another script (or the Inspector) may have changed this slot's variables.
            if slot.props != inst.synced {
                let props = slot.props.clone();
                let mut fields = inst.script.fields();
                apply_props(&mut fields, &props);
                drop(fields);
                inst.synced = props;
            }
            let mut ctx = Ctx {
                entity: inst.entity,
                scene,
                input,
                time,
                physics,
                raycast_fn: Physics::raycast_with_options,
                overlap_fn: Physics::overlap_sphere,
                touching_fn: Physics::touching,
                overlap_filtered_fn: Physics::overlap_sphere_with_options,
                log,
            };
            let result = catch_unwind(AssertUnwindSafe(|| {
                if !inst.started {
                    inst.started = true;
                    inst.script.start(&mut ctx);
                }
                if !ctx.scene.world.get::<Scripts>(inst.entity).and_then(|s| s.0.get(inst.slot)).is_some_and(|s| s.name == inst.name) {
                    return;
                }
                match phase {
                    Phase::Update if wanted => inst.script.update(&mut ctx),
                    Phase::Fixed if wanted => inst.script.fixed_update(&mut ctx),
                    _ => {}
                }
            }));
            if let Err(p) = result {
                inst.failed = true;
                log.push(format!("script panicked and was disabled: {}", panic_text(p)));
                continue;
            }
            // Publish the script's variables so other scripts and the Inspector can read them.
            let current = read_props(&inst.script.fields());
            if current != inst.synced {
                if let Some(slot) = scene.world.get_mut::<Scripts>(inst.entity).and_then(|s| s.0.get_mut(inst.slot)) {
                    if slot.name == inst.name {
                        slot.props = current.clone();
                    }
                }
                inst.synced = current;
            }
        }
    }
}

fn validate_defs(defs: &[ScriptDef]) -> Result<(), String> {
    let mut names = HashSet::new();
    for def in defs {
        if def.name.trim().is_empty() {
            return Err("script name cannot be empty".into());
        }
        if !names.insert(def.name) {
            return Err(format!("duplicate script name '{}'", def.name));
        }
    }
    Ok(())
}
