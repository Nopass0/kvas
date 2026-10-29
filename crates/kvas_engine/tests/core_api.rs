//! Headless regression coverage for script-facing engine services.
use glam::{Vec2, Vec3};
use kvas_engine::{
    components::*,
    hierarchy,
    input::{Input, Key, MouseButton},
    registry::Registry,
    runtime::{Runtime, RuntimeConfig},
    scene::{self, Scene},
};

#[test]
fn deep_hierarchies_have_no_arbitrary_depth_cutoff() {
    let mut scene = Scene::default();
    let root = scene.spawn("root");
    scene.world.get_mut::<Transform>(root).unwrap().position.x = 1.0;
    let mut tip = root;
    for _ in 0..150 {
        let child = scene.spawn("child");
        scene.world.get_mut::<Transform>(child).unwrap().position.x = 1.0;
        assert!(hierarchy::set_parent_local(&mut scene.world, child, Some(tip)));
        tip = child;
    }
    assert_eq!(hierarchy::ancestors_count(&scene.world, tip), 150);
    assert_eq!(hierarchy::global_matrix(&scene.world, tip).w_axis.x, 151.0);
    assert!(!hierarchy::set_parent_local(&mut scene.world, root, Some(tip)));
    assert_eq!(hierarchy::descendants(&scene.world, root).len(), 150);
}

#[test]
fn malformed_cycles_are_bounded_and_can_be_deleted() {
    let mut scene = Scene::default();
    let a = scene.spawn("a");
    let b = scene.spawn("b");
    scene.world.insert(a, Parent(b));
    scene.world.insert(b, Parent(a));
    assert_eq!(hierarchy::descendants(&scene.world, a), vec![b]);
    assert_eq!(hierarchy::ancestors_count(&scene.world, a), 1);
    assert!(hierarchy::global_matrix(&scene.world, a).is_finite());
    hierarchy::despawn_recursive(&mut scene.world, a);
    assert!(scene.world.is_empty());
}

#[test]
fn reparenting_rejects_stale_handles_and_singular_matrices_without_mutation() {
    let mut scene = Scene::default();
    let child = scene.spawn("child");
    let parent = scene.spawn("parent");
    scene.world.get_mut::<Transform>(parent).unwrap().scale = Vec3::ZERO;
    assert!(!hierarchy::set_parent(&mut scene.world, child, Some(parent)));
    assert!(hierarchy::parent_of(&scene.world, child).is_none());
    assert!(hierarchy::set_parent_local(&mut scene.world, child, Some(parent)));
    scene.world.despawn(parent);
    assert!(hierarchy::roots(&scene.world).contains(&child));
    assert!(!hierarchy::set_parent_local(&mut scene.world, child, Some(parent)));
    assert!(!hierarchy::set_parent(&mut scene.world, parent, None));
}

#[test]
fn loader_repairs_bad_hierarchy_and_reports_warnings() {
    let reg = Registry::default();
    let text = r#"{"entities":[{"parent":1,"components":[]},{"parent":0,"components":[]},{"parent":42,"components":[]}]}"#;
    let (mut scene, warnings) = Scene::from_json(&reg, text).unwrap();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert_eq!(hierarchy::roots(&scene.world).len(), 2);
    for e in scene.world.entities() {
        hierarchy::despawn_recursive(&mut scene.world, e);
    }
    assert!(scene.world.is_empty());
}

#[test]
fn duplication_omits_transients_and_keeps_child_names_and_local_pose() {
    let mut scene = Scene::default();
    let reg = Registry::default();
    let a = scene.spawn("parent");
    let b = scene.spawn("дочерний");
    let hidden = scene.spawn("hidden");
    hierarchy::set_parent_local(&mut scene.world, b, Some(a));
    hierarchy::set_parent_local(&mut scene.world, hidden, Some(a));
    scene.world.insert(hidden, Transient);
    scene.world.get_mut::<Transform>(b).unwrap().position = Vec3::X;
    let copy = scene::duplicate(&mut scene.world, &reg, a);
    assert_eq!(hierarchy::name_of(&scene.world, copy), "parent (Copy)");
    let children = hierarchy::children_all(&scene.world, copy);
    assert_eq!(children.len(), 1);
    assert_eq!(hierarchy::name_of(&scene.world, children[0]), "дочерний");
    assert_eq!(scene.world.get::<Transform>(children[0]).unwrap().position, Vec3::X);
    let (snapshot, warnings) = scene.snapshot(&reg).unwrap();
    assert!(warnings.is_empty());
    assert_eq!(snapshot.world.len(), 4);
}

