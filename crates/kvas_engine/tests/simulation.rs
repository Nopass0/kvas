use glam::Vec3;
use kvas_engine::prelude::*;
use kvas_engine::registry::Registry;
use kvas_engine::runtime::Runtime;
use kvas_engine::scene::Scene;

fn floor(scene: &mut Scene) -> Entity {
    let e = hierarchy::spawn_named(&mut scene.world, "Floor");
    scene.world.insert(e, Transform { scale: Vec3::new(20.0, 1.0, 20.0), position: Vec3::new(0.0, -0.5, 0.0), ..Default::default() });
    scene.world.insert(e, Collider::default()); // no body: static
    e
}

fn cube(scene: &mut Scene, name: &str, pos: Vec3) -> Entity {
    let e = hierarchy::spawn_named(&mut scene.world, name);
    scene.world.insert(e, Transform::at(pos));
    scene.world.insert(e, RigidBody::default());
    scene.world.insert(e, Collider::default());
    e
}

fn run(rt: &mut Runtime, seconds: f32) {
    for _ in 0..(seconds * 60.0) as usize {
        rt.tick(1.0 / 60.0);
    }
}

#[test]
fn box_falls_and_rests_on_floor() {
    let mut scene = Scene::default();
    floor(&mut scene);
    let b = cube(&mut scene, "Box", Vec3::new(0.0, 4.0, 0.0));
    let mut rt = Runtime::new(scene, None);
    run(&mut rt, 3.0);
    let y = rt.scene.world.get::<Transform>(b).unwrap().position.y;
    assert!((y - 0.5).abs() < 0.05, "box should rest on the floor, y = {y}");
}

