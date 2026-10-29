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

    // At the end every door is unlocked; opened, a doorway must be free to walk through (the
    // production mesh had closed leaves in them that stood in the way as walls).
    for e in rt.scene.world.entities() {
        if let Some(s) = rt.scene.world.get_mut::<kvas_engine::components::Scripts>(e) {
            if let Some(slot) = s.0.iter_mut().find(|s| s.name == "Door") {
                slot.props.insert("locked".into(), false.into());
                slot.props.insert("open".into(), true.into());
            }
        }
    }
    for _ in 0..120 {
        rt.tick(1.0 / 30.0);
    }
    use glam::Vec3;
    let mut blocked = Vec::new();
    // (doorway, centre x, z, the wall runs along x)
    for (name, x, z, along_x) in [("entrance", 6.0, 0.0, true), ("kitchen", 6.5, -8.0, true), ("office", 9.1, -11.0, true), ("storage", 7.0, -14.05, false), ("back", 0.0, -14.0, false), ("emergency", 0.0, -10.0, false), ("yard", 0.0, -6.5, false)] {
        let across = if along_x { Vec3::Z } else { Vec3::X };
        for h in [0.4f32, 1.2, 1.8] {
            let from = Vec3::new(x, h, z) - across * 0.9;
            let hit = rt.physics.raycast(from, across, 1.8);
            let what = hit.map(|hit| (kvas_engine::hierarchy::name_of(&rt.scene.world, hit.entity).to_string(), hit.point));
            if hit.is_some() {
                blocked.push(format!("the {name} doorway is blocked at {h} m by {what:?}"));
            }
        }
    }
    assert!(blocked.is_empty(), "{}", blocked.join("
"));
}
