//! Builds the "Hollow Brook" village scene in `games/village` from the Medieval Village MegaKit:
//! `cargo run -p kvas_engine --example make_village [kit glTF dir] [tree dir] [games dir]`.
//!
//! A cobbled square with five houses (plaster and brick), fences, a well, wagon, crates, lanterns,
//! a stream of voxel water, grass, volumetric clouds, golden-hour light and a gentle post look.

use glam::Vec3;
use kvas_engine::prelude::*;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use kvas_engine::sky::SkySettings;
use std::collections::BTreeSet;
use std::path::Path;

/// Width of one wall module in metres.
const M: f32 = 2.0;
/// Height of one storey.
const H: f32 = 3.0;

struct Builder {
    scene: Scene,
    used: BTreeSet<String>,
    seed: std::cell::Cell<u32>,
    /// Source folder of the Stylized Trees pack (FBX models live in `Models`).
    stylized_dir: std::path::PathBuf,
}

#[derive(Clone, Copy, PartialEq)]
enum Style {
    Plaster,
    Brick,
}

impl Builder {
    fn rand(&self) -> f32 {
        let next = self.seed.get().wrapping_mul(1664525).wrapping_add(1013904223);
        self.seed.set(next);
        (next >> 8) as f32 / 16_777_216.0
    }

    fn model(&mut self, name: &str, pos: Vec3, yaw: f32, scale: f32, parent: Option<Entity>) -> Entity {
        self.used.insert(name.to_string());
        let e = hierarchy::spawn_named(&mut self.scene.world, name);
        let w = &mut self.scene.world;
        w.insert(e, Transform { position: pos, rotation: Vec3::new(0.0, yaw, 0.0), scale: Vec3::splat(scale) });
        w.insert(e, MeshRenderer { mesh: format!("models/{name}.gltf"), color: [1.0; 4], roughness: 1.0, ..Default::default() });
        if let Some(p) = parent {
            hierarchy::set_parent_local(w, e, Some(p));
        }
        e
    }

    /// A model from the Stylized Nature MegaKit (trees, rocks, flowers, grass...).
    fn nature(&mut self, name: &str, pos: Vec3, yaw: f32, scale: f32, shadow: bool) -> Entity {
        self.used.insert(format!("#nature:{name}"));
        let e = hierarchy::spawn_named(&mut self.scene.world, name);
        let w = &mut self.scene.world;
        w.insert(e, Transform { position: pos, rotation: Vec3::new(0.0, yaw, 0.0), scale: Vec3::splat(scale) });
        w.insert(e, MeshRenderer { mesh: format!("models/nature/{name}.gltf"), color: [1.0; 4], roughness: 1.0, cast_shadows: shadow, ..Default::default() });
        e
    }

    /// A tree from the Stylized Trees pack, scaled to `height` metres and placed so its trunk base
    /// stands at `target` (the FBX files keep the trees at odd offsets from their origin).
    fn stylized(&mut self, name: &str, target: Vec3, yaw: f32, height: f32, shadow: bool) -> Entity {
        self.used.insert(format!("$st:{name}"));
        let assets = kvas_engine::assets::Assets::Dir(self.stylized_dir.join("Models"));
        let mesh = kvas_engine::mesh::MeshData::load(&assets, &format!("{name}.fbx")).expect("stylized tree");
        let (lo, hi) = mesh.bounds();
        let scale = height / (hi.y - lo.y).max(0.1);
        let origin = Vec3::new((lo.x + hi.x) * 0.5, lo.y, (lo.z + hi.z) * 0.5);
        let offset = glam::Mat3::from_rotation_y(yaw.to_radians()) * (origin * scale);
        let e = hierarchy::spawn_named(&mut self.scene.world, name);
        let w = &mut self.scene.world;
        w.insert(e, Transform { position: target - offset, rotation: Vec3::new(0.0, yaw, 0.0), scale: Vec3::splat(scale) });
        w.insert(e, MeshRenderer { mesh: format!("models/stylized/{name}.fbx"), color: [1.0; 4], roughness: 1.0, cast_shadows: shadow, ..Default::default() });
        e
    }

    fn group(&mut self, name: &str, pos: Vec3, yaw: f32) -> Entity {
        let e = hierarchy::spawn_named(&mut self.scene.world, name);
        self.scene.world.insert(e, Transform { position: pos, rotation: Vec3::new(0.0, yaw, 0.0), scale: Vec3::ONE });
        e
    }

    fn prim(&mut self, name: &str, kind: ShapeType, pos: Vec3, size: Vec3, color: [f32; 4]) -> Entity {
        let e = hierarchy::spawn_named(&mut self.scene.world, name);
        let w = &mut self.scene.world;
        w.insert(e, Transform { position: pos, scale: size, ..Default::default() });
        w.insert(e, Shape::of(kind));
        w.insert(e, MeshRenderer { color, roughness: 0.9, ..Default::default() });
        e
    }

