//! Files of a game on the player's computer: save slots, persistent settings and a small file
//! system API. Everything lives under the game's data folder (set by the host: `%APPDATA%/<game>`
//! for a built game, `<project>/.kvas/data` while playing in the editor), so a script cannot
//! touch other files by accident. [`external`] reads and writes anywhere, on purpose.
//!
//! ```
//! use kvas_engine::storage::Storage;
//! let dir = std::env::temp_dir().join(format!("kvas-storage-doc-{}", std::process::id()));
//! let s = Storage::new(&dir);
//! s.save("slot1", "Chapter 2", &serde_json::json!({"hp": 3})).unwrap();
//! assert_eq!(s.load("slot1").unwrap()["hp"], 3);
//! assert_eq!(s.slots()[0].title, "Chapter 2");
//! s.set_setting("volume", serde_json::json!(0.5)).unwrap();
//! assert_eq!(s.setting("volume"), Some(serde_json::json!(0.5)));
//! let _ = std::fs::remove_dir_all(dir);
//! ```

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// One save slot as listed by [`Storage::slots`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SaveInfo {
    /// Slot name (file name without `.json`).
    pub slot: String,
    /// Text shown in a load menu (chapter, place...).
    pub title: String,
    /// Seconds since 1970 when it was written.
    pub time: u64,
    /// Scene the game was in.
    pub scene: String,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    info: SaveInfo,
    data: serde_json::Value,
}

/// The game's data folder.
#[derive(Clone, Debug, Default)]
pub struct Storage {
    root: PathBuf,
    scene: String,
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Turns a script path into one inside `root`, refusing `..`, drive letters and absolute paths.
fn inside(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let p = Path::new(rel);
    if rel.is_empty() || p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(format!("'{rel}' must be a relative path inside the game folder"));
    }
    Ok(root.join(p))
}

fn slot_ok(slot: &str) -> Result<(), String> {
    if slot.is_empty() || slot.len() > 64 || !slot.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(format!("save slot '{slot}': use 1-64 letters, digits, '-' or '_'"));
    }
    Ok(())
}

impl Storage {
    /// Storage rooted at a folder (created on first write).
    pub fn new(root: &Path) -> Self {
        Self { root: root.to_path_buf(), scene: String::new() }
    }
    /// The standard folder for a game: `%APPDATA%/Kvas/<game>` on Windows, `~/.local/share/kvas/<game>` elsewhere.
    pub fn for_game(game: &str) -> Self {
        let safe: String = game.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' }).collect();
        let base = std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share"))).unwrap_or_else(std::env::temp_dir);
        Self::new(&base.join("Kvas").join(if safe.trim().is_empty() { "game" } else { safe.trim() }))
    }
    /// The data folder.
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Records the running scene (written into saves).
    pub fn set_scene(&mut self, scene: &str) {
        self.scene = scene.to_string();
    }

