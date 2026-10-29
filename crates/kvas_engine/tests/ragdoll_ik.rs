//! Ragdolls and IK on LAST SERVICE's characters in a running game (physics included): a
//! ragdoll falls onto the floor without sinking through it and goes away when switched off;
//! feet stand on a raised floor; the head turns to a target. Skipped without the game's assets.

use glam::Vec3;
use kvas_engine::assets::Assets;
use kvas_engine::components::*;
use kvas_engine::runtime::Runtime;
use kvas_engine::scene::Scene;
use std::path::PathBuf;

const ANNA: &str = "models/characters/female_secretary.fbx";

fn assets() -> Option<Assets> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games/last_service/assets");
    root.join(ANNA).exists().then(|| Assets::Dir(root))
}

/// A floor whose top is at `floor` and a character standing at the origin.
fn scene(floor: f32, clip: &str) -> (Scene, kvas_ecs::Entity) {
    let mut scene = Scene::default();
    let ground = scene.spawn("Floor");
    scene.world.insert(ground, Transform { position: Vec3::new(0.0, floor - 0.5, 0.0), ..Default::default() });
    scene.world.insert(ground, Collider { size: Vec3::new(20.0, 1.0, 20.0), ..Default::default() });
    let anna = scene.spawn("Anna");
    scene.world.insert(anna, MeshRenderer { mesh: ANNA.into(), ..Default::default() });
    scene.world.insert(anna, Animator { clip: clip.into(), ..Default::default() });
    (scene, anna)
}

fn skinned(rt: &Runtime, e: kvas_ecs::Entity) -> Vec<Vec3> {
    let dm = rt.scene.world.get::<DynamicMesh>(e).expect("skinned mesh");
    dm.data.vertices.iter().map(|v| Vec3::from(v.pos)).collect()
}

fn run(rt: &mut Runtime, seconds: f32) {
    for _ in 0..(seconds * 60.0) as usize {
        rt.tick(1.0 / 60.0);
    }
}

#[test]
fn ragdoll_falls_onto_the_floor_and_recovers() {
    let Some(assets) = assets() else { return };
    kvas_engine::meshmod::set_assets(assets);
    let (scene, anna) = scene(0.0, "Idle");
    let mut rt = Runtime::new(scene, None);
    run(&mut rt, 0.3);
    let standing = skinned(&rt, anna);
    let height = standing.iter().map(|p| p.y).fold(f32::MIN, f32::max);
    assert!(height > 1.4, "stands {height}");
    rt.scene.world.insert(anna, Ragdoll { active: true, impulse: Vec3::new(0.0, 0.0, -25.0), ..Default::default() });
    run(&mut rt, 3.0);
    let bodies = rt.scene.world.entities().into_iter().filter(|e| kvas_engine::hierarchy::name_of(&rt.scene.world, *e).starts_with("ragdoll#")).count();
    assert!(bodies >= 9, "ragdoll parts: {bodies}");
    let lying = skinned(&rt, anna);
    let top = lying.iter().map(|p| p.y).fold(f32::MIN, f32::max);
    let bottom = lying.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    assert!(lying.iter().all(|p| p.is_finite()));
    assert!(top < height * 0.6, "fell over: top {top} of {height}");
    assert!(bottom > -0.15, "did not sink through the floor: {bottom}");
    rt.scene.world.get_mut::<Ragdoll>(anna).unwrap().active = false;
    run(&mut rt, 0.2);
    let left = rt.scene.world.entities().into_iter().filter(|e| kvas_engine::hierarchy::name_of(&rt.scene.world, *e).starts_with("ragdoll#")).count();
    assert_eq!(left, 0, "bodies removed");
    let again = skinned(&rt, anna).iter().map(|p| p.y).fold(f32::MIN, f32::max);
    assert!((again - height).abs() < 0.1, "animated again: {again}");
}

#[test]
fn feet_stand_on_the_floor_and_the_head_follows_a_target() {
    let Some(assets) = assets() else { return };
    kvas_engine::meshmod::set_assets(assets);
    // The character stands at y = 0 but the floor is 15 cm higher (or 20 cm lower).
    for floor in [0.15f32, -0.2] {
        let (mut scene, anna) = scene(floor, "Idle");
        scene.world.insert(anna, CharacterIk::default());
        let mut rt = Runtime::new(scene, None);
        run(&mut rt, 1.0);
        let lowest = skinned(&rt, anna).iter().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!((lowest - floor).abs() < 0.06, "feet on the floor at {floor}: lowest vertex {lowest}");
    }
    // Looking at a point to her side turns the head that way.
    let (mut scene, anna) = scene(0.0, "Idle");
    scene.world.insert(anna, CharacterIk { feet: false, ..Default::default() });
    let mut rt = Runtime::new(scene, None);
    run(&mut rt, 0.5);
    let bones = kvas_engine::skeletal::model(ANNA).unwrap().humanoid.clone().unwrap();
    let (side, forward) = (bones.right, bones.forward);
    // The tip of the face: the most forward vertex of the head; follow that vertex.
    let v = skinned(&rt, anna);
    let top = v.iter().map(|p| p.y).fold(f32::MIN, f32::max);
    let face = (0..v.len()).filter(|i| v[*i].y > top - 0.25).max_by(|a, b| v[*a].dot(forward).total_cmp(&v[*b].dot(forward))).unwrap();
    let before = v[face];
    rt.scene.world.get_mut::<CharacterIk>(anna).unwrap().look_point = Some(Vec3::new(0.0, 1.5, 0.0) + side * 3.0 + forward * 1.0);
    run(&mut rt, 1.5);
    let after = skinned(&rt, anna)[face];
    assert!((after - before).dot(side) > 0.04, "head turned to the side: {before} -> {after}");
}
