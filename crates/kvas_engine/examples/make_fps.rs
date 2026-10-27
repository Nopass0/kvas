//! Builds `games/test_fps`, a first-person puzzle yard, from the packs in `.test_assets`:
//! `cargo run -p kvas_engine --example make_fps [assets dir] [games dir]`
//!
//! Walk, jump, carry barrels and rocks, press buttons, weigh down a pressure plate and light the
//! beacon. Uses Kenney GLB walls, FBX props with Unity PBR maps, PSX kitchen props, hand-painted
//! PBR texture sets for the ground, paving, gates and ~60 procedural, triplanar-mapped rocks,
//! grass, shadows and the procedural sky with its day-night cycle.
//! Only the files the scene uses are copied (and shrunk) into the project.

use glam::Vec3;
use image::imageops::FilterType;
use kvas_engine::assets::Assets;
use kvas_engine::materials::{discover, PbrSet};
use kvas_engine::prelude::*;
use kvas_engine::procedural::Rng;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use kvas_engine::sky::SkySettings;
use std::path::{Path, PathBuf};

/// One Kenney kit tile in metres.
const K: f32 = 3.0;

struct Gen {
    scene: Scene,
    src: PathBuf,
    assets_dir: PathBuf,
    assets: Assets,
}

fn file_list(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir).map(|d| d.flatten().map(|e| e.path()).collect()).unwrap_or_default()
}

fn lower_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// Shrinks an image so its longest edge is at most `edge`.
fn load_resized(path: &Path, edge: u32) -> image::DynamicImage {
    let img = image::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    if img.width().max(img.height()) > edge {
        img.resize(edge, edge, FilterType::Triangle)
    } else {
        img
    }
}

fn save_jpg(img: &image::DynamicImage, path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90).encode_image(&image::DynamicImage::ImageRgb8(img.to_rgb8())).unwrap();
}

impl Gen {
    fn new(src: &Path, out: &Path) -> Self {
        let assets_dir = out.join("assets");
        std::fs::create_dir_all(&assets_dir).unwrap();
        Self { scene: Scene::default(), src: src.to_path_buf(), assets: Assets::Dir(assets_dir.clone()), assets_dir }
    }

    /// Copies a PBR texture set (any of the pack naming styles) as `<dst>/<stem>_BaseColor.jpg` etc.
    fn import_pbr(&self, src_dir: &str, dst_stem: &str, edge: u32) -> PbrSet {
        let dir = self.src.join(src_dir);
        let files = file_list(&dir);
        let find = |keys: &[&str]| -> Option<PathBuf> {
            keys.iter().find_map(|k| files.iter().find(|f| lower_name(f).rsplit_once('.').is_some_and(|(n, _)| n.ends_with(k))).cloned())
        };
        let dst = self.assets_dir.join("textures");
        for (keys, suffix, size) in [
            (&["basecolor"][..], "BaseColor", edge),
            (&["normalogl", "_normal"][..], "Normal", edge),
            (&["roughness"][..], "Roughness", edge / 2),
            (&["ambientocclusion"][..], "AmbientOcclusion", edge / 2),
        ] {
            if let Some(f) = find(keys) {
                save_jpg(&load_resized(&f, size), &dst.join(format!("{dst_stem}_{suffix}.jpg")));
            }
        }
        discover(&self.assets, &format!("textures/{dst_stem}_BaseColor.jpg"))
    }

    /// Copies a Unity-style FBX prop with its maps (Albedo -> jpg, MetallicSmoothness stays png for its alpha).
    fn import_trn(&self, folder: &str, stem: &str) -> String {
        let dir = self.src.join(folder);
        let out = self.assets_dir.join("models/trn");
        std::fs::create_dir_all(&out).unwrap();
        std::fs::copy(dir.join(format!("{stem}.fbx")), out.join(format!("{stem}.fbx"))).unwrap_or_else(|e| panic!("{folder}/{stem}: {e}"));
        // The maps are usually named after the model; some packs name them after the material.
        let tex = if dir.join(format!("{stem}_AlbedoTransparency.png")).exists() {
            stem.to_string()
        } else {
            let hit =
                file_list(&dir).iter().filter_map(|f| lower_name(f).strip_suffix("_albedotransparency.png").map(|n| n.to_string())).next();
            let hit = hit.unwrap_or_else(|| panic!("{folder}/{stem}: no albedo map"));
            file_list(&dir)
                .iter()
                .filter_map(|f| f.file_name().map(|n| n.to_string_lossy().to_string()))
                .find(|n| n.to_ascii_lowercase() == format!("{hit}_albedotransparency.png"))
                .map(|n| n[..n.len() - "_AlbedoTransparency.png".len()].to_string())
                .unwrap()
        };
        let albedo = load_resized(&dir.join(format!("{tex}_AlbedoTransparency.png")), 1024);
        if albedo.to_rgba8().pixels().all(|p| p[3] == 255) {
            save_jpg(&albedo, &out.join(format!("{stem}_AlbedoTransparency.jpg")));
        } else {
            albedo.save(out.join(format!("{stem}_AlbedoTransparency.png"))).unwrap();
        }
        load_resized(&dir.join(format!("{tex}_MetallicSmoothness.png")), 512)
            .save(out.join(format!("{stem}_MetallicSmoothness.png")))
            .unwrap();
        save_jpg(&load_resized(&dir.join(format!("{tex}_Normal.png")), 1024), &out.join(format!("{stem}_Normal.jpg")));
        format!("models/trn/{stem}.fbx")
    }