#[test]
fn input_releases_only_after_all_action_alternatives_are_up() {
    let mut input = Input::default();
    input.try_rebind("Fire", &["Space", "MouseLeft"]).unwrap();
    input.update([Key::Space].into_iter().collect(), [MouseButton::Left].into_iter().collect(), Vec2::ONE, 1.0);
    assert!(input.action_pressed("Fire"));
    input.update(Default::default(), [MouseButton::Left].into_iter().collect(), Vec2::ZERO, 0.0);
    assert!(input.key_released(Key::Space));
    assert!(!input.action_released("Fire"));
    input.update(Default::default(), Default::default(), Vec2::ZERO, 0.0);
    assert!(input.mouse_released(MouseButton::Left));
    assert!(input.action_released("Fire"));
    assert!(input.try_rebind("Fire", &["NotAKey"]).is_err());
    assert_eq!(input.bindings()[0].1, vec!["Space", "MouseLeft"]);
    input.lock_cursor(true);
    input.reset();
    assert!(!input.cursor_locked());
    assert!(!input.action_released("Fire"));
    assert!(input.has_action("Fire"));
    assert!(input.remove_action("Fire"));
    assert!(!input.remove_action("Fire"));
}

#[test]
fn diagonal_movement_is_normalized_and_opposites_cancel() {
    let mut input = Input::default();
    for (name, source) in [("left", "A"), ("right", "D"), ("down", "S"), ("up", "W")] {
        input.rebind(name, &[source]);
    }
    input.update([Key::D, Key::W].into_iter().collect(), Default::default(), Vec2::ZERO, 0.0);
    assert!((input.action_vector("left", "right", "down", "up").length() - 1.0).abs() < 1e-6);
    input.update([Key::A, Key::D].into_iter().collect(), Default::default(), Vec2::ZERO, 0.0);
    assert_eq!(input.action_vector("left", "right", "down", "up"), Vec2::ZERO);
}

#[test]
fn runtime_pause_step_and_time_scale_are_predictable() {
    let mut runtime = Runtime::new(Scene::default(), None);
    for dt in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        runtime.tick(dt);
    }
    assert_eq!(runtime.time.frame, 0);
    runtime.set_paused(true);
    runtime.tick(0.1);
    assert_eq!(runtime.time.frame, 0);
    runtime.step_once();
    assert_eq!(runtime.time.frame, 1);
    assert_eq!(runtime.frame_stats().fixed_steps, 1);
    assert!(runtime.is_paused());
    runtime.set_paused(false);
    runtime.set_time_scale(0.5).unwrap();
    runtime.tick(0.1);
    assert!((runtime.time.dt - 0.05).abs() < 1e-6);
    assert!(runtime.set_time_scale(f32::NAN).is_err());
    assert_eq!(runtime.time_scale(), 0.5);
    runtime.set_time_scale(0.0).unwrap();
    let frame = runtime.time.frame;
    runtime.tick(0.1);
    assert_eq!(runtime.time.frame, frame);
}

#[test]
fn runtime_bounds_catchup_and_preserves_fractional_time() {
    let mut runtime =
        Runtime::with_config(Scene::default(), None, RuntimeConfig { fixed_dt: 0.01, max_frame_dt: 1.0, max_fixed_steps: 2 }).unwrap();
    runtime.tick(0.105);
    let stats = runtime.frame_stats();
    assert_eq!(stats.fixed_steps, 2);
    assert!((stats.dropped_time - 0.08).abs() < 1e-5, "{stats:?}");
    assert!((stats.interpolation_alpha - 0.5).abs() < 1e-4, "{stats:?}");
    assert!(RuntimeConfig { fixed_dt: 0.0, ..Default::default() }.validate().is_err());
    assert!(RuntimeConfig { max_fixed_steps: 0, ..Default::default() }.validate().is_err());
}

