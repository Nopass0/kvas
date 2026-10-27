//! Project creation and game export (packed resources + player + scripts library).

use crate::console::Level;
use crate::scripting::{cargo_build_scripts_with, exe_dir, Log};
use glam::Vec3;
use kvas_engine::assets;
use kvas_engine::prelude::*;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    // Baked in at compile time: the workspace this editor was built from.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let resolved = root.canonicalize().unwrap_or(root);
    // Windows canonical paths start with \?\, which cargo rejects in manifests.
    PathBuf::from(resolved.to_string_lossy().trim_start_matches("\\\\?\\"))
}

fn identifier(name: &str) -> String {
    let id: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    if id.chars().next().is_some_and(|c| c.is_ascii_digit()) || id.is_empty() {
        format!("game_{id}")
    } else {
        id
    }
}

const SCRIPT_TEMPLATE: &str = "use kvas_engine::prelude::*;

/// Rotates the entity. Fields returned from `fields()` appear in the Inspector.
pub struct Spinner {
    speed: Vec3,
}

impl Default for Spinner {
    fn default() -> Self {
        Self { speed: Vec3::new(0.0, 90.0, 0.0) }
    }
}

impl Script for Spinner {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        ctx.transform_mut().rotation += self.speed * dt;
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::vec3(\"speed\", &mut self.speed)]
    }
}
";

/// Creates a new project folder with a starter scene and a scripts crate.
pub fn create_project(dir: &Path, name: &str, kind: &str) -> Result<Project, String> {
    if dir.join("project.json").exists() {
        return Err(format!("{} already contains a project", dir.display()));
    }
    let io = |e: std::io::Error| e.to_string();
    for sub in ["assets/scenes", "assets/models", "assets/textures", "assets/shaders", "scripts/src"] {
        std::fs::create_dir_all(dir.join(sub)).map_err(io)?;
    }
    let package = format!("{}_scripts", identifier(name));
    let project = Project {
        root: dir.to_path_buf(),
        config: ProjectConfig {
            name: name.into(),
            start_scene: "scenes/main.scene.json".into(),
            scripts_package: package.clone(),
            kind: kind.into(),
            ..Default::default()
        },
    };
    project.save().map_err(io)?;

    let engine = engine_root().join("crates/kvas_engine");
    let cargo = format!(
        "[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nkvas_engine = {{ path = {:?} }}\n\n[workspace]\n",
        engine.to_string_lossy().replace('\\', "/")
    );
    std::fs::write(dir.join("scripts/Cargo.toml"), cargo).map_err(io)?;
    std::fs::write(dir.join("scripts/src/lib.rs"), crate::script_files::LIB_RS).map_err(io)?;
    std::fs::write(dir.join("scripts/build.rs"), crate::script_files::BUILD_RS).map_err(io)?;
    std::fs::write(dir.join("scripts/src/spinner.rs"), SCRIPT_TEMPLATE).map_err(io)?;
    // Same dependency versions as the editor, so engine types have identical layouts.
    let _ = std::fs::copy(engine_root().join("Cargo.lock"), dir.join("scripts/Cargo.lock"));

    let mut scene = Scene::default();
    let w = &mut scene.world;
    let cam = hierarchy::spawn_named(w, "Main Camera");
    if kind == "2D" {
        // Side-on orthographic view: objects live in the XY plane.
        w.insert(cam, Transform::at(Vec3::new(0.0, 3.0, 20.0)));
        w.insert(cam, Camera { orthographic: true, ortho_size: 12.0, ..Default::default() });
    } else {
        w.insert(cam, Transform { position: Vec3::new(0.0, 3.0, 8.0), rotation: Vec3::new(-15.0, 0.0, 0.0), ..Default::default() });
        w.insert(cam, Camera::default());
    }
    let sun = hierarchy::spawn_named(w, "Sun");
    w.insert(sun, Transform { position: Vec3::new(0.0, 10.0, 0.0), rotation: Vec3::new(-50.0, -30.0, 0.0), ..Default::default() });
    w.insert(sun, Light::default());
    let ground = hierarchy::spawn_named(w, "Ground");
    w.insert(ground, Transform { position: Vec3::new(0.0, -0.5, 0.0), scale: Vec3::new(30.0, 1.0, 30.0), ..Default::default() });
    w.insert(ground, MeshRenderer { color: [0.55, 0.6, 0.55, 1.0], ..Default::default() });
    w.insert(ground, Collider::default());
    let cube = hierarchy::spawn_named(w, "Cube");
    w.insert(cube, Transform::at(Vec3::new(0.0, 3.0, 0.0)));
    w.insert(cube, MeshRenderer { color: [0.9, 0.5, 0.2, 1.0], ..Default::default() });
    w.insert(cube, RigidBody::default());
    w.insert(cube, Collider::default());
    scene.save(&Registry::default(), &project.assets_dir().join("scenes/main.scene.json"))?;
    // Starter scripts and shaders, ready to drag onto objects.
    crate::library::install(dir)?;
    Ok(project)
}

