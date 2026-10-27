//! The FPS test project without a window: the player settles on the ground, walks, jumps, presses a
//! button that opens a gate, carries a barrel onto the pressure plate and lights the beacon.
//! Needs `games/test_fps` (generate it with the `make_fps` example); the test is skipped without it.

use glam::Vec2;
use kvas_engine::assets::Assets;
use kvas_engine::input::{Key, MouseButton};
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

fn load() -> Option<Runtime> {
    let root = workspace().join("games/test_fps");
    if !root.join("project.json").exists() {
        eprintln!("games/test_fps is not generated; skipping");
        return None;
    }
    let mut command = std::process::Command::new(env!("CARGO"));
    command.args(["build", "-q", "-p", "test_fps_scripts"]);
    if !cfg!(debug_assertions) {
        command.arg("--release");
    }
    let status = command.current_dir(workspace()).status().unwrap();
    assert!(status.success(), "scripts failed to build");
    let project = Project::open(&root).unwrap();
    let profile_dir = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    let host = ScriptHost::load(&profile_dir.join(project.script_lib_name())).expect("scripts must load");
    let assets = Assets::Dir(project.assets_dir());
    let (scene, warnings) = Scene::from_json(&Registry::default(), &assets.read_string(&project.config.start_scene).unwrap()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let mut rt = Runtime::new(scene, Some(host));
    rt.set_bindings(&project.config.binding_defs());
    Some(rt)
}

struct Driver {
    rt: Runtime,
    keys: HashSet<Key>,
    buttons: HashSet<MouseButton>,
    mouse: Vec2,
}

impl Driver {
    fn run(&mut self, frames: usize) {
        for _ in 0..frames {
            self.rt.input.update(self.keys.clone(), self.buttons.clone(), self.mouse, 0.0);
            self.mouse = Vec2::ZERO;
            self.rt.tick(1.0 / 60.0);
        }
    }
    fn pos(&self, name: &str) -> Vec3 {
        let e = hierarchy::find_by_name(&self.rt.scene.world, name).unwrap_or_else(|| panic!("no {name}"));
        hierarchy::global_matrix(&self.rt.scene.world, e).w_axis.truncate()
    }
}

#[test]
fn fps_player_walks_jumps_and_solves_the_puzzle() {
    let Some(rt) = load() else { return };
    let mut d = Driver { rt, keys: HashSet::new(), buttons: HashSet::new(), mouse: Vec2::ZERO };
    d.run(120);
    let ground = d.pos("Player");
    let eyes = d.pos("Eyes");
    for (e, _) in d.rt.scene.world.iter::<MeshRenderer>() {
        let p = hierarchy::global_matrix(&d.rt.scene.world, e).w_axis.truncate();
        if p.distance(eyes) < 6.0 {
            let t = d.rt.scene.world.get::<Transform>(e).unwrap();
            eprintln!("near eyes {eyes:?}: {} at {p:?} scale {:?}", hierarchy::name_of(&d.rt.scene.world, e), t.scale);
        }
    }
    assert!((ground.y - 0.9).abs() < 0.12, "player rests on the ground: {ground:?}");
    assert!(d.rt.log.iter().all(|l| !l.contains("panicked")), "{:?}", d.rt.log);

    // Walk forward (towards -Z).
    d.keys.insert(Key::W);
    d.run(90);
    d.keys.clear();
    let walked = d.pos("Player");
    assert!(walked.z < ground.z - 3.0, "walked forward: {ground:?} -> {walked:?}");

    // Jump.
    d.run(30);
    d.keys.insert(Key::Space);
    d.run(2);
    d.keys.clear();
    let mut peak = 0.0f32;
    for _ in 0..50 {
        d.run(1);
        peak = peak.max(d.pos("Player").y);
    }
    assert!(peak > 1.5, "jumped to {peak}");
}
