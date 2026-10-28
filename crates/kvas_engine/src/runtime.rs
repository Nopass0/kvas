//! The game loop shared by the editor's Play mode and the standalone player.
//!
//! A tick runs scripts, environment and procedural systems, then zero or more
//! fixed physics steps and finally refreshes world transforms. The shell supplies
//! input before ticking. Frame input edges remain visible to every fixed step in
//! that frame; scripts should consume one-shot gameplay actions in `update`.
//!
//! ```
//! use kvas_engine::{runtime::Runtime, scene::Scene};
//! let mut runtime = Runtime::new(Scene::default(), None);
//! runtime.tick(1.0 / 60.0);
//! assert_eq!(runtime.time.frame, 1);
//! runtime.set_paused(true);
//! runtime.tick(1.0);
//! assert_eq!(runtime.time.frame, 1);
//! ```

use crate::hierarchy;
use crate::input::Input;
use crate::physics::Physics;
use crate::scene::Scene;
use crate::script::{Phase, ScriptHost, ScriptRuntime, Time};

const FIXED_DT: f32 = 1.0 / 60.0;
const MAX_FRAME_DT: f32 = 0.1;

/// Validated timing policy for a [`Runtime`]. Defaults retain the 60 Hz loop.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct RuntimeConfig {
    /// Fixed simulation interval in seconds; must be finite and positive.
    pub fixed_dt: f32,
    /// Maximum accepted wall-clock frame interval, finite and positive.
    pub max_frame_dt: f32,
    /// Maximum fixed steps per tick. Excess whole steps are dropped, preserving
    /// only the fractional remainder to prevent an unbounded catch-up spiral.
    pub max_fixed_steps: u32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self { fixed_dt: FIXED_DT, max_frame_dt: MAX_FRAME_DT, max_fixed_steps: 8 }
    }
}

impl RuntimeConfig {
    /// Checks timing bounds before constructing a runtime.
    pub fn validate(&self) -> Result<(), String> {
        if !self.fixed_dt.is_finite() || self.fixed_dt <= 0.0 {
            return Err("fixed_dt must be finite and positive".into());
        }
        if !self.max_frame_dt.is_finite() || self.max_frame_dt <= 0.0 {
            return Err("max_frame_dt must be finite and positive".into());
        }
        if self.max_fixed_steps == 0 {
            return Err("max_fixed_steps must be positive".into());
        }
        Ok(())
    }
}

/// Timing diagnostics for the most recent tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameStats {
    /// Number of completed fixed simulation steps.
    pub fixed_steps: u32,
    /// Simulation seconds dropped by the fixed-step catch-up limit.
    pub dropped_time: f32,
    /// Remaining accumulator divided by the fixed interval, in `[0, 1)`.
    /// Exposed for render interpolation; the engine does not interpolate poses itself.
    pub interpolation_alpha: f32,
}

/// Owned gameplay state shared by editor Play mode and the standalone player.
/// Dropping the runtime destroys script instances before unloading their library.
pub struct Runtime {
    /// Mutable gameplay world and environment.
    pub scene: Scene,
    /// Rapier simulation and ray queries, updated at the fixed cadence.
    pub physics: Physics,
    /// Script instances and hot-reload host.
    pub scripts: ScriptRuntime,
    /// Input snapshot supplied by the hosting shell before each tick.
    pub input: Input,
    /// Scaled simulation time visible to scripts.
    pub time: Time,
    /// Script messages and runtime diagnostics. The host may drain these at any time.
    pub log: Vec<String>,
    accumulator: f32,
    config: RuntimeConfig,
    paused: bool,
    time_scale: f32,
    stats: FrameStats,
}

impl Runtime {
    /// Applies the project's input bindings (call right after `new`).
    pub fn set_bindings<S: AsRef<str>>(&mut self, defs: &[(String, Vec<S>)]) {
        self.input.set_bindings(defs);
    }

    /// Creates a 60 Hz runtime, initializing world transforms before the first tick.
    pub fn new(scene: Scene, host: Option<ScriptHost>) -> Self {
        Self::with_config(scene, host, RuntimeConfig::default()).expect("valid default timing")
    }

    /// Creates a runtime with a validated timing policy.
    /// Returns an error for non-finite/non-positive intervals or a zero step limit.
    pub fn with_config(mut scene: Scene, host: Option<ScriptHost>, config: RuntimeConfig) -> Result<Self, String> {
        config.validate()?;
        hierarchy::update_globals(&mut scene.world);
        let mut scripts = ScriptRuntime::default();
        scripts.set_host(host);
        Ok(Self {
            scene,
            physics: Physics::new(),
            scripts,
            input: Input::default(),
            time: Time::default(),
            log: Vec::new(),
            accumulator: 0.0,
            config,
            paused: false,
            time_scale: 1.0,
            stats: FrameStats::default(),
        })
    }