    fn import_glb(&self, name: &str) -> String {
        let out = self.assets_dir.join("models/kit");
        std::fs::create_dir_all(out.join("Textures")).unwrap();
        let from = self.src.join("Models/GLB format");
        std::fs::copy(from.join(format!("{name}.glb")), out.join(format!("{name}.glb"))).unwrap_or_else(|e| panic!("{name}: {e}"));
        for f in file_list(&from.join("Textures")) {
            std::fs::copy(&f, out.join("Textures").join(f.file_name().unwrap())).unwrap();
        }
        format!("models/kit/{name}.glb")
    }

    fn import_psx(&self, name: &str) -> String {
        let base = self.src.join("PSX Kitchen Props Pack");
        let out = self.assets_dir.join("models/psx");
        std::fs::create_dir_all(out.join("fbx")).unwrap();
        std::fs::create_dir_all(out.join("textures")).unwrap();
        std::fs::copy(base.join(format!("fbx/{name}.fbx")), out.join(format!("fbx/{name}.fbx"))).unwrap_or_else(|e| panic!("{name}: {e}"));
        for f in file_list(&base.join("textures")) {
            std::fs::copy(&f, out.join("textures").join(f.file_name().unwrap())).unwrap();
        }
        format!("models/psx/fbx/{name}.fbx")
    }

    fn spawn(&mut self, name: &str, pos: Vec3, rot: Vec3, scale: Vec3) -> Entity {
        let e = hierarchy::spawn_named(&mut self.scene.world, name);
        self.scene.world.insert(e, Transform { position: pos, rotation: rot, scale });
        e
    }

    fn child(&mut self, parent: Entity, name: &str, pos: Vec3, scale: Vec3) -> Entity {
        let e = self.spawn(name, pos, Vec3::ZERO, scale);
        self.scene.world.insert(e, Parent(parent)); // keep the local transform
        e
    }

    /// A primitive shape with a triplanar PBR material (`metres_per_tile` metres per texture repeat).
    fn solid(&mut self, name: &str, kind: ShapeType, pos: Vec3, scale: Vec3, mat: &PbrSet, metres_per_tile: f32) -> Entity {
        let e = self.spawn(name, pos, Vec3::ZERO, scale);
        self.scene.world.insert(e, Shape::of(kind));
        self.scene.world.insert(e, pbr(String::new(), mat, metres_per_tile));
        e
    }

    fn collider(&mut self, e: Entity, shape: ShapeKind, size: Vec3) {
        self.scene.world.insert(e, Collider { shape, size, ..Default::default() });
    }

    /// An FBX prop with the maps found next to it. `y_offset` lifts models whose pivot is not at the bottom.
    fn fbx(&mut self, name: &str, mesh: &str, pos: Vec3, yaw: f32, scale: f32) -> Entity {
        let set = discover(&self.assets, mesh);
        let e = self.spawn(name, pos, Vec3::new(0.0, yaw, 0.0), Vec3::splat(scale));
        let pbr_on = !set.orm.is_empty();
        self.scene.world.insert(
            e,
            MeshRenderer {
                mesh: mesh.into(),
                texture: set.albedo,
                normal_map: set.normal,
                orm_map: set.orm,
                metallic: if pbr_on { 1.0 } else { 0.0 },
                roughness: if pbr_on { 1.0 } else { 0.8 },
                ..Default::default()
            },
        );
        e
    }

    /// A physics prop: a body entity with a centred collider and the model as its child.
    fn dynamic_prop(
        &mut self,
        name: &str,
        mesh: &str,
        center: Vec3,
        yaw: f32,
        mass: f32,
        shape: ShapeKind,
        size: Vec3,
        model_offset: f32,
    ) -> Entity {
        let body = self.spawn(name, center, Vec3::new(0.0, yaw, 0.0), Vec3::ONE);
        self.scene.world.insert(body, RigidBody { mass, ..Default::default() });
        self.collider(body, shape, size);
        let model = self.fbx(&format!("{name} model"), mesh, Vec3::new(0.0, model_offset, 0.0), 0.0, 1.0);
        self.scene.world.insert(model, Parent(body)); // keep the local transform
        body
    }