#[test]
fn transform_helpers_preserve_units_and_reject_degenerate_poses() {
    use glam::{Mat4, Quat};
    let t = Transform { position: Vec3::new(1.0, 2.0, 3.0), rotation: Vec3::new(0.0, 90.0, 0.0), scale: Vec3::splat(2.0) };
    let point = Vec3::new(1.0, -1.0, 2.0);
    assert!((t.inverse_transform_point(t.transform_point(point)).unwrap() - point).length() < 1e-5);
    assert!((t.transform_direction(Vec3::NEG_Z).length() - 1.0).abs() < 1e-6);
    assert!(Transform::try_from_matrix(t.matrix()).is_some());
    assert!(Transform::try_from_matrix(Mat4::ZERO).is_none());
    let mut pose = Transform::default();
    assert!(!pose.look_at(Vec3::ZERO, Vec3::Y));
    assert!(!pose.look_at(Vec3::Y, Vec3::Y));
    assert!(pose.look_at(Vec3::X, Vec3::Y));
    assert!((pose.forward() - Vec3::X).length() < 1e-5);
    let a = Transform { rotation: Vec3::new(0.0, 170.0, 0.0), ..Default::default() };
    let b = Transform { rotation: Vec3::new(0.0, -170.0, 0.0), ..Default::default() };
    let halfway = a.interpolate(&b, 0.5);
    assert!(halfway.forward().dot(Quat::from_rotation_y(std::f32::consts::PI) * Vec3::NEG_Z) > 0.999);
}

#[test]
fn mesh_operations_check_topology_and_keep_mirrored_winding() {
    use glam::Mat4;
    use kvas_engine::mesh::MeshData;
    let mut cube = MeshData::primitive("cube").unwrap();
    cube.validate().unwrap();
    assert!((cube.surface_area().unwrap() - 6.0).abs() < 1e-5);
    let mirrored = cube.transformed(Mat4::from_scale(Vec3::new(-2.0, 1.0, 1.0))).unwrap();
    for tri in mirrored.indices.as_chunks::<3>().0.iter() {
        let a = &mirrored.vertices[tri[0] as usize];
        let b = &mirrored.vertices[tri[1] as usize];
        let c = &mirrored.vertices[tri[2] as usize];
        let geometric = (Vec3::from(b.pos) - Vec3::from(a.pos)).cross(Vec3::from(c.pos) - Vec3::from(a.pos));
        assert!(geometric.dot(Vec3::from(a.normal)) > 0.0);
    }
    cube.append(&mirrored).unwrap();
    assert_eq!(cube.triangle_count(), 24);
    cube.recalculate_normals().unwrap();
    cube.validate().unwrap();
    let before = cube.vertices.len();
    let mut bad = MeshData::primitive("cube").unwrap();
    bad.indices[0] = u32::MAX;
    assert!(cube.append(&bad).is_err());
    assert_eq!(cube.vertices.len(), before);
    assert!(bad.recalculate_normals().is_err());
}

#[test]
fn procedural_key_roundtrips_retain_all_integer_seed_bits() {
    use kvas_engine::{
        mesh::ProcParams,
        noise::{Noise, NoiseTexture},
        voxel::CloudVolume,
    };
    let params = ProcParams {
        kind: "sphere".into(),
        size: Vec3::ONE,
        radius: 1.0,
        radius2: 0.5,
        height: 1.0,
        segments: 24,
        rings: 16,
        seed: u32::MAX - 1,
    };
    assert_eq!(ProcParams::parse(&params.key()), Some(params));
    let texture = NoiseTexture::new(Noise { seed: u32::MAX - 1, ..Default::default() });
    assert_eq!(NoiseTexture::parse(&texture.key()), Some(texture));
    let cloud = CloudVolume { size: Vec3::ONE, cell: 1.0, coverage: 0.5, scale: 1.0, seed: i32::MIN + 1 };
    assert_eq!(CloudVolume::parse(&cloud.key()), Some(cloud));
    assert!(ProcParams::parse("proc:sphere:1,1,1,NaN,1,1,24,16,42").is_none());
    assert!(CloudVolume::parse("cloudvol:1,1,1,1,0.5,1,42,garbage").is_none());
    let noise = Noise { octaves: u32::MAX, ..Default::default() };
    assert!(noise.sample3(f32::MAX, f32::MAX, f32::MAX).is_finite());
    assert!(noise.try_image(u32::MAX, 1, [0.0; 3], [1.0; 3]).is_err());
}

