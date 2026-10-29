//! LAST SERVICE plays itself from the menu to the credits: every story step is reached by the
//! player's own interaction ray. Skipped when the game's assets were not imported
//! (`python tools/import_last_service.py`).

use kvas_engine::assets::Assets;
use kvas_engine::project::Project;
use kvas_engine::registry::Registry;
use kvas_engine::runtime::Runtime;
use kvas_engine::scene::Scene;
use kvas_engine::script::ScriptHost;
use std::path::PathBuf;

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

#[test]
fn last_service_story_can_be_finished() {
    let root = workspace().join("games/last_service");
    if !root.join("assets/world/parts/index.json").exists() {
        eprintln!("LAST SERVICE assets are not imported; skipping");
        return;
    }
    let mut command = std::process::Command::new(env!("CARGO"));
    command.args(["build", "-p", "last_service_scripts"]);
    if !cfg!(debug_assertions) {
        command.arg("--release");
    }
    assert!(command.current_dir(workspace()).status().unwrap().success(), "scripts failed to build");
    let project = Project::open(&root).unwrap();
    let profile_dir = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    let host = ScriptHost::load(&profile_dir.join(project.script_lib_name())).expect("scripts load");
    let assets = Assets::Dir(project.assets_dir());
    // Mesh colliders read the models.
    kvas_engine::meshmod::set_assets(assets.clone());
    let (mut scene, warnings) = Scene::from_json(&Registry::default(), &assets.read_string(&project.config.start_scene).unwrap()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    scene.globals.insert("ls_autotest".into(), serde_json::json!(true));
    let mut rt = Runtime::new(scene, Some(host));
    rt.set_bindings(&project.config.binding_defs());
    let data = std::env::temp_dir().join(format!("kvas-last-service-{}", std::process::id()));
    rt.input.set_storage(kvas_engine::storage::Storage::new(&data));
    rt.input.set_scene_path(&project.config.start_scene);
    let mut log = Vec::new();
    for _ in 0..3000 {
        rt.tick(1.0 / 30.0);
        kvas_engine::procedural::update(&mut rt.scene.world, rt.time.elapsed, 1.0 / 30.0, true);
        log.extend(rt.drain_log());
        if log.iter().any(|l| l.contains("AUTOTEST PASS") || l.contains("AUTOTEST FAIL")) {
            break;
        }
    }
    let _ = std::fs::remove_dir_all(&data);
    let text = log.join("\n");
    assert!(text.contains("AUTOTEST PASS"), "{text}");
    assert!(!text.contains("panicked"), "{text}");
}
