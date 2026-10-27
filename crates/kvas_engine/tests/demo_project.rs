//! End to end without a window: build the demo scripts, hot-load the dll, run the real demo scene.

use glam::Vec2;
use kvas_engine::assets::Assets;
use kvas_engine::input::Key;
use kvas_engine::prelude::*;
use kvas_engine::project::Project;
use kvas_engine::registry::Registry;
use kvas_engine::runtime::Runtime;
use kvas_engine::scene::Scene;
use kvas_engine::script::ScriptHost;
use std::collections::HashSet;
use std::path::PathBuf;

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn build_scripts() -> PathBuf {
    static BUILT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BUILT
        .get_or_init(|| {
            let mut command = std::process::Command::new(env!("CARGO"));
            command.args(["build", "-p", "demo_scripts"]);
            if !cfg!(debug_assertions) {
                command.arg("--release");
            }
            let status = command.current_dir(workspace()).status().unwrap();
            assert!(status.success(), "scripts failed to build");
            let project = Project::open(&workspace().join("games/demo")).unwrap();
            // The test binary lives in target/<profile>/deps, the library in target/<profile>.
            let profile_dir = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
            profile_dir.join(project.script_lib_name())
        })
        .clone()
}

fn load_demo() -> (Scene, ScriptHost) {
    let project = Project::open(&workspace().join("games/demo")).unwrap();
    let assets = Assets::Dir(project.assets_dir());
    let text = assets.read_string(&project.config.start_scene).unwrap();
    let (scene, warnings) = Scene::from_json(&Registry::default(), &text).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    (scene, ScriptHost::load(&build_scripts()).expect("scripts must load and pass the ABI check"))
}

#[test]
fn demo_scene_runs_with_rust_scripts() {
    let (scene, host) = load_demo();
    let names = host.names();
    for expected in ["Spinner", "Mover", "Flicker", "PlayerBall", "FollowCamera", "BoxSpawner"] {
        assert!(names.contains(&expected), "missing script {expected}: {names:?}");
    }
    let player = hierarchy::find_by_name(&scene.world, "Player").unwrap();
    let torus = hierarchy::find_by_name(&scene.world, "Torus").unwrap();
    let start = scene.world.get::<Transform>(player).unwrap().position;
    let mut rt = Runtime::new(scene, Some(host));
    rt.set_bindings(&Project::open(&workspace().join("games/demo")).unwrap().config.binding_defs());

    // Hold D (move right) for two seconds, tap E once to spawn a box.
    for frame in 0..180 {
        let mut keys = HashSet::from([Key::D]);
        if frame == 10 {
            keys.insert(Key::E);
        }
        rt.input.update(keys, HashSet::new(), Vec2::ZERO, 0.0);
        rt.tick(1.0 / 60.0);
    }

    assert!(rt.log.iter().all(|l| !l.contains("panicked") && !l.contains("not found")), "script problems: {:?}", rt.log);
    let end = rt.scene.world.get::<Transform>(player).unwrap().position;
    assert!(end.x - start.x > 1.0, "PlayerBall script should have rolled the ball right: {start} -> {end}");
    assert!(end.y > -0.1 && end.y < 3.0, "the ball must stay on the ground, y = {}", end.y);
    assert!(rt.scene.world.get::<Transform>(torus).unwrap().rotation.y.abs() > 1.0, "Spinner should rotate the torus");
    assert!(hierarchy::find_by_name(&rt.scene.world, "Spawned Box 1").is_some(), "BoxSpawner should have spawned a box");
    let boxed = hierarchy::find_by_name(&rt.scene.world, "Spawned Box 1").unwrap();
    assert!(rt.scene.world.get::<Transform>(boxed).unwrap().position.y < 9.9, "spawned box must fall (physics picks up runtime entities)");

    // Pendulum constraint holds: the weight stays 2.8 from its pivot at (9.2, 8, 0).
    let weight = hierarchy::find_by_name(&rt.scene.world, "Pendulum Weight").unwrap();
    let p = rt.scene.world.get::<Transform>(weight).unwrap().position;
    assert!(((p - Vec3::new(9.2, 8.0, 0.0)).length() - 2.8).abs() < 0.15, "pendulum arm length drifted: {p}");
}

#[test]
fn scripts_can_be_hot_reloaded_and_properties_are_applied() {
    let (scene, host) = load_demo();
    let mut rt = Runtime::new(scene, Some(host));
    rt.tick(1.0 / 60.0);
    let cam = hierarchy::find_by_name(&rt.scene.world, "Main Camera").unwrap();
    let before = rt.scene.world.get::<Transform>(cam).unwrap().position;

    // Swap in a freshly loaded copy of the library while "playing".
    rt.scripts.set_host(Some(ScriptHost::load(&build_scripts()).unwrap()));
    for _ in 0..120 {
        rt.tick(1.0 / 60.0);
    }
    assert!(rt.log.iter().all(|l| !l.contains("panicked")), "{:?}", rt.log);
    let after = rt.scene.world.get::<Transform>(cam).unwrap().position;
    assert!((after - before).length() > 0.01, "FollowCamera should keep working after reload");

    // A property stored in the scene reaches the script instance via `fields()`.
    let host = rt.scripts.host().unwrap();
    let mut spinner = host.create("Spinner").unwrap();
    let mut props = serde_json::Map::new();
    props.insert("speed".into(), serde_json::json!([1.0, 2.0, 3.0]));
    kvas_engine::reflect::apply_props(&mut spinner.fields(), &props);
    assert_eq!(spinner.fields()[0].to_json(), serde_json::json!([1.0, 2.0, 3.0]));
}

#[test]
fn scripts_keep_their_native_code_alive_after_the_host_is_dropped() {
    let host = ScriptHost::load(&build_scripts()).unwrap();
    let names = host.names_owned();
    let mut spinner = host.create("Spinner").unwrap();
    drop(host);
    assert!(names.iter().any(|name| name == "Spinner"));
    assert!(!spinner.fields().is_empty());
    let mut properties = serde_json::Map::new();
    properties.insert("speed".into(), serde_json::json!([9.0, 8.0, 7.0]));
    kvas_engine::reflect::apply_props(&mut spinner.fields(), &properties);
    assert_eq!(spinner.fields()[0].to_json(), serde_json::json!([9.0, 8.0, 7.0]));
    drop(spinner); // destructor must also execute while the native module is loaded.
}

#[test]
fn removing_script_slots_stops_their_callbacks_without_despawning_entities() {
    let host = ScriptHost::load(&build_scripts()).unwrap();
    let mut scene = Scene::default();
    let entity = scene.spawn("scripted");
    scene.world.insert(entity, Scripts(vec![ScriptSlot { name: "Spinner".into(), ..Default::default() }]));
    let mut runtime = Runtime::new(scene, Some(host));
    runtime.tick(1.0 / 60.0);
    let before = runtime.scene.world.get::<Transform>(entity).unwrap().rotation;
    assert!(before.length() > 0.0);
    runtime.scene.world.remove::<Scripts>(entity);
    for _ in 0..5 {
        runtime.tick(1.0 / 60.0);
    }
    assert_eq!(runtime.scene.world.get::<Transform>(entity).unwrap().rotation, before);
    assert!(runtime.scene.world.is_alive(entity));
}
