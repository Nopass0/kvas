//! Renders LAST SERVICE's characters after a few seconds of ragdoll physics (one falls over a
//! crate) next to one with foot IK on steps, as a visual check of ragdolls and IK.
//! `cargo run -p kvas_render --example ragdoll_shot -- out.png [seconds]`
use glam::Vec3;
use kvas_engine::{assets::Assets, components::*, hierarchy, runtime::Runtime, scene::Scene};
use kvas_render::{debug::DebugDraw, Frame, Renderer, View};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "ragdoll.png".into());
    let seconds: f32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(2.5);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/last_service/assets");
    let assets = Assets::Dir(root);
    kvas_engine::meshmod::set_assets(assets.clone());
    let mut scene = Scene::default();
    scene.settings.atmosphere.procedural = true;
    scene.settings.atmosphere.time = 14.0;
    scene.settings.ambient = [0.45, 0.45, 0.48];
    let floor = scene.spawn("Floor");
    scene.world.insert(floor, Transform { position: Vec3::new(0.0, -0.5, 0.0), scale: Vec3::new(20.0, 1.0, 20.0), ..Default::default() });
    scene.world.insert(floor, Shape::of(ShapeType::Cube));
    scene.world.insert(floor, MeshRenderer { color: [0.36, 0.37, 0.35, 1.0], ..Default::default() });
    scene.world.insert(floor, Collider::default());
    let sun = scene.spawn("Sun");
    scene.world.insert(sun, Transform { rotation: Vec3::new(-50.0, 30.0, 0.0), ..Default::default() });
    scene.world.insert(sun, Light { intensity: 1.5, ..Default::default() });
    let block = |scene: &mut Scene, name: &str, pos: Vec3, size: Vec3| {
        let e = scene.spawn(name);
        scene.world.insert(e, Transform { position: pos, scale: size, ..Default::default() });
        scene.world.insert(e, Shape::of(ShapeType::Cube));
        scene.world.insert(e, MeshRenderer { color: [0.5, 0.38, 0.25, 1.0], ..Default::default() });
        scene.world.insert(e, Collider::default());
    };
    block(&mut scene, "Crate", Vec3::new(-1.6, 0.3, 0.6), Vec3::new(0.8, 0.6, 0.8));
    block(&mut scene, "Step", Vec3::new(1.75, 0.1, 0.0), Vec3::new(0.3, 0.2, 0.6));
    let character = |scene: &mut Scene, name: &str, mesh: &str, tex: &str, pos: Vec3, clip: &str, scale: f32| {
        let e = scene.spawn(name);
        scene.world.insert(e, Transform { position: pos, scale: Vec3::splat(scale), ..Default::default() });
        scene.world.insert(e, MeshRenderer { mesh: mesh.into(), texture: tex.into(), ..Default::default() });
        scene.world.insert(e, Animator { clip: clip.into(), ..Default::default() });
        e
    };
    let anna = character(&mut scene, "Anna", "models/characters/female_secretary.fbx", "models/characters/secretary_tex.png", Vec3::new(-1.6, 0.0, -0.2), "Walk", 1.0);
    let monster = character(&mut scene, "Counter", "models/characters/MonsterPSX.fbx", "models/characters/MonsterPSX_Diffuse.png", Vec3::new(0.2, 0.0, 0.0), "Run_Frantic", 1.15);
    let pavel = character(&mut scene, "Pavel", "models/characters/male_casual.fbx", "models/characters/man_tex.png", Vec3::new(1.6, 0.0, 0.0), "Idle", 1.0);
    scene.world.insert(pavel, CharacterIk { look_point: Some(Vec3::new(4.0, 1.6, 4.0)), ..Default::default() });
    let mut rt = Runtime::new(scene, None);
    for _ in 0..20 {
        rt.tick(1.0 / 60.0);
    }
    rt.scene.world.insert(anna, Ragdoll { active: true, impulse: Vec3::new(0.0, 0.0, 35.0), ..Default::default() });
    rt.scene.world.insert(monster, Ragdoll { active: true, impulse: Vec3::new(-20.0, 10.0, 25.0), ..Default::default() });
    for _ in 0..(seconds * 60.0) as usize {
        rt.tick(1.0 / 60.0);
    }
    hierarchy::update_globals(&mut rt.scene.world);
    let mut camera = Transform::at(Vec3::new(0.0, 1.9, 5.2));
    camera.look_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y);
    let view = View::new(camera.matrix(), 50.0, 16.0 / 9.0, 0.05, 200.0);
    let debug = DebugDraw::default();
    let frame = Frame { view, size: [1280, 720], time: 0.0, dt: 0.0, debug: &debug, shadows: true, debug_view: 0, cinematic: true };
    let mut renderer = Renderer::new_headless(assets)?;
    let image = renderer.capture(&rt.scene, &frame)?;
    image.save(&out).map_err(|e| e.to_string())?;
    println!("Saved {out}");
    Ok(())
}
