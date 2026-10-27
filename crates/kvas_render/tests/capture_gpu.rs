//! Explicit opt-in native GPU check; ordinary workspace tests stay headless/adapter independent.
use glam::Vec3;
use kvas_engine::{assets::Assets, components::*, hierarchy, scene::Scene};
use kvas_render::{debug::DebugDraw, Frame, Renderer, View};

#[test]
#[ignore = "requires an available native wgpu adapter; run with --ignored"]
fn capture_readback_and_inherited_visibility_work_on_the_gpu() {
    let mut renderer = Renderer::new_headless(Assets::Dir(std::env::temp_dir())).expect("native GPU adapter");
    let mut scene = Scene::default();
    let parent = scene.spawn("parent");
    let cube = scene.spawn("cube");
    hierarchy::set_parent_local(&mut scene.world, cube, Some(parent));
    scene.world.insert(cube, MeshRenderer { color: [1.0, 0.1, 0.1, 1.0], ..Default::default() });
    hierarchy::update_globals(&mut scene.world);
    let mut camera = Transform::at(Vec3::new(2.0, 1.5, 3.0));
    camera.look_at(Vec3::ZERO, Vec3::Y);
    let debug = DebugDraw::default();
    let frame = Frame {
        view: View::new(camera.matrix(), 50.0, 97.0 / 65.0, 0.1, 100.0),
        size: [97, 65],
        time: 0.0,
        dt: 0.0,
        debug: &debug,
        shadows: false,
        debug_view: 0,
        cinematic: false,
    };
    let visible = renderer.capture(&scene, &frame).unwrap();
    assert_eq!(renderer.stats().mesh_draw_calls, 1);
    assert_eq!(renderer.stats().triangles, 12);
    assert_eq!(visible.dimensions(), (97, 65));
    assert_eq!(visible.as_raw().len(), 97 * 65 * 4);
    let repeated = renderer.capture(&scene, &frame).unwrap();
    assert_eq!(visible, repeated);
    scene.world.insert(parent, Visibility { visible: false });
    let hidden = renderer.capture(&scene, &frame).unwrap();
    assert_eq!(renderer.stats().hidden_objects, 1);
    assert_eq!(renderer.stats().mesh_draw_calls, 0);
    let different = visible.pixels().zip(hidden.pixels()).filter(|(a, b)| a != b).count();
    assert!(different > 50, "hidden parent must remove the cube's pixels: changed {different}");
    assert!(renderer.take_errors().is_empty());
    let invalid = Frame { size: [0, 65], ..frame };
    assert!(renderer.capture(&scene, &invalid).is_err());
}

#[test]
#[ignore = "requires an available native wgpu adapter; run with --ignored"]
fn dynamic_mesh_replacement_and_invalid_geometry_are_handled_without_stale_buffers() {
    use kvas_engine::mesh::MeshData;
    use std::sync::Arc;
    let mut renderer = Renderer::new_headless(Assets::Dir(std::env::temp_dir())).unwrap();
    let mut scene = Scene::default();
    let entity = scene.spawn("dynamic");
    scene.world.insert(entity, MeshRenderer::default());
    scene.world.insert(entity, DynamicMesh { data: Arc::new(MeshData::primitive("cube").unwrap()), version: 1 });
    hierarchy::update_globals(&mut scene.world);
    let mut camera = Transform::at(Vec3::new(2.0, 1.5, 3.0));
    camera.look_at(Vec3::ZERO, Vec3::Y);
    let debug = DebugDraw::default();
    let frame = Frame {
        view: View::new(camera.matrix(), 50.0, 1.0, 0.1, 100.0),
        size: [64, 64],
        time: 0.0,
        dt: 0.0,
        debug: &debug,
        shadows: false,
        debug_view: 0,
        cinematic: false,
    };
    renderer.capture(&scene, &frame).unwrap();
    assert_eq!(renderer.stats().triangles, 12);
    // Replacing Arc data with the same version is also detected (e.g. after scene replacement).
    scene.world.insert(entity, DynamicMesh { data: Arc::new(MeshData::primitive("sphere").unwrap()), version: 1 });
    renderer.capture(&scene, &frame).unwrap();
    assert!(renderer.stats().triangles > 12);
    let mut bad = MeshData::primitive("cube").unwrap();
    bad.indices[0] = u32::MAX;
    scene.world.insert(entity, DynamicMesh { data: Arc::new(bad), version: 1 });
    renderer.capture(&scene, &frame).unwrap();
    assert_eq!(renderer.stats().mesh_draw_calls, 0);
    assert_eq!(renderer.take_errors().len(), 1);
    renderer.capture(&scene, &frame).unwrap();
    assert!(renderer.take_errors().is_empty());
    scene.world.insert(entity, DynamicMesh { data: Arc::new(MeshData::primitive("cube").unwrap()), version: 2 });
    renderer.capture(&scene, &frame).unwrap();
    assert_eq!(renderer.stats().triangles, 12);
}
