//! Script source files of the open project: listing, creating and attaching scripts.

use crate::editor::Editor;
use crate::i18n::{t, tf};
use crate::script_templates;
use kvas_engine::prelude::*;
use std::path::PathBuf;

fn check_name(existing: &[String], name: &str) -> Result<(), String> {
    let valid = name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && name.chars().all(|c| c.is_ascii_alphanumeric());
    if !valid {
        return Err(t("Script names are CamelCase letters and digits, e.g. DoorOpener").into());
    }
    if existing.iter().any(|n| n == name) {
        return Err(tf("Script {} already exists", &[name]));
    }
    Ok(())
}

/// Build script and lib.rs that make every file under `scripts/src` a registered script module.
pub const BUILD_RS: &str = include_str!("../library/project/build.rs");
pub const LIB_RS: &str = include_str!("../library/project/lib.rs");

/// True when the project registers scripts automatically (no manual lists).
pub fn is_auto(root: &std::path::Path) -> bool {
    root.join("scripts/build.rs").exists()
        && std::fs::read_to_string(root.join("scripts/src/lib.rs")).is_ok_and(|s| s.contains("kvas_scripts.rs"))
}

fn walk_rs(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = read.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk_rs(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// `(script name, source file)` of every `impl Script for X` under `scripts/src`.
pub fn scan_scripts(root: &std::path::Path) -> Vec<(String, PathBuf)> {
    let mut files = Vec::new();
    walk_rs(&root.join("scripts/src"), &mut files);
    let mut out = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else { continue };
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("impl Script for ") {
                let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if !name.is_empty() {
                    out.push((name, file.clone()));
                }
            }
        }
    }
    out
}

/// Splits a lib.rs that holds several scripts into `(name, file text)` pairs, one per script.
/// Module declarations and the `register_scripts!` list are dropped (they are generated now).
pub fn split_inline(source: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = source.lines().collect();
    let is_start = |l: &str| l.starts_with("script_params!") || l.starts_with("pub struct ") || l.starts_with("struct ");
    let mut starts: Vec<usize> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if is_start(l) {
            let mut j = i;
            while j > 0 && (lines[j - 1].starts_with("///") || lines[j - 1].starts_with("#[")) && !lines[j - 1].starts_with("#[path") {
                j -= 1;
            }
            if starts.last().is_none_or(|p| *p < j) {
                starts.push(j);
            }
        }
    }
    if starts.is_empty() {
        return Vec::new();
    }
    let preamble: Vec<&str> = lines[..starts[0]].iter().copied().filter(|l| l.trim_start().starts_with("use ")).collect();
    let has_impl = |a: usize, b: usize| lines[a..b].iter().any(|l| l.starts_with("impl Script for "));
    // chunks without an impl (a plain struct, params) merge into the next chunk
    let mut merged: Vec<(usize, usize)> = Vec::new();
    let mut pending: Option<usize> = None;
    for (k, &a) in starts.iter().enumerate() {
        let b = starts.get(k + 1).copied().unwrap_or(lines.len());
        let start = pending.take().unwrap_or(a);
        if has_impl(start, b) {
            merged.push((start, b));
        } else {
            pending = Some(start);
        }
    }
    let mut out = Vec::new();
    for (a, b) in merged {
        let body: Vec<&str> = lines[a..b]
            .iter()
            .copied()
            .filter(|l| !(l.starts_with("register_scripts!") || l.starts_with("mod ") || l.starts_with("pub use ") || l.starts_with("#[path")))
            .collect();
        let Some(name) = body
            .iter()
            .find_map(|l| l.strip_prefix("impl Script for "))
            .map(|r| r.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<String>())
        else {
            continue;
        };
        let mut uses: Vec<&str> = preamble.clone();
        if !uses.iter().any(|u| u.contains("kvas_engine::prelude")) {
            uses.insert(0, "use kvas_engine::prelude::*;");
        }
        let mut text = format!("//! {name} script.\n\n");
        for u in uses {
            text.push_str(u);
            text.push('\n');
        }
        text.push('\n');
        text.push_str(body.join("\n").trim_end());
        text.push('\n');
        out.push((name, text));
    }
    out
}

/// Converts a project that lists scripts by hand into the automatic layout: every script gets its own
/// file under `scripts/src` and `build.rs` registers them. Returns true when something changed.
pub fn ensure_auto(root: &std::path::Path) -> Result<bool, String> {
    if is_auto(root) || !root.join("scripts/Cargo.toml").exists() {
        return Ok(false);
    }
    let io = |e: std::io::Error| e.to_string();
    let lib = root.join("scripts/src/lib.rs");
    let source = std::fs::read_to_string(&lib).unwrap_or_default();
    for (name, text) in split_inline(&source) {
        let file = root.join("scripts/src").join(format!("{}.rs", script_templates::module_name(&name)));
        if !file.exists() {
            std::fs::write(&file, text).map_err(io)?;
        }
    }
    std::fs::write(root.join("scripts/build.rs"), BUILD_RS).map_err(io)?;
    std::fs::write(&lib, LIB_RS).map_err(io)?;
    Ok(true)
}