    // ---- saves
    /// Writes a save slot. `title` is what a load menu shows.
    pub fn save(&self, slot: &str, title: &str, data: &serde_json::Value) -> Result<(), String> {
        slot_ok(slot)?;
        let file = SaveFile { info: SaveInfo { slot: slot.into(), title: title.into(), time: now(), scene: self.scene.clone() }, data: data.clone() };
        let text = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
        let dir = self.root.join("saves");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        // Write next to it and rename, so a crash never leaves half a save.
        let tmp = dir.join(format!("{slot}.json.tmp"));
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, dir.join(format!("{slot}.json"))).map_err(|e| e.to_string())
    }
    /// Reads a save slot's data.
    pub fn load(&self, slot: &str) -> Option<serde_json::Value> {
        slot_ok(slot).ok()?;
        let text = std::fs::read_to_string(self.root.join("saves").join(format!("{slot}.json"))).ok()?;
        serde_json::from_str::<SaveFile>(&text).ok().map(|f| f.data)
    }
    /// Typed save: any `Serialize` value.
    pub fn save_as<T: Serialize>(&self, slot: &str, title: &str, value: &T) -> Result<(), String> {
        self.save(slot, title, &serde_json::to_value(value).map_err(|e| e.to_string())?)
    }
    /// Typed load.
    pub fn load_as<T: for<'de> Deserialize<'de>>(&self, slot: &str) -> Option<T> {
        serde_json::from_value(self.load(slot)?).ok()
    }
    /// Every save, newest first.
    pub fn slots(&self) -> Vec<SaveInfo> {
        let mut out: Vec<SaveInfo> = std::fs::read_dir(self.root.join("saves"))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| serde_json::from_str::<SaveFile>(&std::fs::read_to_string(e.path()).ok()?).ok().map(|f| f.info))
            .collect();
        out.sort_by(|a, b| b.time.cmp(&a.time).then(a.slot.cmp(&b.slot)));
        out
    }
    /// Whether a slot exists.
    pub fn has_save(&self, slot: &str) -> bool {
        slot_ok(slot).is_ok() && self.root.join("saves").join(format!("{slot}.json")).exists()
    }
    /// Removes a slot.
    pub fn delete_save(&self, slot: &str) -> Result<(), String> {
        slot_ok(slot)?;
        std::fs::remove_file(self.root.join("saves").join(format!("{slot}.json"))).map_err(|e| e.to_string())
    }

    // ---- settings (one JSON object kept across runs)
    fn settings_path(&self) -> PathBuf {
        self.root.join("settings.json")
    }
    /// All settings.
    pub fn settings(&self) -> serde_json::Map<String, serde_json::Value> {
        std::fs::read_to_string(self.settings_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }
    /// One setting.
    pub fn setting(&self, key: &str) -> Option<serde_json::Value> {
        self.settings().get(key).cloned()
    }
    /// Changes one setting and writes the file.
    pub fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), String> {
        let mut all = self.settings();
        all.insert(key.to_string(), value);
        std::fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        std::fs::write(self.settings_path(), serde_json::to_string_pretty(&all).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }

    // ---- files inside the data folder
    /// Reads a text file of the data folder.
    pub fn read_text(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(inside(&self.root, rel)?).map_err(|e| format!("{rel}: {e}"))
    }
    /// Writes a text file (folders are created).
    pub fn write_text(&self, rel: &str, text: &str) -> Result<(), String> {
        self.write_bytes(rel, text.as_bytes())
    }
    /// Reads bytes.
    pub fn read_bytes(&self, rel: &str) -> Result<Vec<u8>, String> {
        std::fs::read(inside(&self.root, rel)?).map_err(|e| format!("{rel}: {e}"))
    }
    /// Writes bytes (folders are created).
    pub fn write_bytes(&self, rel: &str, bytes: &[u8]) -> Result<(), String> {
        let path = inside(&self.root, rel)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, bytes).map_err(|e| format!("{rel}: {e}"))
    }
    /// Appends a line (logs, statistics).
    pub fn append_line(&self, rel: &str, line: &str) -> Result<(), String> {
        use std::io::Write;
        let path = inside(&self.root, rel)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| e.to_string())?;
        writeln!(f, "{line}").map_err(|e| e.to_string())
    }
    /// Whether a file or folder exists.
    pub fn exists(&self, rel: &str) -> bool {
        inside(&self.root, rel).is_ok_and(|p| p.exists())
    }
    /// Names in a folder (`""` = the data folder itself), folders end with `/`.
    pub fn list(&self, rel: &str) -> Vec<String> {
        let dir = if rel.is_empty() { Ok(self.root.clone()) } else { inside(&self.root, rel) };
        let Ok(dir) = dir else { return Vec::new() };
        let mut out: Vec<String> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() { format!("{name}/") } else { name }
            })
            .collect();
        out.sort();
        out
    }
    /// Removes a file or a whole folder.
    pub fn delete(&self, rel: &str) -> Result<(), String> {
        let p = inside(&self.root, rel)?;
        if p.is_dir() { std::fs::remove_dir_all(p) } else { std::fs::remove_file(p) }.map_err(|e| e.to_string())
    }
}

/// Unrestricted access to the computer's file system (any absolute path). Use for tools, mods
/// and import/export; prefer [`Storage`] for game data.
pub mod external {
    /// Reads a text file.
    pub fn read_text(path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
    }
    /// Writes a text file, creating folders.
    pub fn write_text(path: &str, text: &str) -> Result<(), String> {
        write_bytes(path, text.as_bytes())
    }
    /// Reads bytes.
    pub fn read_bytes(path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{path}: {e}"))
    }
    /// Writes bytes, creating folders.
    pub fn write_bytes(path: &str, bytes: &[u8]) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
    }
    /// Entries of a folder (full paths).
    pub fn list(path: &str) -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(path).into_iter().flatten().flatten().map(|e| e.path().to_string_lossy().to_string()).collect();
        out.sort();
        out
    }
    /// Whether the path exists.
    pub fn exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    /// The user's documents folder (or home), for "export to..." features.
    pub fn documents_dir() -> String {
        std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map(|h| format!("{h}/Documents")).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_slots_and_files() {
        let dir = std::env::temp_dir().join(format!("kvas-storage-test-{}", std::process::id()));
        let mut s = Storage::new(&dir);
        s.set_scene("scenes/a.scene.json");
        assert!(s.save("../x", "t", &serde_json::json!(1)).is_err());
        assert!(s.write_text("../escape.txt", "no").is_err());
        assert!(s.write_text("C:/escape.txt", "no").is_err());
        s.save_as("auto", "Start", &vec![1, 2, 3]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        s.save("manual", "Later", &serde_json::json!({"a": 1})).unwrap();
        let slots = s.slots();
        assert_eq!(slots.iter().map(|i| i.slot.as_str()).collect::<Vec<_>>(), ["manual", "auto"]);
        assert_eq!(slots[1].scene, "scenes/a.scene.json");
        assert_eq!(s.load_as::<Vec<i32>>("auto"), Some(vec![1, 2, 3]));
        s.write_text("notes/day1.txt", "hello").unwrap();
        s.append_line("notes/day1.txt", "world").unwrap();
        assert_eq!(s.read_text("notes/day1.txt").unwrap(), "helloworld\n");
        assert_eq!(s.list("notes"), ["day1.txt"]);
        assert!(s.list("").contains(&"notes/".to_string()));
        s.delete_save("auto").unwrap();
        assert!(!s.has_save("auto"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