#[test]
fn reflection_rejects_numeric_overflow_and_reports_unknown_fields() {
    use kvas_engine::reflect::{apply_props_checked, Field};
    let mut integer = 12;
    let mut real = 2.0;
    {
        let mut fields = vec![Field::i32("count", &mut integer), Field::f32("speed", &mut real)];
        let props = serde_json::json!({"count": i64::MAX, "speed": 1e100, "typo": true});
        assert_eq!(apply_props_checked(&mut fields, props.as_object().unwrap()).len(), 3);
    }
    assert_eq!(integer, 12);
    assert_eq!(real, 2.0);
}

#[test]
fn registry_named_edits_preserve_existing_data_and_enforce_removal_policy() {
    let reg = Registry::default();
    let mut scene = Scene::default();
    let e = scene.spawn("original");
    assert!(!reg.add_named(&mut scene.world, e, "Name").unwrap());
    assert_eq!(hierarchy::name_of(&scene.world, e), "original");
    assert!(reg.remove_named(&mut scene.world, e, "Transform").is_err());
    assert!(reg.add_named(&mut scene.world, e, "MeshRenderer").unwrap());
    assert_eq!(reg.of_entity(&scene.world, e).len(), 3);
    assert!(reg.remove_named(&mut scene.world, e, "MeshRenderer").unwrap());
    assert!(!reg.remove_named(&mut scene.world, e, "MeshRenderer").unwrap());
    assert!(reg.add_named(&mut scene.world, e, "Typo").is_err());
}

#[test]
fn removed_physics_components_and_live_collider_edits_reach_rapier() {
    use kvas_engine::physics::Physics;
    let mut scene = Scene::default();
    let e = scene.spawn("box");
    scene.world.insert(e, RigidBody { kind: BodyKind::Fixed, ..Default::default() });
    scene.world.insert(e, Collider::default());
    let mut physics = Physics::new();
    physics.step(&mut scene, 1.0 / 60.0);
    assert_eq!(physics.body_count(), 1);
    assert_eq!(physics.collider_count(), 1);
    scene.world.get_mut::<Transform>(e).unwrap().position.x = 10.0;
    scene.world.get_mut::<Collider>(e).unwrap().size = Vec3::splat(4.0);
    physics.step(&mut scene, 1.0 / 60.0);
    assert!(physics.raycast(Vec3::new(0.0, 10.0, 0.0), Vec3::NEG_Y, 20.0).is_none());
    assert_eq!(physics.raycast(Vec3::new(10.0, 10.0, 0.0), Vec3::NEG_Y, 20.0).unwrap().entity, e);
    assert_eq!(physics.overlap_sphere(Vec3::new(12.5, 0.0, 0.0), 1.0), vec![e]);
    scene.world.remove::<RigidBody>(e);
    physics.step(&mut scene, 1.0 / 60.0);
    assert_eq!(physics.body_count(), 0);
    assert_eq!(physics.collider_count(), 1);
    scene.world.remove::<Collider>(e);
    physics.step(&mut scene, 1.0 / 60.0);
    assert_eq!(physics.collider_count(), 0);
    assert!(physics.overlap_sphere(Vec3::ZERO, 100.0).is_empty());
}

#[test]
fn a_collider_on_a_child_follows_its_turning_parent() {
    // A door: the hinge turns, the collider is a child of the hinge.
    use kvas_engine::physics::Physics;
    let mut scene = Scene::default();
    let hinge = scene.spawn("hinge");
    let leaf = scene.spawn("leaf");
    scene.world.insert(leaf, Transform { position: Vec3::new(0.5, 1.0, 0.0), ..Default::default() });
    scene.world.insert(leaf, Collider { size: Vec3::new(1.0, 2.0, 0.1), ..Default::default() });
    hierarchy::set_parent_local(&mut scene.world, leaf, Some(hinge));
    let mut physics = Physics::new();
    physics.step(&mut scene, 1.0 / 60.0);
    let through = |p: &Physics| p.raycast(Vec3::new(0.5, 1.0, 2.0), Vec3::NEG_Z, 4.0).map(|h| h.entity);
    assert_eq!(through(&physics), Some(leaf), "closed door blocks");
    scene.world.get_mut::<Transform>(hinge).unwrap().rotation.y = 90.0;
    physics.step(&mut scene, 1.0 / 60.0);
    assert_eq!(through(&physics), None, "the opened door no longer blocks the doorway");
}