impl Editor {
    pub fn scripts_file(&self) -> PathBuf {
        self.project.root.join("scripts/src/lib.rs")
    }

    /// The source file of a script: its own module when it has one, otherwise lib.rs.
    pub fn script_source(&self, name: &str) -> PathBuf {
        if let Some((_, file)) = scan_scripts(&self.project.root).into_iter().find(|(n, _)| n == name) {
            return file;
        }
        self.scripts_file()
    }

    /// Where a script lives: its source file and the line of its definition.
    pub fn script_location(&self, name: &str) -> (PathBuf, usize) {
        let file = self.script_source(name);
        let line = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| {
                let needles = [format!("pub struct {name}"), format!("struct {name}"), format!("impl Script for {name}")];
                needles.iter().find_map(|n| text.lines().position(|l| l.contains(n.as_str())))
            })
            .map_or(1, |i| i + 1);
        (file, line)
    }

    /// Opens a script in the code editor at its definition.
    pub fn open_script(&mut self, name: &str) {
        let (file, line) = self.script_location(name);
        self.prefs.open_at(&file, line);
    }

    /// Scripts declared in the project's source, in registration order.
    pub fn script_names(&self) -> Vec<String> {
        let mut names: Vec<String> = scan_scripts(&self.project.root).into_iter().map(|(n, _)| n).collect();
        names.dedup();
        names
    }

    /// Creates a script from a template as its own file (scripts/src/<name>.rs); build.rs registers it.
    pub fn new_script(&mut self, name: &str, template: usize, _separate: bool) -> bool {
        let root = self.project.root.clone();
        let result = (|| -> Result<PathBuf, String> {
            ensure_auto(&root)?;
            check_name(&scan_scripts(&root).into_iter().map(|(n, _)| n).collect::<Vec<_>>(), name)?;
            let file = root.join("scripts/src").join(format!("{}.rs", script_templates::module_name(name)));
            if file.exists() {
                return Err(tf("File {} already exists", &[&file.file_name().unwrap().to_string_lossy()]));
            }
            let header = script_templates::MODULE_HEADER.replace("NAME", name);
            let body = script_templates::TEMPLATES[template].body.replace("NAME", name);
            std::fs::write(&file, format!("{header}{body}")).map_err(|e| e.to_string())?;
            Ok(file)
        })();
        match result {
            Ok(_) => {
                self.console.info(tf("Created script {}", &[name]));
                self.scripts.start_build(&self.project, self.console.sender());
                self.open_script(name);
                true
            }
            Err(e) => {
                self.console.error(e);
                false
            }
        }
    }

    /// Attaches a script to an object (what dropping a script on it does).
    pub fn attach_script(&mut self, e: Entity, name: &str) {
        self.begin_edit();
        let world = &mut self.active_mut().world;
        let slot = ScriptSlot { name: name.to_string(), props: Default::default() };
        match world.get_mut::<Scripts>(e) {
            Some(s) => s.0.push(slot),
            None => world.insert(e, Scripts(vec![slot])),
        }
        self.selection = Some(e);
        self.status = tf("Attached {}", &[name]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_inline_scripts_into_files() {
        let src = "use kvas_engine::prelude::*;\n\nscript_params! {\n    /// A.\n    pub struct A { x: f32 = 1.0 }\n}\nimpl Script for A {}\n\n/// B.\npub struct B;\nimpl Script for B {}\nregister_scripts!(A, B);\n";
        let parts = split_inline(src);
        assert_eq!(parts.iter().map(|p| p.0.as_str()).collect::<Vec<_>>(), ["A", "B"]);
        assert!(parts[0].1.contains("script_params!") && !parts[0].1.contains("pub struct B"));
        assert!(parts[1].1.contains("/// B.") && !parts[1].1.contains("register_scripts"));
    }

    #[test]
    fn scans_scripts_anywhere() {
        let dir = std::env::temp_dir().join(format!("kvas-scan-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("scripts/src/deep")).unwrap();
        std::fs::write(dir.join("scripts/src/deep/x.rs"), "impl Script for Door {}\n").unwrap();
        assert_eq!(scan_scripts(&dir)[0].0, "Door");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// One-off migration of the repository's sample games (kept ignored; run by hand).
    #[test]
    #[ignore]
    fn migrate_repo_games() {
        for g in ["demo", "showcase", "village"] {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games").join(g);
            assert!(ensure_auto(&root).unwrap());
        }
    }
}
