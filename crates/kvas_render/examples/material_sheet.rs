//! Renders every built-in material on a sphere into one contact-sheet PNG (for the docs and as a visual check).
//! `cargo run --release -p kvas_render --example material_sheet -- docs/images/materials.png [width]`
use glam::{Mat4, Vec3};
use kvas_engine::{assets::Assets, components::*, hierarchy, material_presets::library, scene::Scene};
use kvas_render::{debug::DebugDraw, Frame, Renderer, View};
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let output = PathBuf::from(args.next().unwrap_or_else(|| "materials.png".into()));
    let width = args.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1600).clamp(320, 1920);
    let presets = library();
    // Materials are read from the editor's starter library, exactly as a new project gets them.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../kvas_editor/library");
    let assets = Assets::Dir(root);
    let mut scene = Scene::default();
    scene.settings.atmosphere.procedural = true;
    let columns = 11usize;
    let rows = presets.len().div_ceil(columns);
    for (i, p) in presets.iter().enumerate() {
        let (c, r) = ((i % columns) as f32, (i / columns) as f32);
        let e = hierarchy::spawn_named(&mut scene.world, p.name);
        scene.world.insert(e, MeshRenderer { mesh: "primitive:sphere".into(), shader: format!("materials/{}/{}.kmat", p.category, p.name), ..Default::default() });
        scene.world.insert(e, Transform::at(Vec3::new((c - (columns as f32 - 1.0) / 2.0) * 1.15, -(r - (rows as f32 - 1.0) / 2.0) * 1.15, 0.0)));
    }
    hierarchy::update_globals(&mut scene.world);
    let aspect = columns as f32 / rows as f32;
    let height = (width as f32 / aspect) as u32;
    // The grid has the frame's aspect ratio, so the vertical extent alone decides the distance.
    let eye = Vec3::new(0.0, 0.0, rows as f32 * 1.15 / 2.0 / 20.0f32.to_radians().tan() * 1.1);
    let view = View::new(Mat4::from_translation(eye), 40.0, aspect, 0.1, 200.0);
    let debug = DebugDraw::default();
    let frame = Frame { view, size: [width, height], time: 0.0, dt: 0.0, debug: &debug, shadows: false, debug_view: 0, cinematic: false };
    let mut renderer = Renderer::new_headless(assets)?;
    let image = renderer.capture(&scene, &frame)?;
    let errors = renderer.take_errors();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    image.save(&output).map_err(|e| e.to_string())?;
    println!("{} materials, saved {}x{} to {}", presets.len(), image.width(), image.height(), output.display());
    Ok(())
}