#[test]
fn mesh_colliders_follow_the_model_triangles() {
    use kvas_engine::physics::Physics;
    let mut scene = Scene::default();
    let floor = scene.spawn("floor");
    scene.world.get_mut::<Transform>(floor).unwrap().scale = Vec3::new(10.0, 1.0, 10.0);
    scene.world.insert(floor, MeshRenderer { mesh: "primitive:plane".into(), ..Default::default() });
    scene.world.insert(floor, Collider { shape: ShapeKind::Mesh, ..Default::default() });
    let ball = scene.spawn("ball");
    scene.world.get_mut::<Transform>(ball).unwrap().position = Vec3::new(20.0, 0.0, 0.0);
    scene.world.insert(ball, MeshRenderer { mesh: "primitive:sphere".into(), ..Default::default() });
    scene.world.insert(ball, Collider { shape: ShapeKind::Hull, size: Vec3::splat(2.0), ..Default::default() });
    let mut physics = Physics::new();
    physics.step(&mut scene, 1.0 / 60.0);
    // A box fallback would be one metre tall; the plane's triangles are flat.
    let hit = physics.raycast(Vec3::new(3.0, 5.0, 3.0), Vec3::NEG_Y, 10.0).unwrap();
    assert_eq!(hit.entity, floor);
    assert!(hit.point.y.abs() < 0.01, "{:?}", hit.point);
    assert!(physics.raycast(Vec3::new(6.0, 5.0, 0.0), Vec3::NEG_Y, 10.0).is_none());
    let hit = physics.raycast(Vec3::new(20.0, 5.0, 0.0), Vec3::NEG_Y, 10.0).unwrap();
    assert_eq!(hit.entity, ball);
    assert!((hit.point.y - 1.0).abs() < 0.05, "{:?}", hit.point);
}

#[test]
fn physics_sensor_queries_and_invalid_rays_are_safe() {
    use kvas_engine::physics::Physics;
    let mut scene = Scene::default();
    let trigger = scene.spawn("trigger");
    let body = scene.spawn("body");
    scene.world.insert(trigger, Collider { is_trigger: true, size: Vec3::splat(5.0), ..Default::default() });
    scene.world.insert(body, Collider::default());
    scene.world.insert(body, RigidBody { gravity_scale: 0.0, ..Default::default() });
    let mut physics = Physics::new();
    physics.step(&mut scene, 1.0 / 60.0);
    assert_eq!(physics.touching(trigger), vec![body]);
    assert_eq!(physics.touching(body), vec![trigger]);
    for range in [f32::NAN, f32::INFINITY, -1.0] {
        assert!(physics.raycast(Vec3::ZERO, Vec3::Y, range).is_none());
    }
    physics.reset();
    assert_eq!(physics.body_count(), 0);
    assert!(physics.touching(trigger).is_empty());
}

#[test]
fn gameplay_components_roundtrip_and_expiration_respects_pause_and_children() {
    use kvas_engine::gameplay::{self, Lifetime, Tags, Visibility};
    let mut scene = Scene::default();
    let root = scene.spawn("effect");
    let child = scene.spawn("child");
    hierarchy::set_parent_local(&mut scene.world, child, Some(root));
    let mut tags = Tags::default();
    assert!(tags.insert("enemy"));
    assert!(tags.insert("damageable"));
    assert!(!tags.insert("enemy"));
    assert!(!tags.insert("two words"));
    assert!(tags.remove("damageable"));
    assert!(!tags.contains("damageable"));
    scene.world.insert(root, tags);
    scene.world.insert(root, Lifetime::new(0.05).unwrap());
    scene.world.insert(root, Visibility { visible: false });
    assert!(!hierarchy::is_visible(&scene.world, child));
    assert_eq!(gameplay::tagged(&scene.world, "enemy"), vec![root]);
    let (loaded, warnings) = scene.snapshot(&Registry::default()).unwrap();
    assert!(warnings.is_empty());
    assert_eq!(gameplay::tagged(&loaded.world, "enemy").len(), 1);
    let mut runtime = Runtime::new(scene, None);
    runtime.set_paused(true);
    runtime.tick(0.1);
    assert!(runtime.scene.world.is_alive(root));
    runtime.set_paused(false);
    runtime.tick(0.1);
    assert!(!runtime.scene.world.is_alive(root));
    assert!(!runtime.scene.world.is_alive(child));
}