    /// Active timing configuration. Construct a new runtime to change the fixed cadence.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Whether regular ticks are suspended.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Suspends or resumes ticks. Existing fractional fixed time is preserved.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Current simulation speed multiplier.
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Sets speed in `[0, 100]`; rejects invalid values without changing state.
    /// Zero freezes simulation time like pause. Wall-frame clamping occurs before scaling.
    pub fn set_time_scale(&mut self, scale: f32) -> Result<(), String> {
        if !scale.is_finite() || !(0.0..=100.0).contains(&scale) {
            return Err("time scale must be finite and between 0 and 100".into());
        }
        self.time_scale = scale;
        Ok(())
    }

    /// Timing diagnostics from the most recent tick (zeroed on a skipped tick).
    pub fn frame_stats(&self) -> FrameStats {
        self.stats
    }

    /// Advances one fixed-size simulation frame even when paused or time scale is zero.
    /// Useful for editor single-step controls. It does not consume shell input.
    pub fn step_once(&mut self) {
        self.advance(self.config.fixed_dt);
    }

    /// Drains pending script/runtime log messages, preserving their order.
    pub fn drain_log(&mut self) -> std::vec::Drain<'_, String> {
        self.log.drain(..)
    }

    /// Advances a wall-clock frame. Negative/non-finite intervals and paused
    /// frames are skipped. Large intervals are clamped before applying time scale.
    pub fn tick(&mut self, dt: f32) {
        self.stats = FrameStats::default();
        if self.paused || self.time_scale == 0.0 || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.advance(dt.min(self.config.max_frame_dt) * self.time_scale);
    }

    fn advance(&mut self, dt: f32) {
        self.stats = FrameStats::default();
        self.time.dt = dt;
        self.time.elapsed += dt;
        self.time.frame = self.time.frame.saturating_add(1);

        self.input.begin_frame(dt);
        self.scene.signals.begin_frame();
        self.scripts.sync(&self.scene, &mut self.log);
        self.scripts.run(Phase::Update, &mut self.scene, &self.physics, &self.input, self.time, &mut self.log);
        crate::gameplay::update_lifetimes(&mut self.scene.world, dt);

        let sky = &mut self.scene.settings.atmosphere;
        if sky.speed != 0.0 {
            sky.time = (sky.time + sky.speed * dt).rem_euclid(24.0);
        }
        self.update_autofocus(dt);
        crate::interaction::update(&mut self.scene.world, dt);
        crate::modifiers::update(&mut self.scene.world, self.time.elapsed, dt);
        crate::tween::update(&mut self.scene.world, dt);
        crate::animation::update(&mut self.scene.world, dt);
        crate::neural::update(&mut self.scene.world, dt);
        crate::procedural::update(&mut self.scene.world, self.time.elapsed, dt, true);

        self.accumulator += dt;
        while self.accumulator >= self.config.fixed_dt && self.stats.fixed_steps < self.config.max_fixed_steps {
            self.accumulator -= self.config.fixed_dt;
            self.scripts.run(
                Phase::Fixed,
                &mut self.scene,
                &self.physics,
                &self.input,
                Time { dt: self.config.fixed_dt, ..self.time },
                &mut self.log,
            );
            crate::water::apply_buoyancy(&mut self.scene.world, self.time.elapsed, self.config.fixed_dt);
            self.physics.step(&mut self.scene, self.config.fixed_dt);
            self.stats.fixed_steps += 1;
        }
        if self.accumulator >= self.config.fixed_dt {
            let remainder = self.accumulator % self.config.fixed_dt;
            self.stats.dropped_time = self.accumulator - remainder;
            self.accumulator = remainder;
        }
        self.stats.interpolation_alpha = self.accumulator / self.config.fixed_dt;
        hierarchy::update_globals(&mut self.scene.world);
    }
}

impl Runtime {
    /// Racks the focus distance of the primary camera onto what is in the middle of the screen.
    fn update_autofocus(&mut self, dt: f32) {
        let world = &self.scene.world;
        let Some((entity, camera)) = world.iter::<crate::components::Camera>().find(|(_, c)| c.primary && c.autofocus && c.dof).map(|(e, c)| (e, c.clone())) else { return };
        let m = hierarchy::global_matrix(world, entity);
        let (origin, forward) = (m.w_axis.truncate(), -m.z_axis.truncate().normalize_or_zero());
        let target = self.physics.raycast(origin + forward * camera.near, forward, camera.far.min(300.0)).map_or(camera.far.min(60.0), |h| h.distance + camera.near);
        if let Some(c) = self.scene.world.get_mut::<crate::components::Camera>(entity) {
            let k = 1.0 - (-camera.focus_speed.max(0.1) * dt).exp();
            c.focus_distance += (target - c.focus_distance) * k;
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // Script instances reference code in the library; drop them first.
        self.scripts.set_host(None);
    }
}
