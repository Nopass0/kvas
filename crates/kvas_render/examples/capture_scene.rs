//! Windowless PNG export through the same renderer used by the editor/player.
//! Run: cargo run -p kvas_render --example capture_scene -- output.png [project-folder] [scene-path] [width]
use glam::Vec3;
use kvas_engine::{assets::Assets, components::*, hierarchy, project::Project, registry::Registry, scene::Scene};
use kvas_render::{debug::DebugDraw, Frame, Renderer, View};
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let output = PathBuf::from(args.next().unwrap_or_else(|| "capture.png".into()));
    let (mut scene, assets) = match args.next() {
        Some(root) => {
            let project = Project::open(&PathBuf::from(root))?;
            let assets = Assets::Dir(project.assets_dir());
            let scene_path = args.next().unwrap_or(project.config.start_scene);
            let (scene, warnings) =
                Scene::from_json(&Registry::default(), &assets.read_string(&scene_path).map_err(|e| e.to_string())?)?;
            for warning in warnings {
                eprintln!("{warning}");
            }
            (scene, assets)
        }
        None => {
            let mut scene = Scene::default();
            scene.settings.atmosphere.procedural = true;
            let cube = scene.spawn("Cube");
            scene.world.insert(cube, MeshRenderer { color: [0.15, 0.7, 1.0, 1.0], ..Default::default() });
            scene.world.insert(cube, Transform { position: Vec3::Y * 0.8, rotation: Vec3::new(12.0, 35.0, 0.0), ..Default::default() });
            let ground = scene.spawn("Ground");
            scene
                .world
                .insert(ground, MeshRenderer { mesh: "primitive:plane".into(), color: [0.32, 0.4, 0.34, 1.0], ..Default::default() });
            scene.world.insert(ground, Transform { scale: Vec3::splat(12.0), ..Default::default() });
            (scene, Assets::Dir(std::env::current_dir().map_err(|e| e.to_string())?))
        }
    };
    // Rebuild editor-mode procedural geometry without running gameplay scripts.
    kvas_engine::procedural::update(&mut scene.world, 0.0, 0.0, false);
    hierarchy::update_globals(&mut scene.world);
    // Two seconds of particles, so emitters show up as they would in a running game.
    for step in 0..60 {
        kvas_engine::particles::update(&mut scene.world, step as f32 / 30.0, 1.0 / 30.0, true);
    }
    let mut camera = Transform::at(Vec3::new(4.5, 3.0, 6.0));
    camera.look_at(Vec3::Y * 0.6, Vec3::Y);
    let view = kvas_render::game_view(&scene, 16.0 / 9.0).unwrap_or_else(|| View::new(camera.matrix(), 50.0, 16.0 / 9.0, 0.05, 1000.0));
    let debug = DebugDraw::default();
    let width = args.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(640).clamp(320, 1920);
    let frame = Frame { view, size: [width, width * 9 / 16], time: 0.0, dt: 0.0, debug: &debug, shadows: true, debug_view: 0, cinematic: true };
    let mut renderer = Renderer::new_headless(assets)?;
    let image = renderer.capture(&scene, &frame)?;
    let errors = renderer.take_errors();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    image.save(&output).map_err(|e| format!("{}: {e}", output.display()))?;
    println!("Saved {}x{} to {}", image.width(), image.height(), output.display());
    Ok(())
}