fn say(log: &Log, level: Level, msg: impl Into<String>) {
    let _ = log.send((level, msg.into()));
}

/// Builds `<project>/build/`: the player renamed to the game, `game.pak` with all assets, and the scripts library.
pub fn build_game(project: &Project, log: &Log) -> Result<PathBuf, String> {
    let io = |what: &str| {
        let what = what.to_string();
        move |e: std::io::Error| format!("{what}: {e}")
    };
    let settings = project.config.build.clone();
    let profile = if settings.release { "release" } else { "debug" };
    let player_name = if cfg!(windows) { "kvas_player.exe" } else { "kvas_player" };
    // The player is built per profile next to the editor's own target folder; cargo skips it when current.
    let target = exe_dir()
        .ancestors()
        .find(|d| d.file_name().is_some_and(|n| n == "debug" || n == "release"))
        .and_then(|d| d.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| engine_root().join("target"));
    let player = target.join(profile).join(player_name);
    let own_profile = if cfg!(debug_assertions) { "debug" } else { "release" };
    if !player.exists() || profile != own_profile {
        say(log, Level::Info, format!("Building the player ({profile})..."));
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(["build", "-p", "kvas_player"]).current_dir(engine_root());
        if settings.release {
            cmd.arg("--release");
        }
        let ok = cmd.status().map(|s| s.success()).unwrap_or(false);
        if !ok || !player.exists() {
            return Err(format!("{} not found and building it failed. Run: cargo build -p kvas_player{}", player.display(), if settings.release { " --release" } else { "" }));
        }
    }

    crate::script_files::ensure_auto(&project.root)?;
    let lib = cargo_build_scripts_with(project, log, settings.release)?;
    let out = project.root.join("build");
    std::fs::create_dir_all(&out).map_err(io("create build folder"))?;

    let config = serde_json::to_vec_pretty(&project.config).map_err(|e| e.to_string())?;
    let count = assets::pack(&project.assets_dir(), &out.join("game.pak"), &[("project.json", config)]).map_err(io("pack assets"))?;
    say(log, Level::Info, format!("Packed {count} files into game.pak"));

    let base = if settings.exe_name.trim().is_empty() { identifier(&project.config.name) } else { identifier(settings.exe_name.trim()) };
    let exe_name = format!("{base}{}", if cfg!(windows) { ".exe" } else { "" });
    std::fs::copy(&player, out.join(&exe_name)).map_err(io("copy player"))?;
    let icon = game_icon(project);
    // Antivirus scanners sometimes hold a freshly copied exe for a moment, so retry a few times.
    let mut embedded = set_exe_icon(&out.join(&exe_name), &icon);
    for _ in 0..4 {
        if embedded.is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
        embedded = set_exe_icon(&out.join(&exe_name), &icon);
    }
    match embedded {
        Ok(()) => say(log, Level::Info, "Application icon embedded"),
        Err(e) => say(log, Level::Warn, format!("Could not embed the icon: {e}")),
    }
    // The window / taskbar icon is read from icon.png beside the executable.
    let mut png = Vec::new();
    if icon.resize_exact(256, 256, image::imageops::FilterType::Lanczos3).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).is_ok() {
        let _ = std::fs::write(out.join("icon.png"), png);
    }
    std::fs::copy(&lib, out.join(project.script_lib_name())).map_err(io("copy scripts"))?;
    let readme = format!(
        "{} {}
{}
Run {exe_name}. Keep game.pak, {} and icon.png in the same folder.
",
        project.config.name,
        settings.version,
        settings.company,
        project.script_lib_name()
    );
    let _ = std::fs::write(out.join("README.txt"), readme);
    verify_build(&out, &exe_name, log);
    say(log, Level::Info, format!("Game built: {}", out.join(&exe_name).display()));
    Ok(out)
}

