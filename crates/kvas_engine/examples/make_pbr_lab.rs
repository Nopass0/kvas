//! Portable PBR/HDRI sample inside games/demo. Run after tools/fetch_demo_assets.ps1.
use kvas_engine::{prelude::*, scene::Scene, registry::Registry, material_graph::MaterialGraph};
use std::path::Path;

fn object(scene: &mut Scene, name: &str, kind: ShapeType, position: Vec3, scale: Vec3, color: [f32; 4]) -> Entity {
    let e = scene.spawn(name);
    scene.world.insert(e, Transform { position, scale, ..Default::default() });
    scene.world.insert(e, Shape::of(kind));
    scene.world.insert(e, MeshRenderer { mesh: String::new(), color, ..Default::default() });
    let mut tags = Tags::default(); tags.insert("pbr_lab"); scene.world.insert(e, tags);
    e
}

fn main() {
    let root = Path::new("games/demo/assets");
    let mut scene = Scene::default();
    scene.settings.environment_map = "environments/studio_small_09_1k.hdr".into();
    scene.settings.environment_rotation = 40.0;
    scene.settings.environment_intensity = 0.7;
    scene.settings.filmic = 1.0;
    scene.settings.exposure = 1.0;
    scene.settings.ambient = [0.15; 3];
    let floor = object(&mut scene, "Slate display floor", ShapeType::Cube, Vec3::new(0.0, -0.22, 0.0), Vec3::new(14.0, 0.4, 10.0), [0.18, 0.2, 0.23, 1.0]);
    scene.world.insert(floor, Collider { size: Vec3::ONE, ..Default::default() });
    if let Some(m) = scene.world.get_mut::<MeshRenderer>(floor) { m.roughness = 0.35; m.reflectivity = 0.4; }
    for row in 0..3 {
        for col in 0..5 {
            let position = Vec3::new(col as f32 * 2.0 - 4.0, 0.0, row as f32 * 2.4 - 2.4);
            object(&mut scene, &format!("Pedestal {row}-{col}"), ShapeType::Cylinder, position + Vec3::Y * 0.2, Vec3::new(1.35, 0.4, 1.35), [0.1, 0.12, 0.15, 1.0]);
            let color = [[0.95, 0.67, 0.25, 1.0], [0.72, 0.8, 0.9, 1.0], [0.7, 0.22, 0.1, 1.0]][row];
            let kind = if row == 2 { ShapeType::Torus } else { ShapeType::Sphere };
            let e = object(&mut scene, &format!("{} roughness {:.2}", ["Gold", "Chrome", "Ceramic"][row], col as f32 / 4.0), kind, position + Vec3::Y * 1.05, Vec3::splat(1.25), color);
            if let Some(m) = scene.world.get_mut::<MeshRenderer>(e) { m.roughness = (col as f32 / 4.0).max(0.04); m.metallic = if row < 2 { 1.0 } else { 0.0 }; m.reflectivity = 0.7; }
        }
    }
    std::fs::create_dir_all(root.join("materials/lab")).unwrap();
    for (name, pattern, color) in [("Marble", "Marble veins", [0.8, 0.83, 0.9]), ("Wood", "Wood rings", [0.5, 0.22, 0.08])] {
        let mut graph = MaterialGraph::default_graph();
        graph.nodes.iter_mut().find(|n| n.kind == "Color").unwrap().params = color.to_vec();
        let noise = graph.add_node(pattern, [10.0, 220.0]);
        let mix = graph.add_node("Multiply", [220.0, 80.0]);
        graph.connect(2, mix, 0); graph.connect(noise, mix, 1); graph.connect(mix, 1, 0);
        let rel = format!("materials/lab/{}.kmat", name.to_lowercase());
        std::fs::write(root.join(&rel), graph.to_json()).unwrap();
        let x = if name == "Marble" { -2.0 } else { 2.0 };
        let e = object(&mut scene, name, ShapeType::Cube, Vec3::new(x, 1.0, -5.0), Vec3::splat(1.8), [1.0; 4]);
        scene.world.get_mut::<MeshRenderer>(e).unwrap().shader = rel;
    }
    let camera = scene.spawn("Camera");
    let mut pose = Transform::at(Vec3::new(8.8, 6.5, 12.0)); pose.look_at(Vec3::new(0.0, 0.6, -0.5), Vec3::Y);
    scene.world.insert(camera, pose); scene.world.insert(camera, Camera { fov: 52.0, far: 200.0, ..Default::default() });
    let sun = scene.spawn("Soft key light");
    scene.world.insert(sun, Transform { rotation: Vec3::new(-50.0, -30.0, 0.0), ..Default::default() });
    scene.world.insert(sun, Light { intensity: 0.8, color: [1.0, 0.93, 0.84, 1.0], ..Default::default() });
    std::fs::create_dir_all(root.join("scenes")).unwrap();
    scene.save(&Registry::default(), &root.join("scenes/pbr_lab.scene.json")).unwrap();
    println!("PBR lab: {} objects", scene.world.entities().len());
}