#[test]
fn cached_globals_match_direct_evaluation_with_transformless_ancestors() {
    let mut scene = Scene::default();
    let root = scene.spawn("root");
    scene.world.get_mut::<Transform>(root).unwrap().position = Vec3::new(3.0, 2.0, 1.0);
    let middle = scene.world.spawn();
    scene.world.insert(middle, Parent(root));
    let child = scene.spawn("child");
    hierarchy::set_parent_local(&mut scene.world, child, Some(middle));
    scene.world.get_mut::<Transform>(child).unwrap().rotation.y = 40.0;
    hierarchy::update_globals(&mut scene.world);
    for entity in scene.world.entities_with::<Transform>() {
        assert!(scene.world.get::<GlobalTransform>(entity).unwrap().0.abs_diff_eq(hierarchy::global_matrix(&scene.world, entity), 1e-5));
    }
}

#[test]
fn collision_masks_filter_sensor_pairs_and_queries_and_can_change_at_runtime() {
    use kvas_engine::physics::{Physics, QueryOptions};
    let mut scene = Scene::default();
    let sensor = scene.spawn("sensor");
    let body = scene.spawn("body");
    scene.world.insert(sensor, Collider { is_trigger: true, size: Vec3::splat(3.0), memberships: 1, filter: 1, ..Default::default() });
    scene.world.insert(body, Collider { memberships: 2, filter: 2, ..Default::default() });
    scene.world.insert(body, RigidBody { gravity_scale: 0.0, ..Default::default() });
    let mut physics = Physics::new();
    physics.step(&mut scene, 1.0 / 60.0);
    assert!(physics.touching(sensor).is_empty());
    scene.world.get_mut::<Collider>(sensor).unwrap().filter = 2;
    scene.world.get_mut::<Collider>(body).unwrap().filter = 1;
    physics.step(&mut scene, 1.0 / 60.0);
    assert_eq!(physics.touching(sensor), vec![body]);
    let options = QueryOptions { memberships: 1, filter: 2, ..Default::default() };
    assert_eq!(physics.overlap_sphere_with_options(Vec3::ZERO, 10.0, options), vec![body]);
    let hit = physics.raycast_with_options(Vec3::Y * 10.0, Vec3::NEG_Y, 20.0, options).unwrap();
    assert_eq!(hit.entity, body);
    let disabled = QueryOptions { memberships: 0, filter: 0, ..Default::default() };
    assert!(physics.overlap_sphere_with_options(Vec3::ZERO, 10.0, disabled).is_empty());
}

#[test]
fn excluding_a_body_also_excludes_its_compound_child_colliders() {
    use kvas_engine::physics::Physics;
    let mut scene = Scene::default();
    let owner = scene.spawn("owner");
    let child = scene.spawn("own collider");
    let target = scene.spawn("target");
    scene.world.insert(owner, RigidBody { kind: BodyKind::Fixed, ..Default::default() });
    hierarchy::set_parent_local(&mut scene.world, child, Some(owner));
    scene.world.get_mut::<Transform>(child).unwrap().position.y = 2.0;
    scene.world.get_mut::<Transform>(target).unwrap().position.y = -3.0;
    scene.world.insert(child, Collider::default());
    scene.world.insert(target, Collider::default());
    let mut physics = Physics::new();
    physics.step(&mut scene, 1.0 / 60.0);
    assert_eq!(physics.raycast(Vec3::Y * 5.0, Vec3::NEG_Y, 10.0).unwrap().entity, child);
    assert_eq!(physics.raycast_excluding(Vec3::Y * 5.0, Vec3::NEG_Y, 10.0, Some(owner)).unwrap().entity, target);
}