    fn lantern(&mut self, pos: Vec3, parent: Option<Entity>) {
        let e = hierarchy::spawn_named(&mut self.scene.world, "Lantern");
        let w = &mut self.scene.world;
        w.insert(e, Transform { position: pos, scale: Vec3::splat(0.22), ..Default::default() });
        w.insert(e, Shape::of(ShapeType::Sphere));
        w.insert(e, MeshRenderer { color: [1.0, 0.75, 0.4, 1.0], emissive: 3.0, cast_shadows: false, ..Default::default() });
        w.insert(e, Light { kind: LightKind::Point, color: [1.0, 0.7, 0.4, 1.0], intensity: 2.4, range: 9.0, ..Default::default() });
        w.insert(e, Scripts(vec![ScriptSlot { name: "Flicker".into(), props: Default::default() }]));
        if let Some(p) = parent {
            hierarchy::set_parent_local(w, e, Some(p));
        }
    }

    /// A house centred on `(x, z)`: `nx` x `nz` wall modules, `floors` storeys, door on the +Z side.
    fn house(&mut self, name: &str, x: f32, z: f32, yaw: f32, nx: i32, nz: i32, floors: i32, style: Style) {
        let root = self.group(name, Vec3::new(x, 0.0, z), yaw);
        let (w, d) = (nx as f32 * M, nz as f32 * M);
        let (plain, door, window) = match style {
            Style::Plaster => ("Wall_Plaster_Straight", "Wall_Plaster_Door_Round", "Wall_Plaster_Window_Wide_Round"),
            Style::Brick => ("Wall_UnevenBrick_Straight", "Wall_UnevenBrick_Door_Round", "Wall_UnevenBrick_Window_Wide_Round"),
        };
        for f in 0..floors {
            let y = f as f32 * H;
            // Front (+Z) and back (-Z), modules run along X.
            for i in 0..nx {
                let px = -w * 0.5 + M * (i as f32 + 0.5);
                let door_here = f == 0 && i == nx / 2;
                let front = if door_here { door } else if (i + f) % 2 == 0 { window } else { plain };
                self.model(front, Vec3::new(px, y, d * 0.5), 0.0, 1.0, Some(root));
                self.model(if (i + f) % 2 == 1 { window } else { plain }, Vec3::new(px, y, -d * 0.5), 180.0, 1.0, Some(root));
            }
            // Sides, modules run along Z.
            for j in 0..nz {
                let pz = -d * 0.5 + M * (j as f32 + 0.5);
                let a = if (j + f) % 2 == 0 { window } else { plain };
                self.model(a, Vec3::new(w * 0.5, y, pz), 90.0, 1.0, Some(root));
                self.model(a, Vec3::new(-w * 0.5, y, pz), -90.0, 1.0, Some(root));
            }
            let corner = if style == Style::Brick { "Corner_Exterior_Brick" } else { "Corner_Exterior_Wood" };
            for (cx, cz, cy) in [(w * 0.5, d * 0.5, 0.0), (-w * 0.5, d * 0.5, 90.0), (-w * 0.5, -d * 0.5, 180.0), (w * 0.5, -d * 0.5, 270.0)] {
                self.model(corner, Vec3::new(cx, y, cz), cy, 1.0, Some(root));
            }
        }
        // Wooden floor slab so the inside is not a void.
        let top = floors as f32 * H;
        // Roof: ridge along the longer side.
        let (rw, rd, ryaw) = if nx >= nz { (nz * 2, nx * 2, 90.0) } else { (nx * 2, nz * 2, 0.0) };
        let roof = format!("Roof_RoundTiles_{}x{}", rw.clamp(4, 8), pick_len(rw.clamp(4, 8), rd));
        self.model(&roof, Vec3::new(0.0, top, 0.0), ryaw, 1.0, Some(root));
        // Chimney, steps and a lantern by the door.
        self.model("Prop_Chimney", Vec3::new(w * 0.25, top - 0.3, -d * 0.2), 0.0, 1.0, Some(root));
        self.model("Stairs_Exterior_Straight", Vec3::new(0.0, 0.0, d * 0.5 + 1.3), 0.0, 0.6, Some(root));
        self.lantern(Vec3::new(1.4, 2.3, d * 0.5 + 0.25), Some(root));
        for (k, sx) in [-1.0f32, 1.0].iter().enumerate() {
            let name = if k == 0 { "Bush_Common_Flowers" } else { "barrels" };
            self.used.insert(if k == 0 { format!("#nature:{name}") } else { format!("@tree:{name}") });
            let e = hierarchy::spawn_named(&mut self.scene.world, name);
            let scale = if k == 0 { 2.6 } else { 1.6 };
            self.scene.world.insert(e, Transform { position: Vec3::new(sx * (w * 0.5 - 0.6), 0.0, d * 0.5 + 0.9), rotation: Vec3::new(0.0, 40.0 * k as f32, 0.0), scale: Vec3::splat(scale) });
            self.scene.world.insert(e, MeshRenderer { mesh: if k == 0 { format!("models/nature/{name}.gltf") } else { format!("models/trees/{name}.glb") }, color: [1.0; 4], roughness: 0.95, cast_shadows: k == 1, ..Default::default() });
            hierarchy::set_parent_local(&mut self.scene.world, e, Some(root));
        }
        // A solid box so the player cannot walk through the walls.
        let body = self.prim(&format!("{name} Collider"), ShapeType::Cube, Vec3::new(0.0, H * floors as f32 * 0.5, 0.0), Vec3::new(w, H * floors as f32, d), [1.0; 4]);
        self.scene.world.remove::<MeshRenderer>(body);
        self.scene.world.insert(body, Collider::default());
        hierarchy::set_parent_local(&mut self.scene.world, body, Some(root));
    }
}

