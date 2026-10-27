//! Generates the test projects under `games/`: `cargo run -p kvas_engine --example make_tests`
//! They exercise grass, voxel smoke and clouds, cloth, the sky, shadows and the debug views.
//! Everything is procedural: no third-party assets are needed.

use glam::Vec3;
use kvas_engine::prelude::*;
use kvas_engine::procedural::Rng;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use kvas_engine::sky::SkySettings;
use std::path::Path;

fn script(name: &str, props: serde_json::Value) -> Scripts {
    Scripts(vec![ScriptSlot { name: name.into(), props: props.as_object().cloned().unwrap_or_default() }])
}

fn object(scene: &mut Scene, name: &str, kind: ShapeType, pos: Vec3, scale: Vec3, color: [f32; 4]) -> Entity {
    let e = hierarchy::spawn_named(&mut scene.world, name);
    scene.world.insert(e, Transform { position: pos, scale, ..Default::default() });
    scene.world.insert(e, Shape::of(kind));
    scene.world.insert(e, MeshRenderer { mesh: String::new(), color, roughness: 0.9, ..Default::default() });
    e
}

fn sun(scene: &mut Scene, pitch: f32, yaw: f32, intensity: f32) {
    let e = hierarchy::spawn_named(&mut scene.world, "Sun");
    scene.world.insert(e, Transform { position: Vec3::new(0.0, 20.0, 0.0), rotation: Vec3::new(pitch, yaw, 0.0), ..Default::default() });
    scene.world.insert(e, Light { intensity, ..Default::default() });
}

fn camera(scene: &mut Scene, pos: Vec3, rot: Vec3, scripts: Scripts) {
    let e = hierarchy::spawn_named(&mut scene.world, "Camera");
    scene.world.insert(e, Transform { position: pos, rotation: rot, ..Default::default() });
    scene.world.insert(e, Camera { far: 800.0, ..Default::default() });
    scene.world.insert(e, scripts);
}

fn ground(scene: &mut Scene, size: f32, color: [f32; 4]) {
    let e = object(scene, "Ground", ShapeType::Cube, Vec3::new(0.0, -0.5, 0.0), Vec3::new(size, 1.0, size), color);
    scene.world.insert(e, Collider::default());
}

fn tree(scene: &mut Scene, at: Vec3, height: f32) {
    let root = hierarchy::spawn_named(&mut scene.world, "Pine");
    scene.world.insert(root, Transform::at(at));
    let trunk = object(scene, "Trunk", ShapeType::Cylinder, Vec3::new(0.0, height * 0.2, 0.0), Vec3::new(height * 0.12, height * 0.4, height * 0.12), [0.28, 0.18, 0.1, 1.0]);
    hierarchy::set_parent(&mut scene.world, trunk, Some(root));
    for (i, (y, r)) in [(0.35, 0.5), (0.58, 0.4), (0.8, 0.28)].iter().enumerate() {
        let cone = object(scene, &format!("Needles {i}"), ShapeType::Cone, Vec3::new(0.0, height * y, 0.0), Vec3::new(height * r, height * 0.4, height * r), [0.07, 0.22 + i as f32 * 0.04, 0.1, 1.0]);
        hierarchy::set_parent(&mut scene.world, cone, Some(root));
    }
}

fn save(root: &Path, name: &str, scene: &Scene, scene_name: &str) {
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/main.scene.json"), scene.to_json(&Registry::default())).unwrap();
    Project { root: root.to_path_buf(), config: ProjectConfig { name: name.into(), start_scene: "scenes/main.scene.json".into(), scripts_package: format!("{scene_name}_scripts"), ..Default::default() } }.save().unwrap();
    println!("wrote {}", root.display());
}

fn meadow(root: &Path) {
    let mut scene = Scene::default();
    scene.settings.atmosphere = SkySettings { procedural: true, time: 9.5, speed: 0.0, clouds: 0.35, ..Default::default() };
    sun(&mut scene, -45.0, -30.0, 1.0);
    camera(&mut scene, Vec3::new(0.0, 1.7, 14.0), Vec3::new(-6.0, 0.0, 0.0), script("FirstPerson", serde_json::json!({ "speed": 4.0 })));
    ground(&mut scene, 120.0, [0.2, 0.16, 0.09, 1.0]);

    let lawn = hierarchy::spawn_named(&mut scene.world, "Lawn");
    scene.world.insert(lawn, Grass { count: 70_000, width: 44.0, depth: 44.0, ..Default::default() });
    let wheat = hierarchy::spawn_named(&mut scene.world, "Wheat");
    scene.world.insert(wheat, Transform::at(Vec3::new(11.0, 0.0, -6.0)));
    scene.world.insert(wheat, Grass { count: 18_000, width: 12.0, depth: 12.0, blade_height: 1.15, blade_width: 0.05, base_color: [0.3, 0.22, 0.05, 1.0], tip_color: [0.9, 0.75, 0.3, 1.0], wind_strength: 0.7, seed: 9, ..Default::default() });

    let mut rng = Rng::new(42);
    for i in 0..9 {
        let (x, z) = ((rng.f32() - 0.5) * 36.0, -4.0 - rng.f32() * 16.0);
        tree(&mut scene, Vec3::new(x, 0.0, z), 3.5 + rng.f32() * 3.0);
        let _ = i;
    }
    for i in 0..14 {
        let s = 0.3 + rng.f32() * 0.7;
        let e = object(&mut scene, &format!("Rock {i}"), ShapeType::Sphere, Vec3::new((rng.f32() - 0.5) * 30.0, s * 0.25, (rng.f32() - 0.5) * 30.0), Vec3::new(s * 1.6, s * 0.8, s * 1.2), [0.42, 0.42, 0.45, 1.0]);
        scene.world.insert(e, Collider { shape: ShapeKind::Sphere, ..Default::default() });
    }
    save(root, "Test: Meadow", &scene, "test_meadow");
}