#[test]
fn ccd_stops_a_fast_body_at_a_thin_wall() {
    use kvas_engine::physics::Physics;
    let mut scene = Scene::default();
    let wall = scene.spawn("wall");
    let bullet = scene.spawn("bullet");
    scene.world.insert(wall, Collider { size: Vec3::new(0.1, 10.0, 10.0), restitution: 0.0, ..Default::default() });
    scene.world.get_mut::<Transform>(bullet).unwrap().position.x = -5.0;
    scene.world.insert(bullet, Collider { size: Vec3::splat(0.1), restitution: 0.0, ..Default::default() });
    scene.world.insert(
        bullet,
        RigidBody { gravity_scale: 0.0, ccd: true, linear_damping: 0.0, set_velocity: Some(Vec3::X * 1000.0), ..Default::default() },
    );
    let mut physics = Physics::new();
    physics.step(&mut scene, 1.0 / 60.0);
    let x = scene.world.get::<Transform>(bullet).unwrap().position.x;
    assert!(x < 0.1 && x > -5.0, "CCD should stop tunnelling through the wall, x={x}");
}

#[test]
fn unsigned_reflection_preserves_all_mask_bits() {
    use kvas_engine::reflect::Field;
    let mut mask = 0u32;
    {
        let mut field = Field::u32("mask", &mut mask);
        assert!(field.try_set_json(&serde_json::json!(u32::MAX)));
        assert_eq!(field.to_json(), serde_json::json!(u32::MAX));
        assert!(!field.try_set_json(&serde_json::json!(-1)));
        assert!(!field.try_set_json(&serde_json::json!(u64::MAX)));
    }
    assert_eq!(mask, u32::MAX);
}

#[test]
fn edited_joint_targets_are_rebuilt_and_removing_components_cleans_constraints() {
    use kvas_engine::physics::Physics;
    let mut scene = Scene::default();
    let a = scene.spawn("a");
    let b = scene.spawn("b");
    for entity in [a, b] {
        scene.world.insert(entity, RigidBody { kind: BodyKind::Fixed, ..Default::default() });
    }
    scene.world.insert(a, Joint { kind: JointKind::Ball, target: "b".into(), ..Default::default() });
    let mut physics = Physics::new();
    physics.step(&mut scene, 0.01);
    assert_eq!(physics.joint_count(), 1);
    scene.world.get_mut::<Joint>(a).unwrap().target = "a".into();
    physics.step(&mut scene, 0.01);
    assert_eq!(physics.joint_count(), 0);
    scene.world.get_mut::<Joint>(a).unwrap().target.clear();
    physics.step(&mut scene, 0.01);
    assert_eq!(physics.joint_count(), 1);
    scene.world.remove::<Joint>(a);
    physics.step(&mut scene, 0.01);
    assert_eq!(physics.joint_count(), 0);
    assert_eq!(physics.body_count(), 2);
    scene.world.insert(a, Joint { target: "b".into(), ..Default::default() });
    physics.step(&mut scene, 0.01);
    assert_eq!(physics.joint_count(), 1);
    scene.world.remove::<RigidBody>(b);
    physics.step(&mut scene, 0.01);
    assert_eq!(physics.joint_count(), 0);
}

#[test]
fn tiny_valid_scales_are_not_misclassified_as_singular() {
    use glam::Mat4;
    let pose = Transform::try_from_matrix(Mat4::from_scale(Vec3::splat(0.001))).unwrap();
    assert!(pose.inverse_transform_point(Vec3::ONE).is_some());
    let mut scene = Scene::default();
    let parent = scene.spawn("tiny");
    let child = scene.spawn("child");
    scene.world.insert(parent, pose);
    assert!(hierarchy::set_parent(&mut scene.world, child, Some(parent)));
    assert!(hierarchy::global_matrix(&scene.world, child).abs_diff_eq(Mat4::IDENTITY, 1e-4));
}

#[test]
fn permanent_host_component_extensions_load_into_a_fresh_world() {
    #[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
    struct Score { value: u32 }
    impl kvas_ecs::Component for Score { const NAME: &'static str = "host.Score"; }
    impl kvas_engine::components::Component for Score {}
    let mut scene = Scene::default(); let mut registry = Registry::default();
    registry.register::<Score>(&mut scene.world);
    let entity = scene.spawn("scored"); scene.world.insert(entity, Score { value: 123 });
    let (loaded, warnings) = scene.snapshot(&registry).unwrap(); assert!(warnings.is_empty());
    let entity = loaded.find("scored").unwrap(); assert_eq!(loaded.world.get::<Score>(entity).unwrap().value, 123);
}