/// Roof models exist in a limited set of sizes; choose the nearest available length for a width.
const WARM_GRADE: &str = "// Warm evening grade with a soft vignette. params.x = warmth, params.y = vignette strength.
#include \"builtin:lib/color\"

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    var c = lib_saturation(color, 1.0 + 0.12 * params.x);
    c *= vec3<f32>(1.0 + 0.05 * params.x, 1.0, 1.0 - 0.07 * params.x);
    c *= lib_vignette(uv, params.y);
    return c;
}
";

fn pick_len(width: i32, length: i32) -> i32 {
    let options: &[i32] = match width {
        4 => &[4, 6, 8],
        6 => &[4, 6, 8, 10, 12, 14],
        _ => &[8, 10, 12, 14],
    };
    *options.iter().min_by_key(|l| (**l - length).abs()).unwrap()
}

fn script(name: &str, props: serde_json::Value) -> Scripts {
    Scripts(vec![ScriptSlot { name: name.into(), props: props.as_object().cloned().unwrap_or_default() }])
}

fn main() {
    let mut args = std::env::args().skip(1);
    let kit = args.next().unwrap_or_else(|| ".test_assets/Medieval Village MegaKit[Standard]/glTF".into());
    let trees = args.next().unwrap_or_else(|| ".test_assets/Models/GLB format".into());
    let games = args.next().unwrap_or_else(|| "games".into());
    let stylized = args.next().unwrap_or_else(|| ".test_assets/Stylized Trees".into());
    let nature_dir = args.next().unwrap_or_else(|| ".test_assets/Stylized Nature MegaKit[Standard]/glTF".into());
    let root_dir = Path::new(&games).join("village");

    let mut scene = Scene::default();
    scene.settings.atmosphere = SkySettings { procedural: true, time: 16.5, speed: 0.3, clouds: 0.5, haze: 0.45, stars: 0.9, ..Default::default() };
    scene.settings.exposure = 1.1;
    scene.settings.render_scale = 0.77;
    scene.settings.view_distance = 160.0;
    scene.settings.fog = 0.006;
    scene.settings.filmic = 0.85;
    scene.settings.saturation = 1.12;
    scene.settings.contrast = 1.05;
    scene.settings.post.bloom = 0.22;
    scene.settings.post.bloom_threshold = 0.9;
    scene.settings.post.bloom_radius = 1.0;
    scene.settings.post.vignette = 0.3;
    scene.settings.post.lens_flare = 0.0;
    scene.settings.post.film_grain = 0.05;
    scene.settings.shadows.softness = 1.8;
    scene.settings.post.effects.push(PostEffect { shader: "shaders/fx/warm_grade.wgsl".into(), enabled: true, params: [1.0, 0.25, 0.0, 0.0] });
    scene.settings.shadows.range = 55.0;
    let mut b = Builder { scene, used: BTreeSet::new(), seed: std::cell::Cell::new(7), stylized_dir: std::path::PathBuf::from(&stylized) };

    let sun = hierarchy::spawn_named(&mut b.scene.world, "Sun");
    b.scene.world.insert(sun, Transform { position: Vec3::new(0.0, 30.0, 0.0), rotation: Vec3::new(-30.0, -50.0, 0.0), ..Default::default() });
    b.scene.world.insert(sun, Light { intensity: 1.0, ..Default::default() });
    // First-person player: a capsule body on the road with the camera as its child.
    let player = hierarchy::spawn_named(&mut b.scene.world, "Player");
    b.scene.world.insert(player, Transform::at(Vec3::new(0.0, 1.2, 42.0)));
    b.scene.world.insert(player, Collider { shape: ShapeKind::Capsule, size: Vec3::new(0.7, 1.8, 0.7), friction: 0.0, ..Default::default() });
    b.scene.world.insert(player, RigidBody { mass: 70.0, lock_rotation: true, linear_damping: 0.0, angular_damping: 1.0, ccd: true, ..Default::default() });
    b.scene.world.insert(player, script("FpsPlayer", serde_json::json!({})));
    let cam = hierarchy::spawn_named(&mut b.scene.world, "Camera");
    b.scene.world.insert(cam, Transform::at(Vec3::new(0.0, 0.7, 0.0)));
    b.scene.world.insert(cam, Camera { far: 700.0, focal_length: 28.0, dof: true, aperture: 4.0, autofocus: true, letterbox: 0.0, ..Default::default() });
    hierarchy::set_parent_local(&mut b.scene.world, cam, Some(player));

    // Ground: earth slabs around a pond basin, a cobbled road with a square, meadows.
    let pond = (-30.0f32, 2.0f32, 26.0f32, 20.0f32); // centre x, z, size x, size z
    let (px0, px1, pz0, pz1) = (pond.0 - pond.2 * 0.5, pond.0 + pond.2 * 0.5, pond.1 - pond.3 * 0.5, pond.1 + pond.3 * 0.5);
    for (name, x0, x1, z0, z1) in [("Ground West", -110.0, px0, -110.0, 110.0), ("Ground East", px1, 110.0, -110.0, 110.0), ("Ground North", px0, px1, -110.0, pz0), ("Ground South", px0, px1, pz1, 110.0)] {
        let g = b.prim(name, ShapeType::Cube, Vec3::new((x0 + x1) * 0.5, -0.5, (z0 + z1) * 0.5), Vec3::new(x1 - x0, 1.0, z1 - z0), [0.26, 0.36, 0.16, 1.0]);
        b.scene.world.insert(g, Collider::default());
    }
    let bed = b.prim("Pond Bed", ShapeType::Cube, Vec3::new(pond.0, -1.7, pond.1), Vec3::new(pond.2, 1.0, pond.3), [0.2, 0.22, 0.18, 1.0]);
    b.scene.world.insert(bed, Collider::default());
    // A painted surface along the road: grass that fades into dirt, with sand and stone patches,
    // blended by a soft mask (no hard seams). The mask is brushed here the same way the editor does.
    let verge = hierarchy::spawn_named(&mut b.scene.world, "Road Verge");
    b.scene.world.insert(verge, Transform::at(Vec3::new(0.0, 0.012, 0.0)));
    b.scene.world.insert(verge, Shape { kind: ShapeType::Plane, size: Vec3::new(18.0, 1.0, 96.0), rings: 4, ..Shape::of(ShapeType::Plane) });
    b.scene.world.insert(verge, MeshRenderer::default());
    b.scene.world.insert(verge, TexturePaint {
        layer1: "textures/ground_grass.png".into(),
        layer2: "textures/ground_dirt.png".into(),
        layer3: "textures/ground_sand.png".into(),
        layer4: "textures/ground_stones.png".into(),
        mask: "road_verge".into(),
        tiling: 0.22,
        breakup: 0.5,
        resolution: 1024,
    });
    {
        use kvas_engine::paint::{BrushMode, PaintMaps};
        let maps = PaintMaps::global();
        maps.create("road_verge", 1024, 1024);
        // Dirt hugging both road edges, widening and thinning along the way.
        for i in 0..500 {
            let v = i as f32 / 500.0;
            for side in [-1.0f32, 1.0] {
                let wander = (v * 37.0).sin() * 0.018 + (v * 91.0).sin() * 0.008;
                let u = 0.5 + side * (0.11 + wander + b.rand() * 0.01);
                let radius = 0.018 + 0.014 * (v * 23.0).sin().abs() + b.rand() * 0.008;
                maps.brush("road_verge", [u, v], radius, 0.3, 0.0, BrushMode::Layer(1));
            }
        }
        // Wider dirt stains and bare patches near the lamp posts and doorsteps.
        for _ in 0..60 {
            let side = if b.rand() < 0.5 { -1.0 } else { 1.0 };
            let (u, v) = (0.5 + side * (0.12 + b.rand() * 0.12), b.rand());
            maps.brush("road_verge", [u, v], 0.015 + b.rand() * 0.02, 0.5, 0.0, BrushMode::Layer(1));
        }
        // Sandy puddle margins and a few stone patches further out.
        for _ in 0..70 {
            let side = if b.rand() < 0.5 { -1.0 } else { 1.0 };
            maps.brush("road_verge", [0.5 + side * (0.2 + b.rand() * 0.2), b.rand()], 0.02 + b.rand() * 0.03, 0.7, 0.1, BrushMode::Layer(2));
        }
        for _ in 0..45 {
            let side = if b.rand() < 0.5 { -1.0 } else { 1.0 };
            maps.brush("road_verge", [0.5 + side * (0.15 + b.rand() * 0.25), b.rand()], 0.012 + b.rand() * 0.02, 0.8, 0.3, BrushMode::Layer(3));
        }
        // Soften everything once so no stroke edge shows.
        for _ in 0..500 {
            maps.brush("road_verge", [0.5 + (b.rand() - 0.5) * 0.7, b.rand()], 0.04, 0.9, 0.2, BrushMode::Smooth);
        }
    }
    // Road along Z (two tiles wide) and a path to the pond; a wider square in the middle.
    for i in -22i32..=22 {
        let z = i as f32 * 2.0 + 1.0;
        for dx in [-1.0f32, 1.0] {
            b.model(if (i + dx as i32) % 2 == 0 { "Floor_UnevenBrick" } else { "Floor_Brick" }, Vec3::new(dx, 0.02, z), 0.0, 1.0, None);
        }
    }
    for i in 0..7i32 {
        let x = -4.0 - i as f32 * 2.0 - 1.0;
        for dz in [-1.0f32, 1.0] {
            b.model("Floor_UnevenBrick", Vec3::new(x, 0.02, dz + 2.0), 0.0, 1.0, None);
        }
    }
    for gx in [-3i32, -2, 2, 3] {
        for gz in -2i32..=2 {
            b.model(if (gx + gz) % 2 == 0 { "Floor_Brick" } else { "Floor_UnevenBrick" }, Vec3::new(gx as f32 * 2.0 + gx.signum() as f32 * 0.0, 0.025, gz as f32 * 2.0 + 4.0), 0.0, 1.0, None);
        }
    }
    for (x, z, wdt, dp) in [(40.0, 0.0, 34.0, 90.0), (-68.0, 0.0, 30.0, 90.0), (0.0, -58.0, 90.0, 24.0), (0.0, 58.0, 90.0, 24.0)] {
        let g = hierarchy::spawn_named(&mut b.scene.world, "Meadow");
        b.scene.world.insert(g, Transform::at(Vec3::new(x, 0.0, z)));
        b.scene.world.insert(g, Grass { count: 60_000, width: wdt, depth: dp, blade_height: 0.5, ..Default::default() });
    }

    // Houses along both sides of the road: varied size, storeys and style.
    let sizes = [(3, 3, 2), (2, 3, 1), (3, 4, 2), (3, 3, 1), (2, 3, 3), (3, 3, 2), (3, 4, 1), (2, 3, 2), (3, 3, 3)];
    for (k, z) in [-38.0f32, -29.0, -20.0, -11.0, -2.0, 7.0, 16.0, 25.0, 34.0].iter().enumerate() {
        for (side, sign) in [(0usize, -1.0f32), (1, 1.0)] {
            let (nx, nz, floors) = sizes[(k + side * 4) % sizes.len()];
            let style = if (k + side) % 3 == 1 { Style::Brick } else { Style::Plaster };
            let center_x = sign * (4.0 + nz as f32);
            let yaw = if sign < 0.0 { 90.0 } else { -90.0 };
            b.house(&format!("House {}{}", k + 1, if side == 0 { "L" } else { "R" }), center_x, *z, yaw, nx, nz, floors, style);
        }
    }

    // Square furniture: well, wagon, crates, lamp posts along the road.
    b.prim("Well Wall", ShapeType::Cylinder, Vec3::new(0.0, 0.5, 4.0), Vec3::new(1.8, 1.0, 1.8), [0.55, 0.52, 0.48, 1.0]);
    b.prim("Well Water", ShapeType::Cylinder, Vec3::new(0.0, 0.85, 4.0), Vec3::new(1.4, 0.05, 1.4), [0.1, 0.3, 0.45, 0.8]);
    for dx in [-0.8f32, 0.8] {
        b.prim("Well Post", ShapeType::Cube, Vec3::new(dx, 1.7, 4.0), Vec3::new(0.14, 1.8, 0.14), [0.35, 0.25, 0.15, 1.0]);
    }
    b.prim("Well Roof", ShapeType::Cone, Vec3::new(0.0, 3.0, 4.0), Vec3::new(2.4, 0.9, 2.4), [0.6, 0.25, 0.15, 1.0]);
    let wagon = b.model("Prop_Wagon", Vec3::new(-2.8, 0.0, 14.0), 80.0, 1.0, None);
    b.scene.world.insert(wagon, Collider { size: Vec3::new(2.0, 1.5, 4.0), ..Default::default() });
    for (i, (x, z)) in [(3.4f32, 9.5f32), (4.2, 10.4), (-3.2, -6.5), (-3.9, -5.7), (3.8, -14.0)].iter().enumerate() {
        let c = b.model("Prop_Crate", Vec3::new(*x, 0.0, *z), 12.0 * i as f32, 1.0, None);
        b.scene.world.insert(c, Collider { size: Vec3::splat(1.0), ..Default::default() });
    }
    for i in 0..13i32 {
        let z = -36.0 + i as f32 * 6.0;
        for x in [-2.6f32, 2.6] {
            b.prim("Lamp Post", ShapeType::Cylinder, Vec3::new(x, 1.5, z), Vec3::new(0.1, 3.0, 0.1), [0.12, 0.12, 0.14, 1.0]);
            b.lantern(Vec3::new(x, 3.1, z), None);
        }
    }

    // The pond: a rim of rocks, a plank pier and the water itself.
    for i in 0..26 {
        let a = i as f32 / 26.0 * std::f32::consts::TAU;
        let (rx, rz) = (pond.2 * 0.5 + 0.2, pond.3 * 0.5 + 0.2);
        let r = b.rand();
        let (x, z) = ((pond.0 + a.cos() * rx).clamp(px0 - 0.3, px1 + 0.3), (pond.1 + a.sin() * rz).clamp(pz0 - 0.3, pz1 + 0.3));
        let name = ["Rock_Medium_1", "Rock_Medium_2", "Rock_Medium_3"][i % 3];
        b.nature(name, Vec3::new(x, 0.0, z), r * 360.0, 0.8 + r * 0.9, true);
        // Pebbles and plants crowd around each rock.
        for k in 0..3 {
            let (ox, oz) = (b.rand() * 2.4 - 1.2, b.rand() * 2.4 - 1.2);
            let peb = ["Pebble_Round_1", "Pebble_Round_3", "Pebble_Square_2", "Pebble_Round_5"][(i + k) % 4];
            b.nature(peb, Vec3::new(x + ox, 0.0, z + oz), b.rand() * 360.0, 1.5 + b.rand(), false);
        }
        let plant = ["Plant_1", "Fern_1", "Plant_7", "Clover_2", "Flower_3_Group", "Flower_4_Group"][i % 6];
        let (ox, oz) = (b.rand() * 3.0 - 1.5, b.rand() * 3.0 - 1.5);
        b.nature(plant, Vec3::new(x + ox, 0.0, z + oz), b.rand() * 360.0, 0.7 + b.rand() * 0.5, false);
    }
    for i in 0..6 {
        b.prim("Pier Plank", ShapeType::Cube, Vec3::new(px1 - 0.5 - i as f32 * 1.0, 0.0, pond.1), Vec3::new(1.0, 0.1, 2.2), [0.42, 0.3, 0.18, 1.0]);
    }
    let water = hierarchy::spawn_named(&mut b.scene.world, "Pond Water");
    b.scene.world.insert(water, Transform::at(Vec3::new(pond.0, -0.28, pond.1)));
    b.scene.world.insert(water, VoxelWater { style: WaterStyle::Realistic, cells: 52, cell: 0.5, wave_height: 0.12, wavelength: 4.5, direction: 20.0, ..Default::default() });
    b.scene.world.insert(water, MeshRenderer { color: [0.04, 0.22, 0.28, 0.6], roughness: 0.04, reflectivity: 0.9, cast_shadows: false, ..Default::default() });
    b.scene.world.insert(water, AudioSource { clip: "sounds/brook.wav".into(), volume: 0.5, min_distance: 4.0, max_distance: 30.0, ..Default::default() });

    // A node material (mossy stone) on a boulder by the pond: the albedo texture blended with moss
    // where a noise mask says so, higher up and on top-facing surfaces.
    {
        use kvas_engine::material_graph::MaterialGraph;
        let mut gr = MaterialGraph::default_graph();
        gr.textures = ["textures/rock_color.png".into(), "textures/rock_normal.png".into(), String::new()];
        let tex = gr.add_node("Texture", [40.0, 40.0]);
        gr.nodes.iter_mut().find(|n| n.id == tex).unwrap().params = vec![0.0, 0.5];
        let moss = gr.add_node("Color", [40.0, 170.0]);
        gr.nodes.iter_mut().find(|n| n.id == moss).unwrap().params = vec![0.18, 0.38, 0.08];
        let noise = gr.add_node("Noise", [40.0, 250.0]);
        gr.nodes.iter_mut().find(|n| n.id == noise).unwrap().params = vec![1.6];
        let step = gr.add_node("Smoothstep", [230.0, 250.0]);
        gr.nodes.iter_mut().find(|n| n.id == step).unwrap().params = vec![0.45, 0.65];
        let mix = gr.add_node("Mix", [420.0, 90.0]);
        let rough = gr.add_node("Value", [420.0, 260.0]);
        gr.nodes.iter_mut().find(|n| n.id == rough).unwrap().params = vec![0.85];
        gr.connect(noise, step, 0);
        gr.connect(tex, mix, 0);
        gr.connect(moss, mix, 1);
        gr.connect(step, mix, 2);
        gr.connect(mix, 1, 0);
        gr.connect(rough, 1, 1);
        gr.nodes.iter_mut().find(|n| n.kind == "Output").unwrap().pos = [620.0, 120.0];
        std::fs::create_dir_all(root_dir.join("assets/materials")).unwrap();
        std::fs::write(root_dir.join("assets/materials/mossy_stone.kmat"), gr.to_json()).unwrap();
        let rock = hierarchy::spawn_named(&mut b.scene.world, "Mossy Boulder");
        b.scene.world.insert(rock, Transform { position: Vec3::new(pond.0 + 15.5, 0.5, pond.1 + 8.0), rotation: Vec3::new(0.0, 35.0, 0.0), scale: Vec3::new(2.4, 1.7, 2.2) });
        b.scene.world.insert(rock, Shape { kind: ShapeType::Rock, size: Vec3::ONE, radius2: 1.0, segments: 16, rings: 12, seed: 4, ..Shape::of(ShapeType::Rock) });
        b.scene.world.insert(rock, MeshRenderer { shader: "materials/mossy_stone.kmat".into(), color: [1.0; 4], ..Default::default() });
        b.scene.world.insert(rock, Collider { size: Vec3::new(1.0, 1.0, 1.0), ..Default::default() });
    }

    // Trees: broadleaf and pines near the village, a dense pine wood further out, a few twisted ones.
    for i in 0..44usize {
        let side = if i % 2 == 0 { 1.0f32 } else { -1.0 };
        let z = -48.0 + (i as f32 / 44.0) * 100.0 + b.rand() * 3.0;
        let x = side * (24.0 + b.rand() * 20.0) + if side < 0.0 { -24.0 } else { 0.0 };
        if x > px0 - 5.0 && x < px1 + 5.0 && z > pz0 - 5.0 && z < pz1 + 5.0 {
            continue;
        }
        let name = match i % 7 {
            0 | 1 => format!("CommonTree_{}", 1 + i % 5),
            2 | 3 => format!("Pine_{}", 1 + i % 5),
            4 => format!("TwistedTree_{}", 1 + i % 5),
            5 => format!("CommonTree_{}", 1 + (i + 2) % 5),
            _ => format!("Pine_{}", 1 + (i + 3) % 5),
        };
        let sc = 0.9 + b.rand() * 0.7;
        b.nature(&name, Vec3::new(x, 0.0, z), b.rand() * 360.0, sc, i % 3 == 0);
    }
    for i in 0..40usize {
        let side = if i % 2 == 0 { 1.0f32 } else { -1.0 };
        let x = side * (48.0 + b.rand() * 22.0) + if side < 0.0 { -22.0 } else { 0.0 };
        let z = -70.0 + b.rand() * 140.0;
        b.nature(&format!("Pine_{}", 1 + i % 5), Vec3::new(x, 0.0, z), b.rand() * 360.0, 1.0 + b.rand() * 0.9, false);
    }
    // Meadow flora: flowers, clover, tall grass and fern clumps, mushrooms under the trees.
    let flora = ["Flower_3_Group", "Flower_4_Group", "Clover_1", "Clover_2", "Grass_Common_Tall", "Grass_Wispy_Tall", "Fern_1", "Plant_1_Big", "Bush_Common", "Mushroom_Common", "Flower_3_Single", "Flower_4_Single"];
    for i in 0..120usize {
        let east = i % 3 != 0;
        let x = if east { 12.0 + b.rand() * 36.0 } else { -62.0 + b.rand() * 22.0 };
        let z = -50.0 + b.rand() * 100.0;
        let name = flora[i % flora.len()];
        let scale = if name.starts_with("Grass") || name.starts_with("Clover") { 1.6 + b.rand() } else { 0.9 + b.rand() * 0.8 };
        b.nature(name, Vec3::new(x, 0.0, z), b.rand() * 360.0, scale, false);
    }
    // Verges along the road between the lamps: flowers and low plants, each of them a small garden.
    for i in 0..22i32 {
        let z = -40.0 + i as f32 * 3.7;
        for side in [-1.0f32, 1.0] {
            let name = ["Flower_3_Group", "Clover_1", "Flower_4_Group", "Grass_Common_Short", "Plant_1", "Bush_Common_Flowers"][(i as usize + (side > 0.0) as usize * 3) % 6];
            let x = side * (3.0 + b.rand() * 0.6);
            b.nature(name, Vec3::new(x, 0.02, z + b.rand()), b.rand() * 360.0, 0.45 + b.rand() * 0.3, false);
        }
    }
    // Stylized Trees pack: willows over the pond, oaks behind the houses, columnar trees closing
    // the road, swiggly ones in the meadows, pines on the hills.
    for (i, (dx, dz)) in [(-9.0f32, -8.5f32), (8.5, -7.0), (-2.0, 10.8)].iter().enumerate() {
        let name = format!("STWillow{}", 1 + i);
        b.stylized(&name, Vec3::new(pond.0 + dx, 0.0, pond.1 + dz), b.rand() * 360.0, 8.5 + b.rand() * 2.5, true);
    }
    for i in 0..10usize {
        let side = if i % 2 == 0 { 1.0f32 } else { -1.0 };
        let z = -40.0 + (i / 2) as f32 * 18.0 + b.rand() * 3.0;
        b.stylized(&format!("STOak{}", 1 + i % 5), Vec3::new(side * (17.0 + b.rand() * 6.0), 0.0, z), b.rand() * 360.0, 10.0 + b.rand() * 4.0, true);
    }
    for i in 0..6usize {
        let x = if i % 2 == 0 { -5.5 } else { 5.5 };
        b.stylized(&format!("STColumnar{}", 1 + i % 5), Vec3::new(x, 0.0, -46.0 - (i / 2) as f32 * 5.0), 0.0, 9.0 + b.rand() * 2.0, false);
    }
    for i in 0..8usize {
        let x = 20.0 + b.rand() * 28.0;
        b.stylized(&format!("STSwiggly{}", 1 + i % 4), Vec3::new(if i % 2 == 0 { x } else { -x - 30.0 }, 0.0, -35.0 + b.rand() * 70.0), b.rand() * 360.0, 4.5 + b.rand() * 2.0, false);
    }
    for i in 0..14usize {
        let side = if i % 2 == 0 { 1.0f32 } else { -1.0 };
        b.stylized(&format!("STPine{}", 1 + i % 7), Vec3::new(side * (60.0 + b.rand() * 25.0), 0.0, -70.0 + b.rand() * 140.0), b.rand() * 360.0, 11.0 + b.rand() * 4.0, false);
    }
    // A few dead trunks and rocks to break up the skyline.
    for (i, (x, z)) in [(-44.0f32, -30.0f32), (46.0, 30.0), (-38.0, 38.0), (40.0, -34.0)].iter().enumerate() {
        b.nature(&format!("DeadTree_{}", 1 + i), Vec3::new(*x, 0.0, *z), i as f32 * 71.0, 1.1, true);
        b.nature(["Rock_Medium_1", "Rock_Medium_2"][i % 2], Vec3::new(*x + 3.0, 0.0, *z + 2.0), i as f32 * 40.0, 2.2, true);
    }
    let clouds = hierarchy::spawn_named(&mut b.scene.world, "Clouds");
    b.scene.world.insert(clouds, Transform::at(Vec3::new(0.0, 80.0, 0.0)));
    b.scene.world.insert(clouds, VoxelClouds::default());
    let music = hierarchy::spawn_named(&mut b.scene.world, "Village Music");
    b.scene.world.insert(music, AudioSource { clip: "music/village.wav".into(), volume: 0.3, spatial: false, ..Default::default() });

    // Copy the models the scene uses, with their buffers and the kit textures.
    let models_out = root_dir.join("assets/models");
    std::fs::create_dir_all(models_out.join("trees")).unwrap();
    let kit = Path::new(&kit);
    for name in b.used.iter().filter(|n| !n.starts_with('@') && !n.starts_with('#') && !n.starts_with('$')) {
        for ext in ["gltf", "bin"] {
            let from = kit.join(format!("{name}.{ext}"));
            std::fs::copy(&from, models_out.join(format!("{name}.{ext}"))).unwrap_or_else(|e| panic!("{}: {e}", from.display()));
        }
    }
    for entry in std::fs::read_dir(kit).unwrap().flatten() {
        if entry.path().extension().is_some_and(|e| e == "png") {
            std::fs::copy(entry.path(), models_out.join(entry.file_name())).unwrap();
        }
    }
    let nature_dir = Path::new(&nature_dir);
    std::fs::create_dir_all(models_out.join("nature")).unwrap();
    for name in b.used.iter().filter_map(|n| n.strip_prefix("#nature:")) {
        for ext in ["gltf", "bin"] {
            let from = nature_dir.join(format!("{name}.{ext}"));
            std::fs::copy(&from, models_out.join(format!("nature/{name}.{ext}"))).unwrap_or_else(|e| panic!("{}: {e}", from.display()));
        }
    }
    for entry in std::fs::read_dir(nature_dir).unwrap().flatten() {
        if entry.path().extension().is_some_and(|e| e == "png") {
            std::fs::copy(entry.path(), models_out.join("nature").join(entry.file_name())).unwrap();
        }
    }
    let stylized_dir = Path::new(&stylized);
    std::fs::create_dir_all(models_out.join("stylized/Textures")).unwrap();
    for name in b.used.iter().filter_map(|n| n.strip_prefix("$st:")) {
        std::fs::copy(stylized_dir.join(format!("Models/{name}.fbx")), models_out.join(format!("stylized/{name}.fbx"))).unwrap();
    }
    for entry in std::fs::read_dir(stylized_dir.join("Textures")).unwrap().flatten() {
        std::fs::copy(entry.path(), models_out.join("stylized/Textures").join(entry.file_name())).unwrap();
    }
    let trees = Path::new(&trees);
    for name in b.used.iter().filter_map(|n| n.strip_prefix("@tree:")) {
        std::fs::copy(trees.join(format!("{name}.glb")), models_out.join(format!("trees/{name}.glb"))).unwrap();
    }
    std::fs::create_dir_all(models_out.join("trees/Textures")).unwrap();
    for entry in std::fs::read_dir(trees.join("Textures")).unwrap().flatten() {
        std::fs::copy(entry.path(), models_out.join("trees/Textures").join(entry.file_name())).unwrap();
    }

    kvas_engine::paint::PaintMaps::global().save_all(&root_dir.join("assets"));
    // A sample camera shader, listed in the scene so it shows up in "Camera shaders".
    std::fs::create_dir_all(root_dir.join("assets/shaders/fx")).unwrap();
    std::fs::write(root_dir.join("assets/shaders/fx/warm_grade.wgsl"), WARM_GRADE).unwrap();
    std::fs::create_dir_all(root_dir.join("assets/scenes")).unwrap();
    std::fs::write(root_dir.join("assets/scenes/main.scene.json"), b.scene.to_json(&Registry::default())).unwrap();
    Project { root: root_dir.clone(), config: ProjectConfig { name: "Hollow Brook".into(), start_scene: "scenes/main.scene.json".into(), scripts_package: "village_scripts".into(), ..Default::default() } }.save().unwrap();
    println!("wrote {} ({} models)", root_dir.display(), b.used.len());
}
