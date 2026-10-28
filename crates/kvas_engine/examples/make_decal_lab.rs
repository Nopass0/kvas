//! Writes `games/demo/assets/scenes/decals.scene.json`: a floor, a wall, steps and a few shapes
//! covered with every decal preset and blend mode (also a stress row of 400 small decals).
//! `cargo run -p kvas_engine --example make_decal_lab`
use glam::Vec3;
use kvas_engine::decal::{preset, Decal, DecalBlend, DecalShape, PRESETS};
use kvas_engine::prelude::*;
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;

fn shape(scene: &mut Scene, name: &str, kind: ShapeType, pos: Vec3, scale: Vec3, rot: Vec3, color: [f32; 4], shader: &str) {
    let e = scene.spawn(name);
    scene.world.insert(e, Transform { position: pos, rotation: rot, scale });
    scene.world.insert(e, Shape::of(kind));
    scene.world.insert(e, MeshRenderer { color, roughness: 0.8, shader: shader.into(), ..Default::default() });
}

fn decal(scene: &mut Scene, name: &str, d: Decal, pos: Vec3, rot: Vec3, size: Vec3) {
    let e = scene.spawn(name);
    scene.world.insert(e, Transform { position: pos, rotation: rot, scale: size });
    scene.world.insert(e, d);
}

fn main() {
    let mut scene = Scene::default();
    scene.settings.atmosphere.procedural = true;
    scene.settings.atmosphere.time = 14.0;
    let sun = scene.spawn("Sun");
    scene.world.insert(sun, Transform { rotation: Vec3::new(-50.0, 30.0, 0.0), ..Default::default() });
    scene.world.insert(sun, Light { intensity: 1.1, ..Default::default() });
    let cam = scene.spawn("Camera");
    let mut t = Transform::at(Vec3::new(0.0, 6.0, 13.0));
    t.look_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
    scene.world.insert(cam, t);
    scene.world.insert(cam, Camera::default());

    let concrete = "";
    shape(&mut scene, "Floor", ShapeType::Cube, Vec3::new(0.0, -0.25, 0.0), Vec3::new(24.0, 0.5, 16.0), Vec3::ZERO, [0.6, 0.6, 0.58, 1.0], concrete);
    shape(&mut scene, "Wall", ShapeType::Cube, Vec3::new(0.0, 2.5, -5.0), Vec3::new(20.0, 5.0, 0.5), Vec3::ZERO, [0.7, 0.68, 0.64, 1.0], concrete);
    for i in 0..4 {
        shape(&mut scene, "Step", ShapeType::Cube, Vec3::new(6.5, 0.15 + i as f32 * 0.3, -3.2 + i as f32 * 0.6), Vec3::new(4.0, 0.3, 0.6), Vec3::ZERO, [0.55, 0.55, 0.55, 1.0], "");
    }
    shape(&mut scene, "Ball", ShapeType::Sphere, Vec3::new(-6.0, 1.0, 1.0), Vec3::splat(2.0), Vec3::ZERO, [0.8, 0.8, 0.82, 1.0], "");
    shape(&mut scene, "Barrel", ShapeType::Cylinder, Vec3::new(-3.0, 1.0, 2.5), Vec3::new(1.4, 2.0, 1.4), Vec3::ZERO, [0.3, 0.32, 0.36, 1.0], "");

    // Every preset on the floor, and the same presets on the wall (projected along -Z).
    for (i, name) in PRESETS.iter().enumerate() {
        let x = -10.0 + (i % 6) as f32 * 3.4;
        let z = 1.0 + (i / 6) as f32 * 3.4;
        let mut d = preset(name);
        d.seed = i as i32 + 1;
        decal(&mut scene, name, d.clone(), Vec3::new(x + 1.5, 0.0, z), Vec3::new(0.0, i as f32 * 30.0, 0.0), Vec3::new(2.4, 1.0, 2.4));
        if i < 8 {
            decal(&mut scene, name, d, Vec3::new(-8.5 + i as f32 * 2.4, 3.2, -4.75), Vec3::new(90.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 2.0));
        }
    }
    // Blend modes along the wall's lower band.
    for (i, blend) in [DecalBlend::Normal, DecalBlend::Multiply, DecalBlend::Screen, DecalBlend::Lighten, DecalBlend::Darken, DecalBlend::Overlay, DecalBlend::Add, DecalBlend::SoftLight, DecalBlend::ReliefOnly, DecalBlend::ColorDodge, DecalBlend::ColorBurn, DecalBlend::Tint]
        .into_iter()
        .enumerate()
    {
        let d = Decal { shape: DecalShape::PaintSplat, blend, color: [0.2, 0.55, 1.0, 1.0], seed: i as i32, normal_strength: 2.0, ..Default::default() };
        decal(&mut scene, "Blend", d, Vec3::new(-9.0 + i as f32 * 1.6, 1.0, -4.75), Vec3::new(90.0, 0.0, 0.0), Vec3::new(1.4, 1.0, 1.4));
    }
    // Wrapping over curved and stepped geometry.
    decal(&mut scene, "Over the ball", preset("rust"), Vec3::new(-6.0, 2.0, 1.0), Vec3::ZERO, Vec3::new(2.2, 2.5, 2.2));
    decal(&mut scene, "Over the steps", Decal { shape: DecalShape::PaintSplat, color: [1.0, 0.75, 0.1, 1.0], angle_fade: 0.0, ..Default::default() }, Vec3::new(6.5, 0.6, -2.3), Vec3::new(55.0, 0.0, 0.0), Vec3::new(3.0, 3.0, 3.0));
    decal(&mut scene, "Around the barrel", Decal { shape: DecalShape::Leaks, blend: DecalBlend::Multiply, color: [0.3, 0.2, 0.1, 1.0], angle_fade: 0.0, ..Default::default() }, Vec3::new(-3.0, 1.2, 3.2), Vec3::new(90.0, 0.0, 0.0), Vec3::new(1.6, 1.6, 1.6));
    // Many small decals (performance row).
    for i in 0..400 {
        let d = Decal { shape: DecalShape::BulletHole, color: [0.05, 0.05, 0.05, 1.0], normal_strength: 2.0, seed: i, max_distance: 40.0, ..Default::default() };
        let (x, y) = (-9.5 + (i % 40) as f32 * 0.45, 4.6 - (i / 40) as f32 * 0.2);
        decal(&mut scene, "Hole", d, Vec3::new(x, y, -4.75), Vec3::new(90.0, 0.0, 0.0), Vec3::new(0.18, 0.5, 0.18));
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/demo/assets/scenes/decals.scene.json");
    std::fs::write(&path, scene.to_json(&Registry::default())).expect("write scene");
    println!("wrote {}", path.display());
}
