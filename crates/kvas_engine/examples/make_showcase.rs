//! Builds the "Treasure Isle" showcase project in `games/showcase`:
//! `cargo run -p kvas_engine --example make_showcase [kit dir] [games dir]`.
//!
//! A small island with a ruined tower, a dock, voxel water, grass, volumetric clouds, eight gems
//! to collect, spatial sea ambience and music. Models come from the Kenney Retro Medieval Kit.

use glam::Vec3;
use kvas_engine::prelude::*;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use kvas_engine::sky::SkySettings;
use std::collections::BTreeSet;
use std::path::Path;

const K: f32 = 3.0;

struct Builder {
    scene: Scene,
    used: BTreeSet<String>,
}

impl Builder {
    fn entity(&mut self, name: &str) -> Entity {
        hierarchy::spawn_named(&mut self.scene.world, name)
    }

    fn model(&mut self, model: &str, pos: Vec3, yaw: f32, scale: f32) -> Entity {
        self.used.insert(model.to_string());
        let e = self.entity(model);
        let w = &mut self.scene.world;
        w.insert(e, Transform { position: pos, rotation: Vec3::new(0.0, yaw, 0.0), scale: Vec3::splat(scale) });
        let cutout = model.starts_with("tree") || model.starts_with("fence");
        w.insert(e, MeshRenderer { mesh: format!("models/{model}.glb"), color: [1.0; 4], roughness: 0.9, cast_shadows: !cutout, ..Default::default() });
        e
    }

    fn solid(&mut self, e: Entity, size: Vec3) {
        self.scene.world.insert(e, Collider { size, ..Default::default() });
    }
}

fn script(name: &str, props: serde_json::Value) -> Scripts {
    Scripts(vec![ScriptSlot { name: name.into(), props: props.as_object().cloned().unwrap_or_default() }])
}

