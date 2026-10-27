//! Project layout on disk:
//!   project.json   – name and start scene
//!   assets/        – scenes, models, textures, shaders (packed into game.pak on build)
//!   scripts/       – a Cargo crate (cdylib) with the game's Rust scripts

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// An input action and the inputs bound to it ("Space", "W", "MouseLeft", ...).
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct InputBinding {
    /// Case-sensitive action name used by script input queries.
    pub name: String,
    /// Alternative physical input names, e.g. `Space`, `W`, `MouseLeft`.
    pub keys: Vec<String>,
}

/// Standard movement, jumping, sprinting, interaction and mouse-fire bindings.
pub fn default_bindings() -> Vec<InputBinding> {
    let b = |name: &str, keys: &[&str]| InputBinding { name: name.into(), keys: keys.iter().map(|k| k.to_string()).collect() };
    vec![
        b("MoveForward", &["W", "ArrowUp"]),
        b("MoveBack", &["S", "ArrowDown"]),
        b("MoveLeft", &["A", "ArrowLeft"]),
        b("MoveRight", &["D", "ArrowRight"]),
        b("Jump", &["Space"]),
        b("Sprint", &["Shift"]),
        b("Interact", &["E"]),
        b("Fire", &["MouseLeft"]),
        b("AltFire", &["MouseRight"]),
    ]
}

/// Serializable project manifest. Missing fields receive backwards-compatible defaults.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    /// Display name shown by the editor and player.
    pub name: String,
    /// Initial scene JSON path relative to the assets directory.
    pub start_scene: String,
    /// Cargo package name of the scripts crate (also the dll base name).
    pub scripts_package: String,
    /// "3D" or "2D" (2D projects start with an orthographic camera).
    pub kind: String,
    /// Named input action definitions, editable in Project Settings.
    pub input: Vec<InputBinding>,
    /// How the built game looks and starts (set in the Build window).
    pub build: BuildSettings,
    /// Fixed-step cadence and catch-up policy shared by editor and standalone games.
    pub timing: crate::runtime::RuntimeConfig,
    /// Initial simulation speed (0..10).
    pub time_scale: f32,
    /// Initial audio master gain (0..1).
    pub master_volume: f32,
}

/// Settings of the standalone game made by "Build Game".
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BuildSettings {
    /// Window title; empty uses the project name.
    pub title: String,
    /// Executable file name without extension; empty derives it from the project name.
    pub exe_name: String,
    /// Shown in the build notes.
    pub version: String,
    /// Publisher shown in the build notes.
    pub company: String,
    /// Icon image (png/jpg/ico), relative to the project folder or absolute; empty uses the Kvas logo.
    pub icon: String,
    /// Initial window width in pixels.
    pub width: u32,
    /// Initial window height in pixels.
    pub height: u32,
    /// Start in fullscreen.
    pub fullscreen: bool,
    /// Start maximized.
    pub maximized: bool,
    /// Allow resizing the window.
    pub resizable: bool,
    /// Wait for the display refresh (no tearing).
    pub vsync: bool,
    /// Optimised build (slower to compile, much faster to run).
    pub release: bool,
}

impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            title: String::new(),
            exe_name: String::new(),
            version: "1.0.0".into(),
            company: String::new(),
            icon: String::new(),
            width: 1280,
            height: 720,
            fullscreen: false,
            maximized: false,
            resizable: true,
            vsync: true,
            release: true,
        }
    }
}

impl ProjectConfig {
    /// Copies input definitions into the representation accepted by Runtime/Input.
    pub fn binding_defs(&self) -> Vec<(String, Vec<String>)> {
        self.input.iter().map(|b| (b.name.clone(), b.keys.clone())).collect()
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "Game".into(),
            start_scene: "scenes/main.scene.json".into(),
            scripts_package: "game_scripts".into(),
            kind: "3D".into(),
            input: default_bindings(),
            build: BuildSettings::default(),
            timing: Default::default(),
            time_scale: 1.0,
            master_volume: 1.0,
        }
    }
}

impl ProjectConfig {
    /// Parses a manifest, returning a contextual JSON error on malformed input.
    /// Use [`Self::validate`] to check semantic settings after parsing.
    pub fn from_json(text: &str) -> Result<ProjectConfig, String> {
        serde_json::from_str(text).map_err(|e| format!("project.json: {e}"))
    }

    /// Reports semantic configuration issues without mutating the manifest.
    /// Empty action bindings are allowed, but duplicate actions and unknown sources are reported.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if let Err(e) = self.timing.validate() { errors.push(e); }
        if !self.time_scale.is_finite() || !(0.0..=10.0).contains(&self.time_scale) { errors.push("time_scale must be between 0 and 10".into()); }
        if !self.master_volume.is_finite() || !(0.0..=1.0).contains(&self.master_volume) { errors.push("master_volume must be between 0 and 1".into()); }
        if self.name.trim().is_empty() {
            errors.push("project name is empty".into());
        }
        if crate::assets::normalize_path(&self.start_scene).is_err() {
            errors.push("start_scene must be an asset-relative path".into());
        }
        if !matches!(self.kind.as_str(), "2D" | "3D") {
            errors.push("project kind must be 2D or 3D".into());
        }
        if self.scripts_package.is_empty() || !self.scripts_package.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
            errors.push("scripts_package must be a non-empty Cargo package name".into());
        }
        let mut seen = std::collections::HashSet::new();
        for binding in &self.input {
            if binding.name.trim().is_empty() {
                errors.push("input action name is empty".into());
            }
            if !seen.insert(&binding.name) {
                errors.push(format!("duplicate input action '{}'", binding.name));
            }
            for source in &binding.keys {
                if crate::input::Source::parse(source).is_none() {
                    errors.push(format!("action '{}': unknown input '{source}'", binding.name));
                }
            }
        }
        errors
    }
}

/// Open project directory and editable manifest. Cloning does not copy files.
#[derive(Clone)]
pub struct Project {
    /// Absolute project directory after [`Self::open`].
    pub root: PathBuf,
    /// In-memory manifest; call [`Self::save`] to persist edits.
    pub config: ProjectConfig,
}

impl Project {
    /// Opens a directory containing `project.json`, resolving its absolute path.
    /// I/O/JSON failures return contextual strings. Semantic validation is explicit.
    pub fn open(root: &Path) -> Result<Project, String> {
        let text =
            std::fs::read_to_string(root.join("project.json")).map_err(|e| format!("{}: {e}", root.join("project.json").display()))?;
        // Absolute, so child processes started in other folders (cargo) still find it.
        let root = std::path::absolute(root).unwrap_or_else(|_| root.to_path_buf());
        Ok(Project { root, config: ProjectConfig::from_json(&text)? })
    }

    /// Atomically replaces `project.json` using a sibling temporary file.
    /// The project directory must already exist.
    pub fn save(&self) -> std::io::Result<()> {
        let bytes = serde_json::to_vec_pretty(&self.config).map_err(std::io::Error::other)?;
        crate::file_io::atomic_write(&self.root.join("project.json"), &bytes)
    }

    /// Assets directory used for loose-file loading.
    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }

    /// Cargo project directory for native scripts.
    pub fn scripts_dir(&self) -> PathBuf {
        self.root.join("scripts")
    }

    /// File name of the compiled scripts library on this platform.
    pub fn script_lib_name(&self) -> String {
        let base = self.config.scripts_package.replace('-', "_");
        if cfg!(windows) {
            format!("{base}.dll")
        } else if cfg!(target_os = "macos") {
            format!("lib{base}.dylib")
        } else {
            format!("lib{base}.so")
        }
    }
}