    fn kit(&mut self, model: &str, pos: Vec3, yaw: f32) -> Entity {
        let mesh = self.import_glb(model);
        let e = self.spawn(model, pos, Vec3::new(0.0, yaw, 0.0), Vec3::splat(K));
        self.scene.world.insert(e, MeshRenderer { mesh, color: [1.0; 4], roughness: 0.9, ..Default::default() });
        e
    }

    /// A straight wall piece with a box collider (the pivot of kit pieces is at the base).
    fn wall(&mut self, pos: Vec3, yaw: f32) {
        let w = self.kit("wall-fortified", pos, yaw);
        let c = self.child(w, "collider", Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 0.4));
        self.collider(c, ShapeKind::Box, Vec3::ONE);
    }

    fn tower(&mut self, pos: Vec3) {
        let root = self.spawn("Tower", pos, Vec3::ZERO, Vec3::ONE);
        for (i, piece) in ["tower-base", "tower", "tower-top"].iter().enumerate() {
            let mesh = self.import_glb(piece);
            let p = self.child(root, piece, Vec3::new(0.0, K * i as f32, 0.0), Vec3::splat(K));
            self.scene.world.insert(p, MeshRenderer { mesh, color: [1.0; 4], roughness: 0.9, ..Default::default() });
        }
        let c = self.child(root, "collider", Vec3::new(0.0, K * 1.5, 0.0), Vec3::ONE);
        self.collider(c, ShapeKind::Box, Vec3::new(K * 0.85, K * 3.0, K * 0.85));
    }

    fn light(
        &mut self,
        parent: Option<Entity>,
        name: &str,
        pos: Vec3,
        color: [f32; 4],
        intensity: f32,
        range: f32,
        flicker: bool,
    ) -> Entity {
        let e = self.spawn(name, pos, Vec3::ZERO, Vec3::ONE);
        if let Some(p) = parent {
            self.scene.world.insert(e, Parent(p)); // keep the local transform
        }
        self.scene.world.insert(e, Light { kind: LightKind::Point, color, intensity, range, shadows: false, ..Default::default() });
        if flicker {
            self.scene.world.insert(e, script("Flicker", serde_json::json!({})));
        }
        e
    }

    /// A glowing button on a pedestal; returns the button entity.
    fn button(&mut self, name: &str, base: Vec3, channel: &str, prompt: &str, kind: SwitchKind, stone: &PbrSet) -> Entity {
        self.solid(
            &format!("{name} pedestal"),
            ShapeType::Cylinder,
            base + Vec3::new(0.0, 0.45, 0.0),
            Vec3::new(0.7, 0.9, 0.7),
            stone,
            1.5,
        );
        let ped = hierarchy::find_by_name(&self.scene.world, &format!("{name} pedestal")).unwrap();
        self.collider(ped, ShapeKind::Cylinder, Vec3::new(1.0, 1.0, 1.0));
        let b = self.spawn(name, base + Vec3::new(0.0, 0.97, 0.0), Vec3::ZERO, Vec3::new(0.42, 0.14, 0.42));
        self.scene.world.insert(b, Shape::of(ShapeType::Cylinder));
        self.scene.world.insert(
            b,
            MeshRenderer { mesh: String::new(), color: [1.0, 0.2, 0.1, 1.0], emissive: 0.9, roughness: 0.3, ..Default::default() },
        );
        self.collider(b, ShapeKind::Cylinder, Vec3::ONE);
        self.scene.world.insert(b, Switch { kind, channel: channel.into(), prompt: prompt.into(), hold_time: 14.0, ..Default::default() });
        b
    }

    /// A stone gate that slides down into the ground while its channel is active.
    fn gate(&mut self, name: &str, pos: Vec3, width: f32, channel: &str, stone: &PbrSet) -> Entity {
        let g = self.solid(name, ShapeType::Cube, pos + Vec3::new(0.0, 2.25, 0.0), Vec3::new(width, 4.5, 0.9), stone, 3.5);
        self.scene.world.insert(g, RigidBody { kind: BodyKind::Kinematic, ..Default::default() });
        self.collider(g, ShapeKind::Box, Vec3::ONE);
        self.scene
            .world
            .insert(g, Slider { channel: channel.into(), open_offset: Vec3::new(0.0, -4.6, 0.0), speed: 2.2, ..Default::default() });
        g
    }
}

fn script(name: &str, props: serde_json::Value) -> Scripts {
    Scripts(vec![ScriptSlot { name: name.into(), props: props.as_object().cloned().unwrap_or_default() }])
}

fn pbr(mesh: String, set: &PbrSet, metres_per_tile: f32) -> MeshRenderer {
    MeshRenderer {
        mesh,
        color: [1.0; 4],
        roughness: 1.0,
        texture: set.albedo.clone(),
        normal_map: set.normal.clone(),
        orm_map: set.orm.clone(),
        normal_strength: 1.2,
        triplanar: metres_per_tile,
        ..Default::default()
    }
}

