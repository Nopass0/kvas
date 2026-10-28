//! Game-wide services on [`Ctx`]: the screen interface, saves and settings, files, cameras and
//! asking the host to quit.
//!
//! ```
//! use kvas_engine::prelude::*;
//!
//! #[derive(Default)]
//! pub struct Menu { volume: f32 }
//! impl Script for Menu {
//!     fn update(&mut self, ctx: &mut Ctx) {
//!         let ui = ctx.ui();
//!         ui.panel([0.0, 0.0, ui.width(), 720.0], [0.0, 0.0, 0.0, 0.7], 0.0);
//!         ui.text("PAUSED", [ui.width() / 2.0, 200.0], 48.0, [1.0; 4], "", Align::Center);
//!         self.volume = ui.slider("volume", [540.0, 300.0, 200.0, 24.0], self.volume);
//!         if ui.button("save", [540.0, 360.0, 200.0, 40.0], "SAVE") {
//!             let _ = ctx.save_game("slot1", "Chapter 1", &serde_json::json!({ "volume": self.volume }));
//!         }
//!         if ui.button("quit", [540.0, 420.0, 200.0, 40.0], "QUIT") {
//!             ctx.quit_game();
//!         }
//!     }
//! }
//! ```

use crate::canvas::Canvas;
use crate::components::{Camera, Transform};
use crate::script::Ctx;
use crate::storage::{SaveInfo, Storage};
use kvas_ecs::Entity;

impl Ctx<'_> {
    /// The screen interface of this frame (see [`crate::canvas`]).
    pub fn ui(&self) -> &Canvas {
        &self.input.canvas
    }
    /// Width of the virtual interface screen (its height is always 720).
    pub fn ui_width(&self) -> f32 {
        self.input.canvas.width()
    }

    // ---- saves and settings
    /// The game's data folder: saves, settings and files.
    pub fn storage(&self) -> Storage {
        self.input.storage()
    }
    /// Writes a save slot (`title` is shown in load menus; the scene is recorded too).
    pub fn save_game(&self, slot: &str, title: &str, data: &serde_json::Value) -> Result<(), String> {
        self.input.storage().save(slot, title, data)
    }
    /// Reads a save slot.
    pub fn load_game(&self, slot: &str) -> Option<serde_json::Value> {
        self.input.storage().load(slot)
    }
    /// Every save, newest first.
    pub fn save_slots(&self) -> Vec<SaveInfo> {
        self.input.storage().slots()
    }
    /// Whether a slot exists.
    pub fn has_save(&self, slot: &str) -> bool {
        self.input.storage().has_save(slot)
    }
    /// A persistent setting (kept across runs), or `default`.
    pub fn setting_f32(&self, key: &str, default: f32) -> f32 {
        self.input.storage().setting(key).and_then(|v| v.as_f64()).map_or(default, |v| v as f32)
    }
    /// A persistent on/off setting, or `default`.
    pub fn setting_bool(&self, key: &str, default: bool) -> bool {
        self.input.storage().setting(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }
    /// A persistent text setting, or `default`.
    pub fn setting_text(&self, key: &str, default: &str) -> String {
        self.input.storage().setting(key).and_then(|v| v.as_str().map(str::to_string)).unwrap_or_else(|| default.to_string())
    }
    /// Stores a persistent setting.
    pub fn set_setting(&self, key: &str, value: serde_json::Value) {
        if let Err(e) = self.input.storage().set_setting(key, value) {
            eprintln!("setting {key}: {e}");
        }
    }

    // ---- cameras
    /// The camera the game looks through.
    pub fn active_camera(&self) -> Option<Entity> {
        crate::components::active_camera(&self.scene.world)
    }
    /// Looks through another camera from the next frame on (security monitors, cut-scenes...).
    pub fn set_active_camera(&mut self, camera: Entity) -> bool {
        crate::components::set_active_camera(&mut self.scene.world, camera)
    }
    /// Switches to the camera named `name`.
    pub fn set_active_camera_named(&mut self, name: &str) -> bool {
        match self.find(name) {
            Some(e) => self.set_active_camera(e),
            None => false,
        }
    }
    /// Adds a camera shader (`fn effect`) to a camera; returns its index.
    pub fn add_camera_effect(&mut self, camera: Entity, shader: &str, params: [f32; 4]) -> Option<usize> {
        let c = self.scene.world.get_mut::<Camera>(camera)?;
        c.effects.push(crate::fx::PostEffect { shader: shader.to_string(), enabled: true, params });
        Some(c.effects.len() - 1)
    }
    /// Changes the numbers of a camera shader (fade a VHS, pulse damage...).
    pub fn set_camera_effect(&mut self, camera: Entity, index: usize, enabled: bool, params: [f32; 4]) {
        if let Some(fx) = self.scene.world.get_mut::<Camera>(camera).and_then(|c| c.effects.get_mut(index)) {
            fx.enabled = enabled;
            fx.params = params;
        }
    }
    /// Field of view of a camera in degrees (zoom, sprint effect).
    pub fn set_camera_fov(&mut self, camera: Entity, fov: f32) {
        if let Some(c) = self.scene.world.get_mut::<Camera>(camera) {
            c.fov = fov.clamp(5.0, 170.0);
            c.focal_length = 0.0;
        }
    }
    /// World position of the active camera.
    pub fn camera_position(&self) -> Option<glam::Vec3> {
        let e = self.active_camera()?;
        Some(crate::hierarchy::global_matrix(&self.scene.world, e).w_axis.truncate())
    }
    /// Direction the active camera looks along.
    pub fn camera_forward(&self) -> Option<glam::Vec3> {
        let e = self.active_camera()?;
        Some(-crate::hierarchy::global_matrix(&self.scene.world, e).z_axis.truncate().normalize_or_zero())
    }
    /// Places a camera and points it at a target (cut-scenes).
    pub fn place_camera(&mut self, camera: Entity, position: glam::Vec3, target: glam::Vec3) {
        if let Some(t) = self.scene.world.get_mut::<Transform>(camera) {
            t.position = position;
            t.look_at(target, glam::Vec3::Y);
        }
    }

    // ---- host
    /// Closes the game (in the editor it stops Play).
    pub fn quit_game(&self) {
        self.input.request_quit();
    }
    /// Switches the window to fullscreen or back.
    pub fn set_fullscreen(&self, on: bool) {
        self.input.request_fullscreen(on);
    }
}