/// Starts the built game for a few seconds and reports whether it stayed up (a smoke test of the standalone build).
fn verify_build(out: &Path, exe_name: &str, log: &Log) {
    let mut cmd = std::process::Command::new(out.join(exe_name));
    cmd.current_dir(out).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    let Ok(mut child) = cmd.spawn() else {
        say(log, Level::Warn, "The built game could not be started");
        return;
    };
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(250));
        if let Ok(Some(status)) = child.try_wait() {
            say(log, Level::Error, format!("The built game exited right away ({status})"));
            return;
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    say(log, Level::Info, "Smoke test passed: the built game starts and keeps running");
}

/// The icon of the built game: `icon.png` (or `icon.ico`) in the project folder, else the Kvas logo.
fn game_icon(project: &Project) -> image::DynamicImage {
    let chosen = project.config.build.icon.trim();
    if !chosen.is_empty() {
        let path = if Path::new(chosen).is_absolute() { PathBuf::from(chosen) } else { project.root.join(chosen) };
        if let Ok(img) = image::open(&path) {
            return img;
        }
    }
    for name in ["icon.png", "icon.ico", "assets/icon.png"] {
        if let Ok(img) = image::open(project.root.join(name)) {
            return img;
        }
    }
    image::load_from_memory(include_bytes!("../assets/logo-256.png")).expect("bundled logo")
}

/// PNG-compressed icon frames (size, bytes) for a Windows icon resource.
fn icon_frames(img: &image::DynamicImage) -> Vec<(u32, Vec<u8>)> {
    [256u32, 128, 64, 48, 32, 16]
        .iter()
        .filter_map(|&size| {
            let frame = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
            let mut bytes = Vec::new();
            frame.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).ok()?;
            Some((size, bytes))
        })
        .collect()
}

/// Embeds an icon into a Windows executable (no-op elsewhere).
#[cfg(windows)]
pub fn set_exe_icon(exe: &Path, img: &image::DynamicImage) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::LibraryLoader::{BeginUpdateResourceW, EndUpdateResourceW, UpdateResourceW};
    let wide: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let frames = icon_frames(img);
    // GRPICONDIR: reserved, type 1, count; then one 14-byte entry per frame pointing at its RT_ICON id.
    let mut group = vec![0u8, 0, 1, 0];
    group.extend((frames.len() as u16).to_le_bytes());
    for (i, (size, bytes)) in frames.iter().enumerate() {
        let dim = if *size >= 256 { 0u8 } else { *size as u8 };
        group.extend([dim, dim, 0, 0]);
        group.extend(1u16.to_le_bytes());
        group.extend(32u16.to_le_bytes());
        group.extend((bytes.len() as u32).to_le_bytes());
        group.extend((i as u16 + 1).to_le_bytes());
    }
    // SAFETY: plain Win32 resource-update calls on a file we just copied; handles are closed below.
    unsafe {
        let handle = BeginUpdateResourceW(wide.as_ptr(), 0);
        if handle.is_null() {
            return Err("BeginUpdateResource failed (is the exe running?)".into());
        }
        let make_id = |id: u16| id as usize as *const u16;
        let mut ok = true;
        for (i, (_, bytes)) in frames.iter().enumerate() {
            ok &= UpdateResourceW(handle, make_id(3), make_id(i as u16 + 1), 0, bytes.as_ptr() as *const _, bytes.len() as u32) != 0;
        }
        ok &= UpdateResourceW(handle, make_id(14), make_id(1), 0, group.as_ptr() as *const _, group.len() as u32) != 0;
        let code_update = windows_sys::Win32::Foundation::GetLastError();
        let committed = EndUpdateResourceW(handle, if ok { 0 } else { 1 }) != 0;
        let code_end = windows_sys::Win32::Foundation::GetLastError();
        if !(ok && committed) {
            return Err(format!("UpdateResource failed (update ok = {ok}, error {code_update}; commit ok = {committed}, error {code_end})"));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn set_exe_icon(_exe: &Path, _img: &image::DynamicImage) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod build_tests {
    use super::*;

    /// Builds the village sample into a standalone folder and smoke-tests it (slow; run by hand).
    #[test]
    #[ignore]
    fn build_village_standalone() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/village");
        let project = Project::open(&root).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let out = build_game(&project, &tx);
        while let Ok((level, line)) = rx.try_recv() {
            let _ = level; eprintln!("{line}");
        }
        let out = out.unwrap();
        assert!(out.join("game.pak").exists());
        assert!(std::fs::read_dir(&out).unwrap().flatten().any(|e| e.path().extension().is_some_and(|x| x == "exe")));
    }
}
