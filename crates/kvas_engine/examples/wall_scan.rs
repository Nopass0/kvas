//! Finds the openings of LAST SERVICE's walls with the physics of the real meshes: rays across a
//! wall line every 5 cm at hip and head height; the free stretches are doorways.
//! `cargo run -p kvas_engine --example wall_scan`
use glam::Vec3;
use kvas_engine::{assets::Assets, registry::Registry, runtime::Runtime, scene::Scene};

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/last_service/assets");
    let assets = Assets::Dir(root);
    kvas_engine::meshmod::set_assets(assets.clone());
    let (mut scene, _) = Scene::from_json(&Registry::default(), &assets.read_string("scenes/main.scene.json").unwrap()).unwrap();
    // Only the production mesh: drop the game's own walls and doors.
    let own: Vec<_> = scene
        .world
        .entities()
        .into_iter()
        .filter(|e| {
            let n = kvas_engine::hierarchy::name_of(&scene.world, *e);
            n.starts_with("Wall ") || n.starts_with("Door ") || n == "Ceiling"
        })
        .collect();
    for e in own {
        scene.world.remove::<kvas_engine::components::Collider>(e);
    }
    let mut rt = Runtime::new(scene, None);
    rt.tick(1.0 / 60.0);
    // (wall, fixed coordinate, from, to, along x?)
    for (name, fixed, a, b, along_x) in [
        ("south z=0", 0.0f32, 0.0f32, 12.0f32, true),
        ("hall-kitchen z=-8", -8.0, 0.0, 12.0, true),
        ("office-storage z=-11", -11.0, 7.0, 12.0, true),
        ("kitchen-north z=-13", -13.0, 0.0, 7.0, true),
        ("west x=0", 0.0, 0.0, -15.0, false),
        ("x=7", 7.0, -8.0, -15.0, false),
        ("east x=12", 12.0, 0.0, -15.0, false),
    ] {
        let mut open: Vec<(f32, f32)> = Vec::new();
        let steps = ((b - a) as f32).abs() / 0.05;
        for i in 0..=steps as usize {
            let s = a + (b - a).signum() * i as f32 * 0.05;
            let free = [1.2f32, 1.8].iter().all(|h| {
                let (p, d) = if along_x { (Vec3::new(s, *h, fixed + 0.3), Vec3::NEG_Z) } else { (Vec3::new(fixed - 0.3, *h, s), Vec3::X) };
                rt.physics.raycast(p, d, 0.6).is_none()
            });
            if free {
                match open.last_mut() {
                    Some((_, end)) if (s - *end).abs() < 0.06 => *end = s,
                    _ => open.push((s, s)),
                }
            }
        }
        let gaps: Vec<String> = open.iter().filter(|(x, y)| (y - x).abs() > 0.5).map(|(x, y)| format!("{:.2}..{:.2} (centre {:.2}, width {:.2})", x, y, (x + y) / 2.0, (y - x).abs())).collect();
        println!("{name}: {}", if gaps.is_empty() { "no opening".to_string() } else { gaps.join(", ") });
    }
}