fn voxels(root: &Path) {
    let mut scene = Scene::default();
    scene.settings.atmosphere = SkySettings { procedural: true, time: 16.8, speed: 0.12, clouds: 0.6, ..Default::default() };
    sun(&mut scene, -30.0, 40.0, 1.0);
    camera(&mut scene, Vec3::new(0.0, 4.0, 14.0), Vec3::new(-8.0, 0.0, 0.0), script("OrbitCamera", serde_json::json!({ "center": [0.0, 12.0, 0.0], "radius": 30.0, "height": 3.0, "speed": 5.0 })));
    ground(&mut scene, 200.0, [0.4, 0.44, 0.34, 1.0]);

    for (i, (x, color, rise)) in [(-6.0f32, [0.7, 0.7, 0.72, 0.9], 3.0f32), (0.0, [0.95, 0.95, 0.98, 0.85], 2.2), (6.0, [0.18, 0.17, 0.17, 0.92], 3.6)].iter().enumerate() {
        let base = object(&mut scene, &format!("Chimney {i}"), ShapeType::Cylinder, Vec3::new(*x, 0.6, 0.0), Vec3::new(0.9, 1.2, 0.9), [0.4, 0.22, 0.18, 1.0]);
        scene.world.insert(base, Collider { shape: ShapeKind::Cylinder, ..Default::default() });
        let smoke = hierarchy::spawn_named(&mut scene.world, &format!("Smoke {i}"));
        scene.world.insert(smoke, Transform::at(Vec3::new(*x, 1.2, 0.0)));
        scene.world.insert(smoke, VoxelSmoke { rise: *rise, seed: 3 + i as i32 * 5, ..Default::default() });
        scene.world.insert(smoke, MeshRenderer { color: *color, roughness: 1.0, ..Default::default() });
    }
    let low = hierarchy::spawn_named(&mut scene.world, "Clouds (low)");
    scene.world.insert(low, Transform::at(Vec3::new(0.0, 32.0, 0.0)));
    scene.world.insert(low, VoxelClouds::default());
    let high = hierarchy::spawn_named(&mut scene.world, "Clouds (high)");
    scene.world.insert(high, Transform::at(Vec3::new(40.0, 70.0, -60.0)));
    scene.world.insert(high, VoxelClouds { size: Vec3::new(300.0, 24.0, 300.0), cell: 8.0, coverage: 0.5, seed: 21, drift: Vec3::new(-2.0, 0.0, 0.5), ..Default::default() });

    let fire = object(&mut scene, "Campfire", ShapeType::Cube, Vec3::new(0.0, 0.2, 8.0), Vec3::new(0.7, 0.4, 0.7), [0.3, 0.15, 0.08, 1.0]);
    scene.world.insert(fire, MeshRenderer { emissive: 1.4, color: [1.0, 0.45, 0.12, 1.0], ..Default::default() });
    let lamp = hierarchy::spawn_named(&mut scene.world, "Fire light");
    scene.world.insert(lamp, Transform::at(Vec3::new(0.0, 1.0, 8.0)));
    scene.world.insert(lamp, Light { kind: LightKind::Point, color: [1.0, 0.5, 0.2, 1.0], intensity: 3.0, range: 14.0, ..Default::default() });
    scene.world.insert(lamp, script("Flicker", serde_json::json!({})));
    save(root, "Test: Voxel smoke and clouds", &scene, "test_voxels");
}

