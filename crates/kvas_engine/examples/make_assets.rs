//! Builds `games/test_assets` from the Kenney "Retro Medieval Kit" GLB models in `.test_assets`:
//! `cargo run -p kvas_engine --example make_assets [source dir] [games dir]`.
//! Only the models the scene uses (plus the shared texture folder) are copied into the project.

use glam::Vec3;
use kvas_engine::prelude::*;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use kvas_engine::sky::SkySettings;
use std::collections::BTreeSet;
use std::path::Path;

/// One kit tile is this many metres.
const K: f32 = 3.0;

struct Builder {
    scene: Scene,
    used: BTreeSet<String>,
    root: Entity,
}

impl Builder {
    fn put(&mut self, model: &str, gx: f32, gy: f32, gz: f32, yaw: f32) -> Entity {
        self.used.insert(model.to_string());
        let e = hierarchy::spawn_named(&mut self.scene.world, model);
        self.scene.world.insert(e, Transform { position: Vec3::new(gx * K, gy * K, gz * K), rotation: Vec3::new(0.0, yaw, 0.0), scale: Vec3::splat(K) });
        self.scene.world.insert(e, MeshRenderer { mesh: format!("models/{model}.glb"), color: [1.0; 4], roughness: 0.9, ..Default::default() });
        hierarchy::set_parent(&mut self.scene.world, e, Some(self.root));
        e
    }

