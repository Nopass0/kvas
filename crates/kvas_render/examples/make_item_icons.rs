//! Renders inventory icons of LAST SERVICE from the item models themselves (a lit three-quarter
//! view on a dark tile), so the hotbar shows the very plate or key lying in the world.
//! `cargo run -p kvas_render --example make_item_icons` (assets first: `python tools/import_last_service.py`).
use glam::Vec3;
use kvas_engine::{assets::Assets, components::*, hierarchy, mesh::MeshData, scene::Scene};
use kvas_render::{debug::DebugDraw, Frame, Renderer, View};

/// (icon name, model, texture, tint)
const ITEMS: &[(&str, &str, &str, [f32; 4])] = &[
    ("plate_dirty", "models/kitchen/dirty_plate.glb", "", [0.78, 0.68, 0.55, 1.0]),
    ("plate_clean", "models/kitchen/clean_plate.glb", "", [1.0; 4]),
    ("water", "models/kitchen/glass_of_water.glb", "", [1.0; 4]),
    ("bread", "models/kitchen/toast.glb", "", [1.0; 4]),
    ("chocolate", "models/kitchen/chocolate_bar_close.glb", "", [1.0; 4]),
    ("fuse", "models/misc/cardboard_box_1.glb", "models/textures/cardboard_box_1.png", [1.0, 0.85, 0.35, 1.0]),
    ("keycard", "models/props/keycard_red.glb", "models/props/textures/atlas.png", [1.0; 4]),
    ("crowbar", "models/props/crowbar.glb", "models/props/textures/atlas.png", [1.0; 4]),
    ("key", "models/props/key_skeleton.glb", "models/props/textures/atlas.png", [1.0; 4]),
    ("book", "models/misc/notebook_1.glb", "models/textures/notebook_1.png", [0.75, 0.82, 1.0, 1.0]),
    ("doll", "models/props/doll_porcelain.glb", "models/props/textures/atlas.png", [1.0; 4]),
    ("flashlight", "models/props/flashlight.glb", "models/props/textures/atlas.png", [1.0; 4]),
];

const SIZE: u32 = 256;

fn main() -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/last_service/assets");
    let assets = Assets::Dir(root.clone());
    let out = root.join("ui/icons/items");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut renderer = Renderer::new_headless(assets.clone())?;
    for (name, mesh, texture, tint) in ITEMS {
        let data = MeshData::load(&assets, mesh)?;
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for v in &data.vertices {
            lo = lo.min(Vec3::from(v.pos));
            hi = hi.max(Vec3::from(v.pos));
        }
        let extent = (hi - lo).max_element().max(1e-4);
        let scale = 1.0 / extent;
        let centre = (lo + hi) * 0.5 * scale;

        let mut scene = Scene::default();
        let st = &mut scene.settings;
        st.atmosphere.procedural = false;
        st.sky = [0.035, 0.034, 0.03];
        st.ambient = [0.28, 0.27, 0.25];
        st.fog = 0.0;
        st.exposure = 1.15;
        st.post.vignette = 0.45;
        let item = scene.spawn(name);
        scene.world.insert(item, Transform { position: -centre, rotation: Vec3::ZERO, scale: Vec3::splat(scale) });
        scene.world.insert(item, MeshRenderer { mesh: mesh.to_string(), texture: texture.to_string(), color: *tint, roughness: 0.6, ..Default::default() });
        let key = scene.spawn("Key light");
        scene.world.insert(key, Transform { rotation: Vec3::new(-50.0, 35.0, 0.0), ..Default::default() });
        scene.world.insert(key, Light { intensity: 2.6, color: [1.0, 0.92, 0.8, 1.0], ..Default::default() });
        let rim = scene.spawn("Rim light");
        scene.world.insert(rim, Transform { position: Vec3::new(-1.2, 0.9, -1.4), ..Default::default() });
        scene.world.insert(rim, Light { kind: LightKind::Point, intensity: 5.0, range: 4.0, color: [0.55, 0.7, 0.9, 1.0], ..Default::default() });
        hierarchy::update_globals(&mut scene.world);

        // Three-quarter view from above, the item filling most of the frame.
        let eye = Vec3::new(0.95, 0.85, 1.25) * 1.3;
        let mut camera = Transform::at(eye);
        camera.look_at(Vec3::ZERO, Vec3::Y);
        let view = View::new(camera.matrix(), 34.0, 1.0, 0.02, 20.0);
        let debug = DebugDraw::default();
        let frame = Frame { view, size: [SIZE, SIZE], time: 0.0, dt: 0.0, debug: &debug, shadows: true, debug_view: 0, cinematic: true };
        let image = renderer.capture(&scene, &frame)?;
        let errors = renderer.take_errors();
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }
        let path = out.join(format!("{name}.png"));
        image.save(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("{}", path.display());
    }
    Ok(())
}