fn cloth(root: &Path) {
    let mut scene = Scene::default();
    scene.settings.atmosphere = SkySettings { procedural: true, time: 14.0, speed: 0.0, clouds: 0.3, ..Default::default() };
    sun(&mut scene, -50.0, 25.0, 1.0);
    camera(&mut scene, Vec3::new(0.0, 3.0, 13.0), Vec3::new(-7.0, 0.0, 0.0), script("FirstPerson", serde_json::json!({})));
    ground(&mut scene, 80.0, [0.5, 0.48, 0.42, 1.0]);

    let gust = hierarchy::spawn_named(&mut scene.world, "Wind");
    scene.world.insert(gust, script("GustController", serde_json::json!({ "base": [3.0, 0.0, 1.5], "swell": 0.7 })));

    // Flag on a pole: pinned along its left edge.
    let pole = object(&mut scene, "Flag pole", ShapeType::Cylinder, Vec3::new(-7.0, 2.6, 0.0), Vec3::new(0.12, 5.2, 0.12), [0.55, 0.55, 0.58, 1.0]);
    scene.world.insert(pole, Collider { shape: ShapeKind::Cylinder, ..Default::default() });
    let flag = hierarchy::spawn_named(&mut scene.world, "Flag");
    scene.world.insert(flag, Transform { position: Vec3::new(-6.9, 5.0, 0.0), rotation: Vec3::new(0.0, 0.0, -90.0), ..Default::default() });
    scene.world.insert(flag, Cloth { width: 2.0, height: 3.0, segments: 22, pin: ClothPin::TopEdge, ..Default::default() });
    scene.world.insert(flag, MeshRenderer { color: [0.8, 0.12, 0.1, 1.0], ..Default::default() });

    // Banner hanging from a beam, and a ball that can be dropped on it (E).
    let beam = object(&mut scene, "Beam", ShapeType::Cube, Vec3::new(-1.0, 5.1, 0.0), Vec3::new(4.2, 0.2, 0.2), [0.3, 0.2, 0.12, 1.0]);
    scene.world.insert(beam, Collider::default());
    let banner = hierarchy::spawn_named(&mut scene.world, "Banner");
    scene.world.insert(banner, Transform::at(Vec3::new(-1.0, 5.0, 0.05)));
    scene.world.insert(banner, Cloth { width: 3.8, height: 4.0, segments: 26, tear: 0.0, ..Default::default() });
    scene.world.insert(banner, MeshRenderer { color: [0.1, 0.3, 0.7, 1.0], ..Default::default() });
    scene.world.insert(banner, script("BallDropper", serde_json::json!({})));

    // Tablecloth: starts flat above a table and drapes over its edges.
    let table = object(&mut scene, "Table", ShapeType::Cube, Vec3::new(5.0, 0.5, 2.0), Vec3::new(2.0, 1.0, 2.0), [0.35, 0.22, 0.12, 1.0]);
    scene.world.insert(table, Collider::default());
    let tablecloth = hierarchy::spawn_named(&mut scene.world, "Tablecloth");
    scene.world.insert(tablecloth, Transform { position: Vec3::new(5.0, 2.6, 0.3), rotation: Vec3::new(-90.0, 0.0, 0.0), ..Default::default() });
    scene.world.insert(tablecloth, Cloth { width: 3.4, height: 3.4, segments: 30, pin: ClothPin::Free, wind: Vec3::ZERO, gust: 0.0, stiffness: 0.9, ..Default::default() });
    scene.world.insert(tablecloth, MeshRenderer { color: [0.9, 0.88, 0.8, 1.0], ..Default::default() });
    let orb = object(&mut scene, "Orb", ShapeType::Sphere, Vec3::new(5.0, 1.45, 2.0), Vec3::splat(0.9), [0.9, 0.5, 0.2, 1.0]);
    scene.world.insert(orb, Collider { shape: ShapeKind::Sphere, ..Default::default() });

    // A weak curtain that tears in strong gusts.
    let rail = object(&mut scene, "Rail", ShapeType::Cube, Vec3::new(10.0, 5.1, -2.0), Vec3::new(3.4, 0.15, 0.15), [0.3, 0.3, 0.3, 1.0]);
    scene.world.insert(rail, Collider::default());
    let curtain = hierarchy::spawn_named(&mut scene.world, "Torn curtain");
    scene.world.insert(curtain, Transform::at(Vec3::new(10.0, 5.0, -2.0)));
    scene.world.insert(curtain, Cloth { width: 3.2, height: 3.6, segments: 24, tear: 1.12, wind: Vec3::new(0.0, 0.0, 14.0), gust: 1.4, ..Default::default() });
    scene.world.insert(curtain, MeshRenderer { color: [0.55, 0.15, 0.5, 1.0], ..Default::default() });
    save(root, "Test: Cloth", &scene, "test_cloth");
}

fn main() {
    let games = std::env::args().nth(1).unwrap_or_else(|| "games".into());
    let games = Path::new(&games);
    meadow(&games.join("test_meadow"));
    voxels(&games.join("test_voxels"));
    cloth(&games.join("test_cloth"));
}