    fn prop(&mut self, model: &str, x: f32, z: f32, yaw: f32, scale: f32) -> Entity {
        self.used.insert(model.to_string());
        let e = hierarchy::spawn_named(&mut self.scene.world, model);
        self.scene.world.insert(e, Transform { position: Vec3::new(x, 0.0, z), rotation: Vec3::new(0.0, yaw, 0.0), scale: Vec3::splat(scale) });
        // Cut-out foliage and fences are flat planes: their shadows would be rectangles.
        let cutout = model.starts_with("tree") || model.starts_with("fence");
        self.scene.world.insert(e, MeshRenderer { mesh: format!("models/{model}.glb"), color: [1.0; 4], roughness: 0.9, cast_shadows: !cutout, ..Default::default() });
        e
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let source = args.next().unwrap_or_else(|| ".test_assets/Models/GLB format".into());
    let games = args.next().unwrap_or_else(|| "games".into());
    let root_dir = Path::new(&games).join("test_assets");

    let mut scene = Scene::default();
    scene.settings.atmosphere = SkySettings { procedural: true, time: 17.3, speed: 0.0, clouds: 0.45, ..Default::default() };
    let root = hierarchy::spawn_named(&mut scene.world, "Castle");
    let mut b = Builder { scene, used: BTreeSet::new(), root };

    // Sun, camera, ground.
    let sun = hierarchy::spawn_named(&mut b.scene.world, "Sun");
    b.scene.world.insert(sun, Transform { position: Vec3::new(0.0, 30.0, 0.0), rotation: Vec3::new(-35.0, 35.0, 0.0), ..Default::default() });
    b.scene.world.insert(sun, Light { intensity: 1.0, ..Default::default() });
    let cam = hierarchy::spawn_named(&mut b.scene.world, "Camera");
    b.scene.world.insert(cam, Transform::at(Vec3::new(0.0, 12.0, 40.0)));
    b.scene.world.insert(cam, Camera { far: 900.0, ..Default::default() });
    b.scene.world.insert(cam, Scripts(vec![ScriptSlot { name: "OrbitCamera".into(), props: serde_json::json!({ "center": [0.0, 5.0, 0.0], "radius": 42.0, "height": 14.0, "speed": 4.0 }).as_object().cloned().unwrap() }]));
    let ground = hierarchy::spawn_named(&mut b.scene.world, "Ground");
    b.scene.world.insert(ground, Transform { position: Vec3::new(0.0, -0.5, 0.0), scale: Vec3::new(300.0, 1.0, 300.0), ..Default::default() });
    b.scene.world.insert(ground, Shape::of(ShapeType::Cube));
    b.scene.world.insert(ground, MeshRenderer { mesh: String::new(), color: [0.18, 0.15, 0.1, 1.0], roughness: 1.0, ..Default::default() });
    let meadow = hierarchy::spawn_named(&mut b.scene.world, "Meadow");
    b.scene.world.insert(meadow, Grass { count: 90_000, width: 90.0, depth: 90.0, ..Default::default() });
    let clouds = hierarchy::spawn_named(&mut b.scene.world, "Clouds");
    b.scene.world.insert(clouds, Transform::at(Vec3::new(0.0, 60.0, 0.0)));
    b.scene.world.insert(clouds, VoxelClouds::default());

    // Cobbled courtyard platform that also hides the grass inside the walls.
    b.used.insert("floor".into());
    let platform = hierarchy::spawn_named(&mut b.scene.world, "Courtyard");
    b.scene.world.insert(platform, Transform { position: Vec3::new(0.0, 0.0, 0.0), scale: Vec3::new(K * 9.0, 14.0, K * 9.0), ..Default::default() });
    b.scene.world.insert(platform, MeshRenderer { mesh: "models/floor.glb".into(), color: [1.0; 4], roughness: 1.0, tiling: 1.0, ..Default::default() });

    // Perimeter: fortified walls with a gate, a tower on every corner.
    let n = 4.0;
    for i in -3..=3 {
        let i = i as f32;
        let front = if i == 0.0 { "wall-fortified-gate" } else { "wall-fortified" };
        b.put(front, i, 0.2, n, 0.0);
        b.put("wall-fortified", i, 0.2, -n, 180.0);
        b.put("wall-fortified", -n, 0.2, i, 90.0);
        b.put("wall-fortified", n, 0.2, i, -90.0);
        for (x, z) in [(i, n), (i, -n), (-n, i), (n, i)] {
            b.put("battlement", x, 1.2, z, if x.abs() == n { 90.0 } else { 0.0 });
        }
    }
    for (x, z) in [(-n, -n), (n, -n), (-n, n), (n, n)] {
        b.put("tower-base", x, 0.2, z, 0.0);
        b.put("tower", x, 1.2, z, 0.0);
        b.put("tower-top", x, 2.2, z, 0.0);
    }

    // Two little houses and a keep inside.
    for (x, z, yaw) in [(-2.0, -1.5, 0.0), (2.0, -1.5, 90.0)] {
        b.put("wall-paint-door", x, 0.2, z, yaw);
        b.put("wall-paint-window", x, 1.2, z, yaw);
        b.put("roof", x, 2.2, z, yaw);
    }
    b.put("structure-wall", 0.0, 0.2, -2.5, 0.0);
    b.put("structure", 0.0, 1.2, -2.5, 0.0);
    b.put("roof-high-side", 0.0, 2.2, -2.5, 0.0);
    for z in [-0.5, 0.5, 1.5] {
        b.put("column", -1.2, 0.2, z, 0.0);
        b.put("column", 1.2, 0.2, z, 0.0);
    }

    // Props: barrels, crates, trees, fences outside.
    let courtyard = [(-3.2, 4.0), (3.4, 4.5), (-4.4, -5.5), (4.6, -4.2), (0.8, 6.5), (-1.0, 7.0)];
    for (i, &(x, z)) in courtyard.iter().enumerate() {
        let e = b.prop(if i % 2 == 0 { "barrels" } else { "detail-crate-ropes" }, x, z, i as f32 * 53.0, 2.0);
        b.scene.world.get_mut::<Transform>(e).unwrap().position.y = 0.6;
        b.scene.world.insert(e, Collider { size: Vec3::splat(0.5), ..Default::default() });
    }
    for (i, &(x, z)) in [(-26.0, 20.0), (-31.0, 8.0), (28.0, 16.0), (33.0, -4.0), (-24.0, -26.0), (20.0, -30.0), (-34.0, -10.0), (8.0, 34.0)].iter().enumerate() {
        b.prop(if i % 3 == 0 { "tree-shrub" } else { "tree-large" }, x, z, i as f32 * 41.0, 7.0 + (i % 3) as f32 * 1.5);
    }
    for i in -5..=5 {
        let x = i as f32 * 1.0 * K * 0.9;
        b.prop("fence", x, 22.0, 0.0, K);
    }

    // Write the project.
    let models_out = root_dir.join("assets/models");
    std::fs::create_dir_all(&models_out).unwrap();
    let source = Path::new(&source);
    for name in &b.used {
        let from = source.join(format!("{name}.glb"));
        std::fs::copy(&from, models_out.join(format!("{name}.glb"))).unwrap_or_else(|e| panic!("{}: {e}", from.display()));
    }
    let textures = models_out.join("Textures");
    std::fs::create_dir_all(&textures).unwrap();
    for entry in std::fs::read_dir(source.join("Textures")).unwrap().flatten() {
        std::fs::copy(entry.path(), textures.join(entry.file_name())).unwrap();
    }

    std::fs::create_dir_all(root_dir.join("assets/scenes")).unwrap();
    std::fs::write(root_dir.join("assets/scenes/main.scene.json"), b.scene.to_json(&Registry::default())).unwrap();
    std::fs::create_dir_all(root_dir.join("scripts/src")).unwrap();
    std::fs::write(root_dir.join("scripts/Cargo.toml"), "[package]\nname = \"test_assets_scripts\"\nversion.workspace = true\nedition.workspace = true\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nkvas_engine.workspace = true\n").unwrap();
    std::fs::write(root_dir.join("scripts/src/lib.rs"), "include!(\"../../../test_shared/scripts.rs\");\n").unwrap();
    Project { root: root_dir.clone(), config: ProjectConfig { name: "Test: Medieval kit".into(), start_scene: "scenes/main.scene.json".into(), scripts_package: "test_assets_scripts".into(), ..Default::default() } }.save().unwrap();
    println!("wrote {} ({} models)", root_dir.display(), b.used.len());
}