#[test]
fn child_collider_moves_with_parent_body() {
    let mut scene = Scene::default();
    floor(&mut scene);
    let parent = cube(&mut scene, "Parent", Vec3::new(0.0, 3.0, 0.0));
    let child = hierarchy::spawn_named(&mut scene.world, "Child");
    scene.world.insert(child, Transform::at(Vec3::new(1.0, 0.0, 0.0)));
    scene.world.insert(child, Collider::default());
    hierarchy::set_parent(&mut scene.world, child, Some(parent));
    scene.world.insert(child, Transform::at(Vec3::new(1.0, 0.0, 0.0))); // local offset
    let mut rt = Runtime::new(scene, None);
    run(&mut rt, 10.0);
    let w = &rt.scene.world;
    let py = w.get::<Transform>(parent).unwrap().position;
    assert!(py.y < 1.1, "parent should have fallen: {py}");
    // The child keeps its local offset because it is part of the compound body.
    let local = w.get::<Transform>(child).unwrap().position;
    assert!((local - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-3, "child local moved: {local}");
}

#[test]
fn hinge_to_world_keeps_body_on_circle() {
    let mut scene = Scene::default();
    let b = cube(&mut scene, "Pendulum", Vec3::new(2.0, 5.0, 0.0));
    scene.world.insert(b, Joint { kind: JointKind::Hinge, axis: Vec3::Z, anchor: Vec3::new(-2.0, 0.0, 0.0), ..Default::default() });
    let mut rt = Runtime::new(scene, None);
    run(&mut rt, 2.0);
    // Pivot is the world point (0,5,0); the body must stay ~2 units from it.
    let p = rt.scene.world.get::<Transform>(b).unwrap().position;
    let dist = (p - Vec3::new(0.0, 5.0, 0.0)).length();
    assert!((dist - 2.0).abs() < 0.1, "hinge distance drifted: {dist}");
    assert!(p.y < 4.9, "pendulum should have swung down, y = {}", p.y);
}

#[test]
fn raycast_hits_floor() {
    let mut scene = Scene::default();
    let f = floor(&mut scene);
    let mut rt = Runtime::new(scene, None);
    run(&mut rt, 0.1);
    let hit = rt.physics.raycast(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y, 100.0).expect("ray should hit");
    assert_eq!(hit.entity, f);
    assert!((hit.distance - 5.0).abs() < 0.01);
}

#[test]
fn scene_roundtrips_with_hierarchy_and_components() {
    let reg = Registry::default();
    let mut scene = Scene::default();
    let a = cube(&mut scene, "A", Vec3::new(1.0, 2.0, 3.0));
    let b = hierarchy::spawn_named(&mut scene.world, "B");
    scene.world.insert(b, MeshRenderer { mesh: "primitive:sphere".into(), ..Default::default() });
    hierarchy::set_parent(&mut scene.world, b, Some(a));
    let text = scene.to_json(&reg);
    let (loaded, warnings) = Scene::from_json(&reg, &text).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let a2 = hierarchy::find_by_name(&loaded.world, "A").unwrap();
    let b2 = hierarchy::find_by_name(&loaded.world, "B").unwrap();
    assert_eq!(hierarchy::parent_of(&loaded.world, b2), Some(a2));
    assert_eq!(loaded.world.get::<Transform>(a2).unwrap().position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(loaded.world.get::<MeshRenderer>(b2).unwrap().mesh, "primitive:sphere");
}

#[test]
fn reparenting_preserves_world_position() {
    let mut scene = Scene::default();
    let p = hierarchy::spawn_named(&mut scene.world, "P");
    scene.world.insert(p, Transform { position: Vec3::new(10.0, 0.0, 0.0), scale: Vec3::splat(2.0), ..Default::default() });
    let c = hierarchy::spawn_named(&mut scene.world, "C");
    scene.world.insert(c, Transform::at(Vec3::new(1.0, 1.0, 1.0)));
    assert!(hierarchy::set_parent(&mut scene.world, c, Some(p)));
    let world_pos = hierarchy::global_matrix(&scene.world, c).w_axis.truncate();
    assert!((world_pos - Vec3::new(1.0, 1.0, 1.0)).length() < 1e-4);
    assert!(!hierarchy::set_parent(&mut scene.world, p, Some(c)), "cycles must be rejected");
}

#[test]
fn shape_component_roundtrips_and_builds_its_mesh() {
    use kvas_engine::assets::Assets;
    use kvas_engine::mesh::MeshData;
    let reg = Registry::default();
    let mut scene = Scene::default();
    let e = hierarchy::spawn_named(&mut scene.world, "Ring");
    scene.world.insert(e, MeshRenderer::default());
    scene.world.insert(e, Shape { kind: ShapeType::Torus, radius: 1.25, radius2: 0.4, segments: 24, rings: 12, ..Default::default() });
    let (loaded, warnings) = Scene::from_json(&reg, &scene.to_json(&reg)).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let e2 = hierarchy::find_by_name(&loaded.world, "Ring").unwrap();
    let key = kvas_engine::components::mesh_key(&loaded.world, e2).unwrap();
    assert!(key.starts_with("proc:torus:"), "{key}");
    let mesh = MeshData::load(&Assets::Dir(std::env::temp_dir()), &key).unwrap();
    let (min, max) = mesh.bounds();
    assert!((max.x - 1.65).abs() < 0.02 && (max.y - 0.4).abs() < 0.02, "torus bounds {min} {max}");
}

#[test]
fn modifiers_animate_without_scripts() {
    let mut scene = Scene::default();
    let e = hierarchy::spawn_named(&mut scene.world, "Thing");
    scene.world.insert(e, Transform::at(Vec3::new(0.0, 5.0, 0.0)));
    scene.world.insert(e, MeshRenderer::default());
    scene.world.insert(e, Spin { speed: Vec3::new(0.0, 90.0, 0.0) });
    scene.world.insert(e, Bob { amplitude: 1.0, speed: 0.25, ..Default::default() });
    scene.world.insert(e, Pulse { amount: 0.5, speed: 0.25, ..Default::default() });
    scene.world.insert(e, ColorCycle::default());
    let before = scene.world.get::<MeshRenderer>(e).unwrap().color;
    let mut rt = Runtime::new(scene, None);
    run(&mut rt, 1.0); // quarter of a bob/pulse period at 0.25 Hz
    let w = &rt.scene.world;
    let t = w.get::<Transform>(e).unwrap();
    assert!((t.rotation.y - 90.0).abs() < 3.0, "spin: {}", t.rotation.y);
    assert!((t.position.y - 6.0).abs() < 0.05, "bob should be at its peak: {}", t.position.y);
    assert!((t.scale.x - 1.5).abs() < 0.05, "pulse should be at its peak: {}", t.scale.x);
    assert_ne!(w.get::<MeshRenderer>(e).unwrap().color, before, "colour should have cycled");
}

#[test]
fn glb_models_load_with_their_external_texture() {
    use kvas_engine::assets::Assets;
    use kvas_engine::mesh::MeshData;
    // The medieval test project ships a few Kenney models that reference Textures/colormap.png.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/test_assets/assets");
    if !dir.join("models/wall.glb").exists() {
        return; // project not generated on this machine
    }
    let m = MeshData::load(&Assets::Dir(dir), "models/wall.glb").expect("wall.glb loads");
    assert!(m.indices.len() >= 36 && m.vertices.iter().all(|v| v.normal != [0.0; 3]));
    let albedo = m.albedo.expect("the colormap texture is attached");
    assert!(albedo.width > 0 && albedo.rgba.len() == (albedo.width * albedo.height * 4) as usize);
}
