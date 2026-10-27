//! Building the project's Rust scripts with cargo and hot-loading the result.

use crate::console::Level;
use kvas_engine::project::Project;
use kvas_engine::script::ScriptHost;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime};

pub type Log = Sender<(Level, String)>;

pub fn profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

/// Directory holding the editor executable (`target/<profile>`), where the player lives too.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default()
}

/// Scripts must be built into the same target directory as the editor so shared crates are reused.
fn target_dir() -> Option<PathBuf> {
    exe_dir().ancestors().find(|d| d.file_name().is_some_and(|n| n == "debug" || n == "release")).and_then(|d| d.parent().map(|p| p.to_path_buf()))
}

/// Runs `cargo build` for the scripts crate, streaming its output to the console.
/// Returns the path of the built library.
pub fn cargo_build_scripts(project: &Project, log: &Log) -> Result<PathBuf, String> {
    cargo_build_scripts_with(project, log, profile() == "release")
}

/// Same as [`cargo_build_scripts`] with an explicit profile (game builds can choose release).
pub fn cargo_build_scripts_with(project: &Project, log: &Log, release: bool) -> Result<PathBuf, String> {
    let profile_name = if release { "release" } else { "debug" };
    let manifest = project.scripts_dir().join("Cargo.toml");
    if !manifest.exists() {
        return Err(format!("{} not found", manifest.display()));
    }
    let _ = log.send((Level::Info, format!("Building scripts ({})...", profile_name)));
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--manifest-path").arg(&manifest).arg("--message-format=short").arg("--color=never");
    if release {
        cmd.arg("--release");
    }
    let target = target_dir();
    if let Some(t) = &target {
        cmd.env("CARGO_TARGET_DIR", t);
    }
    cmd.current_dir(project.scripts_dir()).stderr(Stdio::piped()).stdout(Stdio::null());
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut cmd, 0x0800_0000); // no console window

    let mut child = cmd.spawn().map_err(|e| format!("cannot run cargo: {e}"))?;
    let stderr = child.stderr.take().ok_or("no cargo output")?;
    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
        let level = if line.contains("error") {
            Level::Error
        } else if line.contains("warning") {
            Level::Warn
        } else {
            Level::Info
        };
        let _ = log.send((level, line));
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("script build failed".into());
    }
    let built = target.unwrap_or_else(|| project.scripts_dir().join("target")).join(profile_name).join(project.script_lib_name());
    if !built.exists() {
        return Err(format!("build succeeded but {} is missing", built.display()));
    }
    let dest = lib_path(project);
    std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
    if release != (profile() == "release") {
        // The editor itself loads only libraries of its own profile; a game build keeps its copy aside.
        let aside = dest.with_file_name(format!("release-{}", project.script_lib_name()));
        std::fs::copy(&built, &aside).map_err(|e| format!("copy library: {e}"))?;
        return Ok(aside);
    }
    std::fs::copy(&built, &dest).map_err(|e| format!("copy library: {e}"))?;
    Ok(dest)
}

/// Where the editor keeps the latest built scripts library (also used by `kvas_player --project`).
pub fn lib_path(project: &Project) -> PathBuf {
    project.root.join(".kvas").join(project.script_lib_name())
}

fn newest_source_stamp(dir: &Path) -> Option<SystemTime> {
    let mut newest = std::fs::metadata(dir.join("Cargo.toml")).and_then(|m| m.modified()).ok();
    let mut stack = vec![dir.join("src")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(t) = entry.metadata().and_then(|m| m.modified()) {
                newest = newest.max(Some(t));
            }
        }
    }
    newest
}

#[derive(Default)]
pub struct ScriptState {
    /// Loaded for the inspector (names and exposed fields). The running game has its own copy.
    pub host: Option<ScriptHost>,
    pub building: Option<Receiver<Result<PathBuf, String>>>,
    pub auto_reload: bool,
    built_stamp: Option<SystemTime>,
    last_check: Option<Instant>,
    /// Set when a new library finished loading; the editor swaps it into a running game.
    pub reloaded: bool,
}

impl ScriptState {
    pub fn new() -> Self {
        Self { auto_reload: true, ..Default::default() }
    }

    pub fn names(&self) -> Vec<String> {
        self.host.as_ref().map(|h| h.names().into_iter().map(String::from).collect()).unwrap_or_default()
    }

    pub fn is_building(&self) -> bool {
        self.building.is_some()
    }

    pub fn start_build(&mut self, project: &Project, log: Log) {
        if self.building.is_some() {
            return;
        }
        let (tx, rx) = channel();
        let project = project.clone();
        self.built_stamp = newest_source_stamp(&project.scripts_dir());
        self.building = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(cargo_build_scripts(&project, &log));
        });
    }

    /// Loads an already built library if one exists (no compile).
    pub fn load_existing(&mut self, project: &Project, log: &Log) {
        let lib = lib_path(project);
        if !lib.exists() {
            return;
        }
        match ScriptHost::load(&lib) {
            Ok(h) => self.host = Some(h),
            Err(e) => {
                let _ = log.send((Level::Warn, format!("{e}")));
            }
        }
    }

    /// Call every frame: collects build results and starts auto-builds when sources change.
    pub fn poll(&mut self, project: &Project, log: &Log) {
        if let Some(rx) = &self.building {
            if let Ok(result) = rx.try_recv() {
                self.building = None;
                match result {
                    Ok(lib) => match ScriptHost::load(&lib) {
                        Ok(host) => {
                            let _ = log.send((Level::Info, format!("Scripts loaded: {}", host.names().join(", "))));
                            self.host = Some(host);
                            self.reloaded = true;
                        }
                        Err(e) => {
                            let _ = log.send((Level::Error, e));
                        }
                    },
                    Err(e) => {
                        let _ = log.send((Level::Error, e));
                    }
                }
            }
            return;
        }
        let due = self.last_check.map_or(true, |t| t.elapsed() > Duration::from_millis(400));
        if !(self.auto_reload && due) {
            return;
        }
        self.last_check = Some(Instant::now());
        let stamp = newest_source_stamp(&project.scripts_dir());
        if stamp.is_some() && stamp != self.built_stamp {
            self.start_build(project, log.clone());
        }
    }
}
