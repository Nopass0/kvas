//! Small reproducible CPU smoke benchmark; measurements are descriptive, not CI thresholds.
//! Run: cargo run -p kvas_engine --release --example bench_core
use glam::Vec3;
use kvas_engine::{
    components::{GlobalTransform, Name, Parent, Transform},
    hierarchy,
    registry::Registry,
    scene::Scene,
};
use std::time::Instant;

fn main() {
    let registry = Registry::default();
    for count in [1_000usize, 5_000, 10_000] {
        let mut scene = Scene::default();
        let mut parent = None;
        for index in 0..count {
            let entity = scene.spawn(&format!("node-{index}"));
            scene.world.get_mut::<Transform>(entity).unwrap().position = Vec3::X * 0.001;
            // The new entity has no children, so this fresh chain is known to be
            // acyclic. Avoid measuring validation of unrelated set_parent calls.
            if let Some(parent) = parent {
                scene.world.insert(entity, Parent(parent));
            }
            parent = Some(entity);
        }
        let mut best_update = std::time::Duration::MAX;
        for _ in 0..5 {
            let start = Instant::now();
            hierarchy::update_globals(&mut scene.world);
            best_update = best_update.min(start.elapsed());
        }
        let tip = parent.unwrap();
        let position = scene.world.get::<GlobalTransform>(tip).unwrap().0.w_axis.x;
        assert!((position - count as f32 * 0.001).abs() < 0.01, "deep hierarchy lost transforms: {position}");
        let start = Instant::now();
        let matched = scene.world.iter2::<Name, Transform>().count();
        let join_time = start.elapsed();
        assert_eq!(matched, count);
        let start = Instant::now();
        let json = scene.try_to_json(&registry).unwrap();
        let save_time = start.elapsed();
        let start = Instant::now();
        let (loaded, warnings) = Scene::from_json(&registry, &json).unwrap();
        let load_time = start.elapsed();
        assert!(warnings.is_empty());
        assert_eq!(loaded.world.len(), count);
        println!(
            "{count:>5} entities: globals={best_update:?}, join={join_time:?}, JSON save={save_time:?}, load={load_time:?}, bytes={}",
            json.len()
        );
    }
}
