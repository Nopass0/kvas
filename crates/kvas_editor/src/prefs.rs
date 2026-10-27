//! Per-user editor preferences (not part of any project): language, look, code editor, view
//! toggles and the window / panel layout. Stored as JSON in the user's config folder.

use crate::i18n::Lang;
use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Code editors offered in Settings: (label, command). An empty command uses the system default.
pub const CODE_EDITORS: &[(&str, &str)] = &[("Kvas built-in editor", "builtin"), ("System default", ""), ("VS Code", "code"), ("Cursor", "cursor"), ("Zed", "zed"), ("RustRover", "rustrover"), ("Notepad++", "notepad++"), ("Sublime Text", "subl")];

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    /// Command that opens a file or folder; empty = system default.
    pub code_editor: String,
    pub language: Lang,
    pub theme: Theme,
    /// Interface zoom, 1.0 = 100%.
    pub ui_scale: f32,
    pub show_grid: bool,
    pub show_colliders: bool,
    pub local_space: bool,
    /// Rebuild scripts when their files change.
    pub scripts_auto_reload: bool,
    pub thumbnail_size: f32,
    /// Snapping steps and modes of the scene view.
    pub snap: crate::snap::SnapSettings,
    /// Size of the editor window in points, restored on the next start.
    pub window_size: Option<[f32; 2]>,
    pub maximized: bool,
    /// Docking layout (panel arrangement) as JSON.
    pub layout: Option<String>,
    /// False until the user has been through the first start (picks the language from the OS).
    pub initialized: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            code_editor: "builtin".into(),
            language: Lang::En,
            theme: Theme::Dark,
            ui_scale: 1.0,
            show_grid: true,
            show_colliders: true,
            local_space: false,
            scripts_auto_reload: true,
            thumbnail_size: 92.0,
            snap: Default::default(),
            window_size: None,
            maximized: false,
            layout: None,
            initialized: false,
        }
    }
}

fn file() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA").or_else(|| std::env::var_os("XDG_CONFIG_HOME")).map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("kvas").join("prefs.json"))
}

impl Prefs {
    pub fn load() -> Self {
        let mut prefs: Prefs = file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        if !prefs.initialized {
            prefs.language = Lang::detect();
            prefs.initialized = true;
            prefs.save();
        }
        prefs.ui_scale = prefs.ui_scale.clamp(0.75, 2.0);
        prefs
    }

    pub fn save(&self) {
        if let Some(f) = file() {
            let _ = std::fs::create_dir_all(f.parent().unwrap());
            let _ = std::fs::write(f, serde_json::to_string_pretty(self).unwrap_or_default());
        }
    }

    /// Opens a file at a line when the chosen editor supports it (VS Code, Cursor, Zed, RustRover).
    pub fn open_at(&self, path: &Path, line: usize) {
        let cmd = self.code_editor.trim();
        if cmd == "builtin" && path.is_file() {
            crate::code_editor::request(path, line);
            return;
        }
        let location = format!("{}:{line}", path.display());
        let args: Option<Vec<String>> = match cmd {
            "code" | "cursor" => Some(vec!["-g".into(), location]),
            "zed" => Some(vec![location]),
            "rustrover" => Some(vec!["--line".into(), line.to_string(), path.display().to_string()]),
            "subl" => Some(vec![location]),
            "notepad++" => Some(vec![format!("-n{line}"), path.display().to_string()]),
            _ => None,
        };
        let Some(args) = args else {
            return self.open(path);
        };
        #[cfg(windows)]
        let result = std::process::Command::new("cmd").args(["/C", cmd]).args(&args).spawn();
        #[cfg(not(windows))]
        let result = std::process::Command::new(cmd).args(&args).spawn();
        if result.is_err() {
            self.open(path);
        }
    }

    /// Opens a file (or a project folder) in the chosen code editor.
    pub fn open(&self, path: &Path) {
        let cmd = self.code_editor.trim();
        if cmd == "builtin" && path.is_file() {
            crate::code_editor::request(path, 1);
            return;
        }
        if cmd.is_empty() || cmd == "builtin" {
            crate::assets_panel::open_external(path);
            return;
        }
        #[cfg(windows)]
        let result = std::process::Command::new("cmd").args(["/C", cmd]).arg(path).spawn();
        #[cfg(not(windows))]
        let result = std::process::Command::new(cmd).arg(path).spawn();
        if result.is_err() {
            crate::assets_panel::open_external(path);
        }
    }
}
