//! Runs the simulation and the renderer for many frames and prints how the frame time and the
//! entity count change. A flat line means no per-frame growth.
//! `cargo run --release -p kvas_render --example leak_probe -- games/demo [frames]`
use glam::Vec3;
use kvas_engine::{assets::Assets, project::Project, registry::Registry, runtime::Runtime, scene::Scene};
use kvas_render::{debug::DebugDraw, Frame, Renderer, View};
use std::{path::PathBuf, time::Instant};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let root = args.next().unwrap_or_else(|| "games/demo".into());
    let frames: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(3000);
    let project = Project::open(&PathBuf::from(root))?;
    let assets = Assets::Dir(project.assets_dir());
    let (scene, _) = Scene::from_json(&Registry::default(), &assets.read_string(&project.config.start_scene).map_err(|e| e.to_string())?)?;
    let mut runtime = Runtime::new(scene, None);
    let mut renderer = Renderer::new_headless(assets)?;
    let debug = DebugDraw::default();
    let (mut sim, mut draw) = (0.0f64, 0.0f64);
    for frame in 1..=frames {
        let t = Instant::now();
        runtime.tick(1.0 / 60.0);
        sim += t.elapsed().as_secs_f64();
        let view = kvas_render::game_view(&runtime.scene, 16.0 / 9.0).unwrap_or_else(|| View::new(glam::Mat4::from_translation(Vec3::new(0.0, 3.0, 8.0)), 50.0, 16.0 / 9.0, 0.05, 1000.0));
        let f = Frame { view, size: [320, 180], time: runtime.time.elapsed, dt: 1.0 / 60.0, debug: &debug, shadows: true, debug_view: 0, cinematic: false };
        let t = Instant::now();
        renderer.capture(&runtime.scene, &f)?;
        draw += t.elapsed().as_secs_f64();
        if frame % (frames / 6).max(1) == 0 {
            println!("frame {frame:>6}: sim {:.3} ms, render {:.2} ms, entities {}", sim / (frames / 6).max(1) as f64 * 1000.0, draw / (frames / 6).max(1) as f64 * 1000.0, runtime.scene.world.len());
            sim = 0.0;
            draw = 0.0;
        }
    }
    Ok(())
}