fn main() {
    let mut args = std::env::args().skip(1);
    let source = args.next().unwrap_or_else(|| ".test_assets/Models/GLB format".into());
    let games = args.next().unwrap_or_else(|| "games".into());
    let root = Path::new(&games).join("showcase");

    let mut scene = Scene::default();
    scene.settings.atmosphere = SkySettings { procedural: true, time: 15.0, speed: 0.05, clouds: 0.35, haze: 0.35, stars: 0.9, ..Default::default() };
    scene.settings.exposure = 1.05;
    scene.settings.saturation = 1.1;
    scene.settings.post.bloom = 0.35;
    scene.settings.post.vignette = 0.25;
    let mut b = Builder { scene, used: BTreeSet::new() };

    // Light and camera.
    let sun = b.entity("Sun");
    b.scene.world.insert(sun, Transform { position: Vec3::new(0.0, 30.0, 0.0), rotation: Vec3::new(-40.0, 35.0, 0.0), ..Default::default() });
    b.scene.world.insert(sun, Light { intensity: 1.0, ..Default::default() });
    let cam = b.entity("Main Camera");
    b.scene.world.insert(cam, Transform::at(Vec3::new(0.0, 7.0, 19.0)));
    b.scene.world.insert(cam, Camera { far: 600.0, ..Default::default() });
    b.scene.world.insert(cam, script("FollowCamera", serde_json::json!({})));

    // Island: sandy slab with a grassy top, seabed below the water.
    let island = b.entity("Island");
    b.scene.world.insert(island, Transform { position: Vec3::new(0.0, -0.6, 0.0), scale: Vec3::new(26.0, 1.2, 22.0), ..Default::default() });
    b.scene.world.insert(island, Shape::of(ShapeType::Cylinder));
    b.scene.world.insert(island, MeshRenderer { color: [0.82, 0.72, 0.5, 1.0], roughness: 1.0, ..Default::default() });
    b.scene.world.insert(island, Collider { shape: ShapeKind::Cylinder, size: Vec3::new(1.0, 1.0, 1.0), ..Default::default() });
    let seabed = b.entity("Seabed");
    b.scene.world.insert(seabed, Transform { position: Vec3::new(0.0, -3.2, 0.0), scale: Vec3::new(160.0, 1.0, 160.0), ..Default::default() });
    b.scene.world.insert(seabed, Shape::of(ShapeType::Cube));
    b.scene.world.insert(seabed, MeshRenderer { color: [0.18, 0.28, 0.32, 1.0], roughness: 1.0, cast_shadows: false, ..Default::default() });
    b.solid(seabed, Vec3::ONE);
    let grass = b.entity("Meadow");
    b.scene.world.insert(grass, Transform::at(Vec3::new(0.0, 0.0, 0.0)));
    b.scene.world.insert(grass, Grass {
        count: 90_000,
        width: 23.0,
        depth: 19.0,
        blade_height: 0.42,
        height_variation: 0.55,
        blade_width: 0.07,
        base_color: [0.02, 0.07, 0.02, 1.0],
        tip_color: [0.42, 0.6, 0.22, 1.0],
        patchiness: 0.85,
        round: true,
        flowers: 0.05,
        wind_strength: 0.45,
        chunk_size: 6.0,
        ..Default::default()
    });

    // Water: smooth, semi-realistic sea and a blocky voxel pond on the island.
    let sea = b.entity("Sea");
    b.scene.world.insert(sea, Transform::at(Vec3::new(0.0, -0.05, 0.0)));
    b.scene.world.insert(sea, VoxelWater { style: WaterStyle::Realistic, cells: 150, cell: 1.4, wave_height: 0.22, wavelength: 11.0, ..Default::default() });
    b.scene.world.insert(sea, AudioSource { clip: "sounds/waves.wav".into(), volume: 0.7, spatial: false, ..Default::default() });
    let pond = b.entity("Voxel Pond");
    b.scene.world.insert(pond, Transform::at(Vec3::new(-6.0, 0.0, -3.0)));
    b.scene.world.insert(pond, VoxelWater { style: WaterStyle::Voxel, cells: 12, cell: 0.5, wave_height: 0.12, wavelength: 3.0, depth: 0.6, ..Default::default() });
    let clouds = b.entity("Clouds");
    b.scene.world.insert(clouds, Transform::at(Vec3::new(0.0, 70.0, 0.0)));
    b.scene.world.insert(clouds, VoxelClouds::default());

    // Ruined tower and a few props.
    b.model("tower-base", Vec3::new(5.0, 0.2, -4.0), 0.0, K);
    b.model("tower", Vec3::new(5.0, 3.2, -4.0), 0.0, K);
    b.model("column-damaged", Vec3::new(8.5, 0.2, -1.0), 20.0, K);
    b.model("column", Vec3::new(2.0, 0.2, -7.0), 0.0, K);
    for (i, &(x, z)) in [(-3.0f32, 5.0f32), (3.5, 4.5), (7.0, 3.0)].iter().enumerate() {
        let e = b.model(if i % 2 == 0 { "barrels" } else { "detail-crate-ropes" }, Vec3::new(x, 0.6, z), i as f32 * 53.0, 2.0);
        b.solid(e, Vec3::splat(0.5));
    }
    // Procedural trees: oaks, pines, palms on the beach and a few bushes.
    let (oak, oak_2) = ([0.05, 0.14, 0.03, 1.0], [0.16, 0.24, 0.04, 1.0]);
    let (pine, pine_2, pine_bark) = ([0.02, 0.07, 0.03, 1.0], [0.04, 0.11, 0.04, 1.0], [0.25, 0.17, 0.12, 1.0]);
    let (palm, palm_2, palm_bark) = ([0.06, 0.16, 0.03, 1.0], [0.18, 0.26, 0.05, 1.0], [0.45, 0.36, 0.25, 1.0]);
    let trees = [
        ("Oak", Vec3::new(-9.0, 0.0, -5.5), Tree { height: 6.2, trunk_radius: 0.3, seed: 3, leaf_density: 16, ..Default::default() }),
        ("Oak 2", Vec3::new(-10.0, 0.0, 3.5), Tree { height: 5.0, trunk_radius: 0.24, seed: 8, leaf_color: [0.06, 0.15, 0.03, 1.0], leaf_color_2: [0.3, 0.26, 0.04, 1.0], ..Default::default() }),
        ("Pine", Vec3::new(9.5, 0.0, -5.0), Tree { style: TreeStyle::Conifer, height: 8.5, trunk_radius: 0.22, levels: 2, branches: 9, branch_angle: 78.0, branch_length: 0.5, leaf_density: 10, leaf_size: 0.6, crookedness: 0.12, bark_color: pine_bark, leaf_color: pine, leaf_color_2: pine_2, seed: 5, ..Default::default() }),
        ("Pine 2", Vec3::new(6.0, 0.0, -9.0), Tree { style: TreeStyle::Conifer, height: 6.5, trunk_radius: 0.18, levels: 2, branches: 8, branch_angle: 76.0, branch_length: 0.5, leaf_density: 10, leaf_size: 0.55, crookedness: 0.1, bark_color: pine_bark, leaf_color: pine, leaf_color_2: pine_2, seed: 12, ..Default::default() }),
        ("Palm", Vec3::new(10.5, 0.0, 5.5), Tree { style: TreeStyle::Palm, height: 6.5, trunk_radius: 0.16, crookedness: 0.6, leaf_density: 6, leaf_size: 0.9, bark_color: palm_bark, leaf_color: palm, leaf_color_2: palm_2, seed: 2, ..Default::default() }),
        ("Palm 2", Vec3::new(-6.5, 0.0, 9.0), Tree { style: TreeStyle::Palm, height: 5.5, trunk_radius: 0.15, crookedness: 0.8, leaf_density: 6, leaf_size: 0.85, bark_color: palm_bark, leaf_color: palm, leaf_color_2: palm_2, seed: 7, ..Default::default() }),
        ("Bush", Vec3::new(-4.0, 0.0, -7.5), Tree { style: TreeStyle::Bush, height: 1.6, trunk_radius: 0.08, levels: 2, branches: 3, leaf_density: 12, leaf_size: 0.55, seed: 4, leaf_color: oak, leaf_color_2: oak_2, ..Default::default() }),
        ("Bush 2", Vec3::new(8.0, 0.0, 1.0), Tree { style: TreeStyle::Bush, height: 1.3, trunk_radius: 0.07, levels: 2, branches: 3, leaf_density: 12, leaf_size: 0.5, seed: 9, leaf_color: [0.05, 0.12, 0.03, 1.0], leaf_color_2: [0.26, 0.05, 0.07, 1.0], ..Default::default() }),
        ("Bush 3", Vec3::new(-11.0, 0.0, -1.0), Tree { style: TreeStyle::Bush, height: 1.5, trunk_radius: 0.07, levels: 2, branches: 3, leaf_density: 12, leaf_size: 0.55, seed: 14, ..Default::default() }),
    ];
    for (i, (name, at, tree)) in trees.into_iter().enumerate() {
        let e = b.entity(name);
        b.scene.world.insert(e, Transform { position: at, rotation: Vec3::new(0.0, i as f32 * 40.0, 0.0), ..Default::default() });
        b.scene.world.insert(e, tree);
    }
    for i in 0..4 {
        b.model("dock-side", Vec3::new(-1.5 + i as f32 * 1.3, 0.1, 12.5), 90.0, 1.6);
    }

    // Mesh modifiers at work: a twisted crystal tower and a bent arch of arrayed blocks.
    let crystal = b.entity("Crystal Tower");
    b.scene.world.insert(crystal, Transform { position: Vec3::new(-4.0, 1.6, 6.0), scale: Vec3::new(1.2, 3.2, 1.2), ..Default::default() });
    b.scene.world.insert(crystal, Shape::of(ShapeType::Cube));
    b.scene.world.insert(crystal, MeshRenderer { color: [0.55, 0.85, 1.0, 1.0], roughness: 0.1, metallic: 0.3, reflectivity: 0.5, emissive: 0.25, ..Default::default() });
    b.scene.world.insert(crystal, MeshModifiers(vec![
        Modifier::named("Subdivide").map(|m| match m { Modifier::Subdivide { enabled, smooth, .. } => Modifier::Subdivide { enabled, levels: 3, smooth }, o => o }).unwrap(),
        Modifier::Twist { enabled: true, angle: 160.0, axis: ModAxis::Y },
        Modifier::Taper { enabled: true, amount: -0.75, axis: ModAxis::Y },
    ]));
    let arch = b.entity("Block Arch");
    b.scene.world.insert(arch, Transform { position: Vec3::new(2.0, 0.4, 8.0), scale: Vec3::new(0.5, 0.5, 0.5), ..Default::default() });
    b.scene.world.insert(arch, Shape::of(ShapeType::Cube));
    b.scene.world.insert(arch, MeshRenderer { color: [0.85, 0.8, 0.7, 1.0], roughness: 0.9, ..Default::default() });
    b.scene.world.insert(arch, MeshModifiers(vec![Modifier::Array { enabled: true, count: 8, offset: Vec3::new(1.1, 0.0, 0.0), rotation: Vec3::ZERO, scale: 1.0 }, Modifier::Bend { enabled: true, angle: 140.0, axis: ModAxis::X }]));

    // The explorer and the gems.
    let player = b.entity("Explorer");
    b.scene.world.insert(player, Transform::at(Vec3::new(0.0, 2.0, 9.0)));
    b.scene.world.insert(player, Shape::of(ShapeType::Sphere));
    b.scene.world.insert(player, MeshRenderer { color: [0.95, 0.35, 0.25, 1.0], roughness: 0.25, metallic: 0.2, reflectivity: 0.25, ..Default::default() });
    b.scene.world.insert(player, Collider { shape: ShapeKind::Sphere, size: Vec3::ONE, friction: 0.8, ..Default::default() });
    b.scene.world.insert(player, RigidBody { mass: 1.0, linear_damping: 1.4, angular_damping: 0.6, ccd: true, ..Default::default() });
    b.scene.world.insert(player, script("Explorer", serde_json::json!({})));

    let gem_spots = [(0.0, 3.0), (-6.0, -3.0), (6.5, 2.0), (3.0, -8.0), (-8.5, 4.0), (9.0, -7.0), (-2.5, -9.0), (0.5, 12.8)];
    for (i, &(x, z)) in gem_spots.iter().enumerate() {
        let gem = b.entity(&format!("Gem {}", i + 1));
        b.scene.world.insert(gem, Transform { position: Vec3::new(x, 1.2, z), scale: Vec3::splat(0.55), ..Default::default() });
        b.scene.world.insert(gem, Shape::of(ShapeType::Torus));
        let hue = i as f32 / gem_spots.len() as f32;
        let [r, g, bl] = hsv_to_rgb(hue * 0.8 + 0.45, 0.65, 1.0);
        b.scene.world.insert(gem, MeshRenderer { color: [r, g, bl, 1.0], roughness: 0.1, emissive: 1.4, metallic: 0.4, reflectivity: 0.5, ..Default::default() });
        b.scene.world.insert(gem, script("Gem", serde_json::json!({})));
    }

    let manager = b.entity("Game Manager");
    b.scene.world.insert(manager, script("GameManager", serde_json::json!({ "total": gem_spots.len() })));

    // Copy only the models the scene uses, plus the kit textures.
    let models_out = root.join("assets/models");
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

    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/main.scene.json"), b.scene.to_json(&Registry::default())).unwrap();
    Project {
        root: root.clone(),
        config: ProjectConfig { name: "Treasure Isle".into(), start_scene: "scenes/main.scene.json".into(), scripts_package: "showcase_scripts".into(), ..Default::default() },
    }
    .save()
    .unwrap();
    println!("wrote {} ({} models)", root.display(), b.used.len());
}
