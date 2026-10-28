//! Starter library installed into every new project (and on demand into existing ones): ready-made
//! scripts sorted into folders, a set of mesh shaders and camera shaders.

use crate::script_files::{ensure_auto, scan_scripts};
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/library_files.rs"));

/// What an install did.
#[derive(Default, Debug)]
pub struct Report {
    pub scripts: usize,
    pub shaders: usize,
    pub materials: usize,
}

/// `(folder, module, source)` of every library script.
fn scripts() -> impl Iterator<Item = (&'static str, &'static str, &'static str)> {
    FILES.iter().filter_map(|(rel, src)| {
        let rest = rel.strip_prefix("scripts/")?;
        let (folder, file) = rest.split_once('/')?;
        Some((folder, file.strip_suffix(".rs")?, *src))
    })
}

fn struct_name(src: &str) -> Option<String> {
    let after = src.split("pub struct ").nth(1)?;
    Some(after.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect())
}

/// Writes the generated declaration files that stop editors from flagging engine names as undefined.
pub fn write_api_files(project_root: &Path) -> Result<(), String> {
    let dir = project_root.join("assets/shaders");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (name, text) in [("kvas_mesh_api.wgsl", kvas_render::mesh_api_stub()), ("kvas_camera_api.wgsl", kvas_render::camera_api_stub())] {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&path, text).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Copies the library's materials and shaders into a project the first time it is opened, so the
/// built-in assets are in the Project panel without a manual install. A marker file remembers it,
/// so materials the user deleted later do not come back. Scripts are not touched.
pub fn ensure_assets(project_root: &Path) -> Result<usize, String> {
    let marker = project_root.join("assets/materials/library/.kvas-library");
    if marker.exists() {
        return Ok(0);
    }
    let io = |e: std::io::Error| e.to_string();
    let mut added = 0;
    for (rel, src) in FILES.iter() {
        let target = if let Some(r) = rel.strip_prefix("shaders/") {
            project_root.join("assets/shaders/library").join(r)
        } else if let Some(r) = rel.strip_prefix("materials/") {
            project_root.join("assets/materials/library").join(r)
        } else {
            continue;
        };
        if target.exists() {
            continue;
        }
        std::fs::create_dir_all(target.parent().unwrap()).map_err(io)?;
        std::fs::write(&target, src).map_err(io)?;
        added += 1;
    }
    std::fs::create_dir_all(marker.parent().unwrap()).map_err(io)?;
    std::fs::write(&marker, "Built-in materials and shaders were copied here once. Delete this file to copy missing ones again.
").map_err(io)?;
    Ok(added)
}

/// Copies the library into a project. Existing files and already registered scripts are left alone.
pub fn install(project_root: &Path) -> Result<Report, String> {
    let io = |e: std::io::Error| e.to_string();
    let mut report = Report::default();
    write_api_files(project_root)?;

    // Shaders live in assets/shaders/library (mesh shaders) and assets/shaders/library/fx (camera shaders).
    for (rel, src) in FILES.iter().filter(|(r, _)| r.starts_with("shaders/")) {
        let target = project_root.join("assets/shaders/library").join(rel.trim_start_matches("shaders/"));
        if target.exists() {
            continue;
        }
        std::fs::create_dir_all(target.parent().unwrap()).map_err(io)?;
        std::fs::write(&target, src).map_err(io)?;
        report.shaders += 1;
    }

    // Procedural materials live in assets/materials/library/<category>.
    for (rel, src) in FILES.iter().filter(|(r, _)| r.starts_with("materials/")) {
        let target = project_root.join("assets/materials/library").join(rel.trim_start_matches("materials/"));
        if target.exists() {
            continue;
        }
        std::fs::create_dir_all(target.parent().unwrap()).map_err(io)?;
        std::fs::write(&target, src).map_err(io)?;
        report.materials += 1;
    }

    // Blender exporter for node materials.
    for (rel, src) in FILES.iter().filter(|(r, _)| r.starts_with("blender/")) {
        let target = project_root.join("tools").join(rel);
        if !target.exists() {
            std::fs::create_dir_all(target.parent().unwrap()).map_err(io)?;
            std::fs::write(&target, src).map_err(io)?;
        }
    }

    // Scripts: one file per script in a folder; scripts/build.rs registers them automatically.
    ensure_auto(project_root)?;
    let known: Vec<String> = scan_scripts(project_root).into_iter().map(|(n, _)| n).collect();
    for (folder, module, text) in scripts() {
        let Some(name) = struct_name(text) else { continue };
        let target = project_root.join("scripts/src").join(folder).join(format!("{module}.rs"));
        if known.contains(&name) || target.exists() {
            continue;
        }
        std::fs::create_dir_all(target.parent().unwrap()).map_err(io)?;
        std::fs::write(&target, text).map_err(io)?;
        report.scripts += 1;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_scripts_in_folders_and_registers_them_once() {
        let dir = std::env::temp_dir().join(format!("kvas-lib-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("scripts/src")).unwrap();
        std::fs::write(dir.join("scripts/Cargo.toml"), "[package]
name = \"x\"
").unwrap();
        std::fs::write(dir.join("scripts/src/lib.rs"), "use kvas_engine::prelude::*;

pub struct Spinner;
impl Script for Spinner {}
register_scripts!(Spinner);
").unwrap();
        let first = install(&dir).unwrap();
        assert!(first.scripts >= 15 && first.shaders >= 15 && first.materials >= 250, "{first:?}");
        let names: Vec<String> = scan_scripts(&dir).into_iter().map(|(n, _)| n).collect();
        assert!(names.contains(&"FpsPlayer".to_string()) && names.contains(&"Spinner".to_string()));
        assert_eq!(names.iter().filter(|n| *n == "Spinner").count(), 1, "the project's own Spinner is kept");
        assert!(dir.join("scripts/src/movement/fps_player.rs").exists());
        assert!(dir.join("scripts/build.rs").exists());
        let second = install(&dir).unwrap();
        assert_eq!((second.scripts, second.shaders, second.materials), (0, 0, 0), "a second install changes nothing");
        let _ = std::fs::remove_dir_all(dir);
    }
}