/// A procedural boulder with a triplanar PBR material; `collide` adds a slightly shrunken box collider.
fn rock(
    g: &mut Gen,
    rng: &mut Rng,
    sets: &[PbrSet; 2],
    name: &str,
    x: f32,
    z: f32,
    size: Vec3,
    detail: i32,
    rugged: f32,
    set: usize,
    collide: bool,
) -> Entity {
    let yaw = rng.f32() * 360.0;
    let e = g.spawn(name, Vec3::new(x, size.y * 0.285, z), Vec3::new(0.0, yaw, 0.0), Vec3::ONE);
    g.scene
        .world
        .insert(e, Shape { size, segments: detail, radius2: rugged, seed: (rng.f32() * 9000.0) as i32, ..Shape::of(ShapeType::Rock) });
    let tile = (size.x.max(size.z) * 0.6).clamp(1.5, 4.0);
    g.scene.world.insert(e, MeshRenderer { normal_strength: 1.7, ..pbr(String::new(), &sets[set], tile) });
    if collide {
        g.collider(e, ShapeKind::Box, Vec3::new(size.x * 0.8, size.y * 0.75, size.z * 0.8));
    }
    e
}

fn main() {
    let mut args = std::env::args().skip(1);
    let source = PathBuf::from(args.next().unwrap_or_else(|| ".test_assets".into()));
    let games = args.next().unwrap_or_else(|| "games".into());
    let root = Path::new(&games).join("test_fps");
    let mut g = Gen::new(&source, &root);

    // ---- materials
    let dirt = g.import_pbr("JPEG/Stylized_HandpaintedDirt_01", "dirt", 2048);
    let paving = g.import_pbr("TiledSurfaces (22)", "paving", 2048);
    let stone = g.import_pbr("BrickWall (13)", "gate", 2048);
    let rock_sets = [
        g.import_pbr("JPEG/Stylized_HandpaintedStones_02", "rock_grey", 2048),
        g.import_pbr("JPEG/Stylized_HandpaintedStones_01", "rock_sand", 2048),
    ];

    // ---- sky and sun: late afternoon, the day runs on so dusk, night with aurora and moon follow
    g.scene.settings.atmosphere = SkySettings {
        procedural: true,
        time: 16.4,
        speed: 0.07,
        clouds: 0.5,
        cloud_speed: 0.012,
        stars: 0.9,
        haze: 0.38,
        moon_phase: 0.62,
        aurora: 0.85,
        milky_way: 0.9,
    };
    g.scene.settings.exposure = 1.05;
    let sun = g.spawn("Sun", Vec3::new(0.0, 30.0, 0.0), Vec3::new(-35.0, 35.0, 0.0), Vec3::ONE);
    g.scene.world.insert(sun, Light { intensity: 1.0, ..Default::default() });

    // ---- ground, paving
    let ground = g.solid("Ground", ShapeType::Cube, Vec3::new(0.0, -0.5, 0.0), Vec3::new(220.0, 1.0, 220.0), &dirt, 6.0);
    g.collider(ground, ShapeKind::Box, Vec3::ONE);
    g.scene.world.get_mut::<MeshRenderer>(ground).unwrap().color = [0.5, 0.58, 0.4, 1.0];
    let plaza = g.solid("Plaza paving", ShapeType::Cube, Vec3::new(0.0, -0.45, -9.0), Vec3::new(27.0, 1.0, 30.0), &paving, 3.0);
    g.collider(plaza, ShapeKind::Box, Vec3::ONE);
    let vault = g.solid("Vault paving", ShapeType::Cube, Vec3::new(0.0, -0.45, -37.0), Vec3::new(21.0, 1.0, 26.0), &paving, 3.0);
    g.collider(vault, ShapeKind::Box, Vec3::ONE);

    // Grass everywhere except under the paving (four patches around it).
    for (i, (x, z, w, d)) in
        [(-54.0, -10.0, 79.0, 140.0), (54.0, -10.0, 79.0, 140.0), (0.0, 38.0, 28.0, 56.0), (0.0, -70.0, 28.0, 20.0)].iter().enumerate()
    {
        let e = g.spawn(&format!("Grass {i}"), Vec3::new(*x, 0.0, *z), Vec3::ZERO, Vec3::ONE);
        g.scene.world.insert(
            e,
            Grass {
                count: (w * d * 4.5) as i32,
                width: *w,
                depth: *d,
                seed: 3 + i as i32,
                chunk_size: 10.0,
                fade_end: 60.0,
                ..Default::default()
            },
        );
    }

    // ---- plaza walls, towers, gates
    for x in [-4.5, -7.5, -10.5, 4.5, 7.5, 10.5] {
        g.wall(Vec3::new(x, 0.0, 6.0), 0.0);
        g.wall(Vec3::new(x, 0.0, -24.0), 0.0);
    }
    for i in 0..9 {
        let z = 3.0 - 3.0 * i as f32;
        g.wall(Vec3::new(-13.5, 0.0, z), 90.0);
        g.wall(Vec3::new(13.5, 0.0, z), 90.0);
    }
    for (x, z) in [(-13.5, 6.0), (13.5, 6.0), (-13.5, -24.0), (13.5, -24.0), (-3.0, 6.0), (3.0, 6.0)] {
        g.tower(Vec3::new(x, 0.0, z));
    }
    g.gate("Gate A", Vec3::new(0.0, 0.0, 6.0), 4.2, "a", &stone);
    g.gate("Gate B", Vec3::new(0.0, 0.0, -24.0), 4.2, "b", &stone);

    // ---- puzzle: button A at the camp, a pressure plate inside, the beacon in the vault
    g.button("Button A", Vec3::new(4.5, 0.0, 11.0), "a", "Open the gate", SwitchKind::Button, &stone);
    let plate = g.solid("Pressure plate", ShapeType::Cube, Vec3::new(-6.0, 0.03, -14.0), Vec3::new(3.0, 0.08, 3.0), &stone, 2.0);
    g.collider(plate, ShapeKind::Box, Vec3::ONE);
    g.scene.world.insert(plate, Switch { kind: SwitchKind::Plate, channel: "b".into(), prompt: String::new(), ..Default::default() });
    g.button("Beacon button", Vec3::new(0.0, 0.0, -41.0), "c", "Light the beacon", SwitchKind::Toggle, &stone);
    let obelisk = g.solid("Obelisk", ShapeType::Cube, Vec3::new(0.0, -3.4, -46.0), Vec3::new(1.6, 8.0, 1.6), &stone, 2.0);
    g.collider(obelisk, ShapeKind::Box, Vec3::ONE);
    g.scene.world.insert(obelisk, RigidBody { kind: BodyKind::Kinematic, ..Default::default() });
    g.scene.world.insert(obelisk, Slider { channel: "c".into(), open_offset: Vec3::new(0.0, 7.0, 0.0), speed: 1.4, ..Default::default() });
    g.scene.world.get_mut::<MeshRenderer>(obelisk).unwrap().emissive = 0.0;
    let beacon = g.light(Some(obelisk), "Beacon light", Vec3::new(0.0, 0.62, 0.0), [0.35, 0.8, 1.0, 1.0], 6.0, 30.0, true);
    g.scene.world.get_mut::<Light>(beacon).unwrap().shadows = false;

    // ---- the camp (south of the gate)
    let firepit = g.import_trn("Firepit", "trn_Firepit");
    g.fbx("Firepit", &firepit, Vec3::new(-5.0, 0.0, 20.0), 0.0, 1.3);
    let log = g.import_trn("FireLog", "trn_Log");
    for i in 0..5 {
        let a = i as f32 * 1.26;
        let e = g.fbx("Log", &log, Vec3::new(-5.0 + a.cos() * 0.35, 0.12, 20.0 + a.sin() * 0.35), a.to_degrees(), 1.6);
        g.scene.world.get_mut::<Transform>(e).unwrap().rotation.z = 12.0;
    }
    let fire = g.light(None, "Fire light", Vec3::new(-5.0, 1.0, 20.0), [1.0, 0.55, 0.22, 1.0], 4.5, 22.0, true);
    g.scene.world.get_mut::<Light>(fire).unwrap().shadows = false;
    let smoke = g.spawn("Fire smoke", Vec3::new(-5.0, 0.8, 20.0), Vec3::ZERO, Vec3::ONE);
    g.scene.world.insert(smoke, VoxelSmoke::default());
    let fire_collider = g.spawn("Firepit collider", Vec3::new(-5.0, 0.25, 20.0), Vec3::ZERO, Vec3::ONE);
    g.collider(fire_collider, ShapeKind::Cylinder, Vec3::new(2.1, 0.5, 2.1));

    let table = g.import_trn("Tables", "trn_TableLong");
    let t = g.fbx("Table", &table, Vec3::new(5.0, 0.0, 21.0), 90.0, 1.0);
    let tc = g.child(t, "collider", Vec3::new(0.0, 0.42, 0.0), Vec3::ONE);
    g.collider(tc, ShapeKind::Box, Vec3::new(2.4, 0.85, 1.0));
    let candelabra = g.import_trn("Candelabra", "trn_Candelabra");
    let c = g.fbx("Candelabra", &candelabra, Vec3::new(5.0, 0.85, 20.2), 0.0, 1.0);
    g.light(Some(c), "Candle light", Vec3::new(0.0, 0.75, 0.0), [1.0, 0.62, 0.3, 1.0], 2.2, 11.0, true);
    let jug = g.import_trn("Vessels", "trn_Jug");
    g.dynamic_prop("Jug", &jug, Vec3::new(5.0, 1.05, 21.9), 20.0, 1.2, ShapeKind::Cylinder, Vec3::new(0.3, 0.4, 0.3), -0.2);
    let bench = g.import_trn("Chairs", "trn_Bench");
    let b = g.fbx("Bench", &bench, Vec3::new(7.4, 0.0, 21.0), 90.0, 1.0);
    let bc = g.child(b, "collider", Vec3::new(0.0, 0.28, 0.0), Vec3::ONE);
    g.collider(bc, ShapeKind::Box, Vec3::new(1.8, 0.57, 0.4));
    let cauldron = g.import_trn("Cauldron", "trn_Cauldron");
    let cd = g.fbx("Cauldron", &cauldron, Vec3::new(-8.2, 0.0, 22.5), 30.0, 1.2);
    let cdc = g.child(cd, "collider", Vec3::new(0.0, 0.4, 0.0), Vec3::ONE);
    g.collider(cdc, ShapeKind::Cylinder, Vec3::new(1.0, 0.8, 1.0));
    let bedside = g.import_trn("Shelves", "trn_BedsideTable");
    let cb = g.fbx("Bedside table", &bedside, Vec3::new(10.5, 0.0, 27.0), 200.0, 1.4);
    let cbc = g.child(cb, "collider", Vec3::new(0.0, 0.3, 0.0), Vec3::ONE);
    g.collider(cbc, ShapeKind::Box, Vec3::new(0.7, 0.6, 0.6));
    let sack = g.import_trn("Sacks", "trn_FlourSack");
    for (i, (x, z)) in [(9.0, 25.5), (9.9, 24.8), (8.4, 24.6)].iter().enumerate() {
        g.dynamic_prop(
            &format!("Sack {i}"),
            &sack,
            Vec3::new(*x, 0.2, *z),
            i as f32 * 70.0,
            9.0,
            ShapeKind::Box,
            Vec3::new(0.5, 0.3, 0.7),
            -0.12,
        );
    }

    // PSX kitchen corner: low-poly props with their own 256 px textures (toggle the PSX look with P).
    let fridge = g.import_psx("fridge_close");
    let f = g.fbx("Fridge", &fridge, Vec3::new(-11.0, 0.94, 26.0), 160.0, 1.0);
    g.collider(f, ShapeKind::Box, Vec3::new(0.9, 1.87, 0.8));
    let ptable = g.import_psx("table_01");
    let pt = g.fbx("Kitchen table", &ptable, Vec3::new(-8.0, 0.44, 27.0), 0.0, 1.0);
    g.collider(pt, ShapeKind::Box, Vec3::new(1.25, 0.88, 1.25));
    let microwave = g.import_psx("microwave");
    g.dynamic_prop("Microwave", &microwave, Vec3::new(-8.2, 1.07, 27.1), 15.0, 6.0, ShapeKind::Box, Vec3::new(0.65, 0.36, 0.35), 0.0);
    let pizza = g.import_psx("pizza_box_clean");
    g.dynamic_prop("Pizza box", &pizza, Vec3::new(-7.7, 0.98, 26.8), 40.0, 0.5, ShapeKind::Box, Vec3::new(0.5, 0.1, 0.5), 0.0);
    let can = g.import_psx("can_cola_cola");
    let beer = g.import_psx("beer_bottle");
    for i in 0..5 {
        let (mesh, size, h) =
            if i % 2 == 0 { (&can, Vec3::new(0.07, 0.15, 0.07), 0.075) } else { (&beer, Vec3::new(0.08, 0.29, 0.08), 0.15) };
        g.dynamic_prop(
            &format!("Drink {i}"),
            mesh,
            Vec3::new(-7.6 + i as f32 * 0.16, 0.89 + h + 0.02, 27.4),
            0.0,
            0.4,
            ShapeKind::Cylinder,
            size,
            0.0,
        );
    }
    let chair = g.import_psx("chair");
    for (i, (x, z, yaw)) in [(-9.6, 27.4, 90.0), (-6.5, 26.6, -80.0)].iter().enumerate() {
        g.dynamic_prop(&format!("Chair {i}"), &chair, Vec3::new(*x, 0.55, *z), *yaw, 4.0, ShapeKind::Box, Vec3::new(0.48, 1.08, 0.47), 0.0);
    }

    // Barrels to carry: two at the camp, three in the plaza.
    let barrel = g.import_trn("Barrel", "trn_Barrel");
    for (i, (x, z)) in [(2.0, 17.0), (-1.4, 15.5), (7.5, -3.5), (9.0, -6.0), (4.0, -9.5)].iter().enumerate() {
        g.dynamic_prop(
            &format!("Barrel {i}"),
            &barrel,
            Vec3::new(*x, 0.52, *z),
            i as f32 * 37.0,
            12.0,
            ShapeKind::Cylinder,
            Vec3::new(0.95, 1.0, 0.95),
            -0.5,
        );
    }
    // Plaza candelabras on the back wall.
    for x in [-7.0, 7.0] {
        let t = g.import_trn("Lamp", "trn_Lamp");
        let l = g.fbx("Wall lamp", &t, Vec3::new(x, 2.4, -23.4), 0.0, 1.0);
        g.light(Some(l), "Lamp light", Vec3::new(0.0, -0.1, 0.8), [1.0, 0.7, 0.4, 1.0], 2.5, 13.0, true);
    }

    // ---- rocks: a ring of boulders as a natural border, a rock garden, small stones to throw
    let mut rng = Rng::new(7);
    let mut jit = Rng::new(99);
    let keep_clear = |x: f32, z: f32| -> bool {
        let in_plaza = x.abs() < 17.0 && z < 10.0 && z > -28.0;
        let in_vault = x.abs() < 14.0 && z < -26.0 && z > -52.0;
        let camp = x.abs() < 15.0 && z > 8.0 && z < 31.0;
        in_plaza || in_vault || camp
    };
    // Border: boulders in a rectangle around the playable area.
    let (half_x, north, south) = (46.0, -58.0, 44.0);
    let mut n = 0;
    let mut x = -half_x;
    while x <= half_x {
        for z in [north, south] {
            let s = Vec3::new(6.0 + rng.f32() * 5.0, 3.5 + rng.f32() * 3.5, 5.0 + rng.f32() * 4.5);
            rock(
                &mut g,
                &mut rng,
                &rock_sets,
                &format!("Border rock {n}"),
                x + jit.f32() * 3.0,
                z + jit.f32() * 3.0,
                s,
                5,
                0.9 + jit.f32() * 0.3,
                (n % 2) as usize,
                true,
            );
            n += 1;
        }
        x += 7.5;
    }
    let mut z = north + 7.5;
    while z < south {
        for x in [-half_x, half_x] {
            let s = Vec3::new(5.5 + rng.f32() * 5.0, 3.5 + rng.f32() * 3.5, 6.0 + rng.f32() * 4.5);
            rock(
                &mut g,
                &mut rng,
                &rock_sets,
                &format!("Border rock {n}"),
                x + jit.f32() * 3.0,
                z + jit.f32() * 3.0,
                s,
                5,
                0.9 + jit.f32() * 0.3,
                (n % 2) as usize,
                true,
            );
            n += 1;
        }
        z += 7.5;
    }
    // Rock garden: scattered boulders of every size.
    let mut placed = 0;
    let mut tries = 0;
    while placed < 38 && tries < 400 {
        tries += 1;
        let (x, z) = (-40.0 + rng.f32() * 80.0, -50.0 + rng.f32() * 90.0);
        if keep_clear(x, z) {
            continue;
        }
        let big = rng.f32();
        let s = if big > 0.78 {
            Vec3::new(3.5 + rng.f32() * 2.5, 2.2 + rng.f32() * 1.8, 3.0 + rng.f32() * 2.5)
        } else if big > 0.4 {
            Vec3::new(1.6 + rng.f32() * 1.4, 1.0 + rng.f32() * 0.8, 1.4 + rng.f32() * 1.2)
        } else {
            Vec3::new(0.6 + rng.f32() * 0.7, 0.4 + rng.f32() * 0.4, 0.55 + rng.f32() * 0.6)
        };
        let detail = if s.x > 3.0 {
            5
        } else if s.x > 1.4 {
            4
        } else {
            3
        };
        rock(&mut g, &mut rng, &rock_sets, &format!("Rock {placed}"), x, z, s, detail, 0.7 + jit.f32() * 0.5, placed % 2, true);
        placed += 1;
    }
    // Hero rocks beside the paths, in plaza corners and in the vault.
    for (i, (x, z, s)) in [
        (12.0, 14.0, Vec3::new(4.8, 3.4, 4.0)),
        (-14.5, 14.5, Vec3::new(5.2, 3.0, 4.4)),
        (-9.5, -3.0, Vec3::new(2.8, 1.8, 2.4)),
        (10.0, -19.0, Vec3::new(3.2, 2.2, 2.8)),
        (-9.0, -29.5, Vec3::new(3.6, 2.6, 3.0)),
        (9.0, -33.0, Vec3::new(4.0, 2.8, 3.4)),
        (-6.0, -46.0, Vec3::new(4.4, 3.2, 3.6)),
    ]
    .iter()
    .enumerate()
    {
        rock(&mut g, &mut rng, &rock_sets, &format!("Hero rock {i}"), *x, *z, *s, 6, 1.0, i % 2, true);
    }
    // Small stones you can pick up and throw.
    for i in 0..8 {
        let r = 0.18 + rng.f32() * 0.14;
        let (x, z) = (-2.5 + rng.f32() * 6.0, 24.0 + rng.f32() * 5.0);
        let e = rock(&mut g, &mut rng, &rock_sets, &format!("Stone {i}"), x, z, Vec3::new(r * 2.4, r * 2.0, r * 2.2), 4, 0.9, i % 2, false);
        g.scene.world.get_mut::<Transform>(e).unwrap().position.y = r * 2.0;
        g.scene.world.insert(e, RigidBody { mass: 3.0, ..Default::default() });
        g.scene.world.insert(
            e,
            Collider { shape: ShapeKind::Sphere, size: Vec3::splat(r * 1.9), friction: 0.9, restitution: 0.15, ..Default::default() },
        );
    }

    // Invisible walls just outside the border, and trees beyond.
    for (name, pos, size) in [
        ("North limit", Vec3::new(0.0, 10.0, -66.0), Vec3::new(200.0, 20.0, 1.0)),
        ("South limit", Vec3::new(0.0, 10.0, 52.0), Vec3::new(200.0, 20.0, 1.0)),
        ("West limit", Vec3::new(-54.0, 10.0, -7.0), Vec3::new(1.0, 20.0, 130.0)),
        ("East limit", Vec3::new(54.0, 10.0, -7.0), Vec3::new(1.0, 20.0, 130.0)),
    ] {
        let e = g.spawn(name, pos, Vec3::ZERO, Vec3::ONE);
        g.collider(e, ShapeKind::Box, size);
    }
    for i in 0..26 {
        let a = i as f32 / 26.0 * std::f32::consts::TAU + rng.f32() * 0.2;
        let r = 62.0 + rng.f32() * 22.0;
        let model = if i % 3 == 0 { "tree-shrub" } else { "tree-large" };
        let mesh = g.import_glb(model);
        let e = g.spawn(
            model,
            Vec3::new(a.cos() * r, 0.0, a.sin() * r - 6.0),
            Vec3::new(0.0, rng.f32() * 360.0, 0.0),
            Vec3::splat(9.0 + rng.f32() * 4.0),
        );
        g.scene.world.insert(e, MeshRenderer { mesh, color: [1.0; 4], roughness: 0.9, cast_shadows: false, ..Default::default() });
    }

    // ---- player
    let player = g.spawn("Player", Vec3::new(0.0, 1.0, 29.0), Vec3::ZERO, Vec3::ONE);
    g.scene
        .world
        .insert(player, RigidBody { mass: 70.0, lock_rotation: true, linear_damping: 0.0, angular_damping: 0.0, ..Default::default() });
    g.scene.world.insert(
        player,
        Collider { shape: ShapeKind::Capsule, size: Vec3::new(0.7, 1.8, 0.7), friction: 0.0, restitution: 0.0, ..Default::default() },
    );
    g.scene.world.insert(
        player,
        script("FpsPlayer", serde_json::json!({ "objective": "Open the gate, weigh down the plate with a barrel, light the beacon" })),
    );
    let eyes = g.child(player, "Eyes", Vec3::new(0.0, 0.7, 0.0), Vec3::ONE);
    g.scene.world.insert(eyes, Camera { fov: 72.0, near: 0.05, far: 1600.0, ..Default::default() });
    let flash = g.child(eyes, "Flashlight", Vec3::new(0.15, -0.1, 0.0), Vec3::ONE);
    g.scene.world.insert(
        flash,
        Light {
            kind: LightKind::Spot,
            color: [1.0, 0.95, 0.85, 1.0],
            intensity: 4.0,
            range: 30.0,
            angle: 30.0,
            softness: 0.45,
            shadows: false,
        },
    );

    // ---- write the project
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/main.scene.json"), g.scene.to_json(&Registry::default())).unwrap();
    std::fs::create_dir_all(root.join("scripts/src")).unwrap();
    std::fs::write(root.join("scripts/Cargo.toml"), "[package]\nname = \"test_fps_scripts\"\nversion.workspace = true\nedition.workspace = true\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nkvas_engine.workspace = true\n").unwrap();
    std::fs::write(root.join("scripts/src/lib.rs"), "include!(\"../../../test_shared/scripts.rs\");\n").unwrap();
    Project {
        root: root.clone(),
        config: ProjectConfig {
            name: "Test: FPS yard".into(),
            start_scene: "scenes/main.scene.json".into(),
            scripts_package: "test_fps_scripts".into(),
            ..Default::default()
        },
    }
    .save()
    .unwrap();
    println!("wrote {} ({} objects)", root.display(), g.scene.world.entities().len());
}
