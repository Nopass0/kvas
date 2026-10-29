//! Builds `games/last_service/assets/scenes/main.scene.json`, the single scene of LAST SERVICE.
//! `cargo run -p kvas_engine --example make_last_service` (assets first: `python tools/import_last_service.py`).
//!
//! The production plan gives positions in Blender metres (+Y from the entrance to the kitchen);
//! here `b(x, y)` turns them into engine coordinates (x, -y). The world is built from the parts
//! that `tools/split_world_glb.py` cut out of the production's merged mesh: one object per zone
//! and material, furniture piece by piece; each part collides with its own triangles. Doors swing
//! on hinges, and every object carries its own scripts (see `games/last_service/scripts/src`).
use glam::Vec3;
use kvas_engine::decal::Decal;
use kvas_engine::fx::PostEffect;
use kvas_engine::prelude::*;
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use serde_json::{json, Value};

const H: f32 = 3.1; // room height

fn tags(t: &str) -> Tags {
    Tags { value: t.into() }
}

fn b(x: f32, y: f32) -> Vec3 {
    Vec3::new(x, 0.0, -y)
}

/// A script slot with its public variables.
fn script(name: &str, props: Value) -> ScriptSlot {
    ScriptSlot { name: name.into(), props: props.as_object().cloned().unwrap_or_default() }
}

/// Parts that are only looked at: tree cards, decals, painted signs (no collision).
fn decorative(material: &str) -> bool {
    let m = material.to_ascii_lowercase();
    ["tree", "decal", "_ink", "lamp_warm", "fern", "grass", "weeds", "shrub"].iter().any(|k| m.contains(k))
}

struct S {
    scene: Scene,
}

impl S {
    fn ent(&mut self, name: &str, pos: Vec3, yaw: f32, scale: Vec3) -> Entity {
        let e = self.scene.spawn(name);
        self.scene.world.insert(e, Transform { position: pos, rotation: Vec3::new(0.0, yaw, 0.0), scale });
        e
    }
    fn model(&mut self, name: &str, mesh: &str, pos: Vec3, yaw: f32, scale: f32) -> Entity {
        let e = self.ent(name, pos, yaw, Vec3::splat(scale));
        self.scene.world.insert(e, MeshRenderer { mesh: mesh.into(), color: [1.0; 4], roughness: 0.8, ..Default::default() });
        e
    }
    /// A prop of the PSX horror pack (they share one texture atlas).
    fn prop(&mut self, name: &str, file: &str, pos: Vec3, yaw: f32, scale: f32) -> Entity {
        let e = self.model(name, &format!("models/props/{file}.glb"), pos, yaw, scale);
        self.scene.world.get_mut::<MeshRenderer>(e).unwrap().texture = "models/props/textures/atlas.png".into();
        e
    }
    /// Invisible solid box (the ground under everything, the edge of the walkable area).
    fn solid(&mut self, name: &str, center: Vec3, size: Vec3) -> Entity {
        let e = self.ent(name, center, 0.0, Vec3::ONE);
        self.scene.world.insert(e, Collider { size, ..Default::default() });
        e
    }
    /// A visible box of the diner's shell (ceiling, wall ends, the wall above a door) with a
    /// collider of the same size.
    fn shell(&mut self, name: &str, center: Vec3, size: Vec3, color: [f32; 4], parent: Entity) -> Entity {
        let e = self.ent(name, center, 0.0, size);
        self.scene.world.insert(e, Shape::of(ShapeType::Cube));
        self.scene.world.insert(e, MeshRenderer { color, roughness: 0.92, ..Default::default() });
        self.scene.world.insert(e, Collider { size: Vec3::ONE, ..Default::default() });
        hierarchy::set_parent(&mut self.scene.world, e, Some(parent));
        e
    }
    fn scripts(&mut self, e: Entity, slots: Vec<ScriptSlot>) {
        self.scene.world.insert(e, Scripts(slots));
    }
    fn group(&mut self, name: &str, parent: Option<Entity>) -> Entity {
        let e = self.ent(name, Vec3::ZERO, 0.0, Vec3::ONE);
        hierarchy::set_parent(&mut self.scene.world, e, parent);
        e
    }
    /// Makes an entity something the player can look at and use (a trigger box around it).
    fn usable(&mut self, e: Entity, size: Vec3) {
        // The trigger lives on a child so the model can keep its own scale.
        let t = self.scene.spawn("Use area");
        let scale = self.scene.world.get::<Transform>(e).unwrap().scale;
        let inv = Vec3::ONE / scale;
        self.scene.world.insert(t, Transform { position: Vec3::Y * size.y * 0.5 * inv.y, scale: inv, ..Default::default() });
        hierarchy::set_parent_local(&mut self.scene.world, t, Some(e));
        self.scene.world.insert(t, Collider { size, is_trigger: true, ..Default::default() });
        self.scene.world.insert(t, tags("use"));
        let name = hierarchy::name_of(&self.scene.world, e).to_string();
        self.scene.world.insert(t, Name(format!("{name}#use")));
    }
    fn light(&mut self, name: &str, pos: Vec3, color: [f32; 4], intensity: f32, range: f32) -> Entity {
        let e = self.ent(name, pos, 0.0, Vec3::ONE);
        self.scene.world.insert(e, Light { kind: LightKind::Point, color, intensity, range, ..Default::default() });
        e
    }
    fn sign(&mut self, name: &str, texture: &str, pos: Vec3, rot: Vec3, size: f32) -> Entity {
        let e = self.scene.spawn(name);
        self.scene.world.insert(e, Transform { position: pos, rotation: rot, scale: Vec3::new(size, 0.2, size) });
        self.scene.world.insert(e, Decal { texture: texture.into(), color: [1.0; 4], angle_fade: 0.2, shape: kvas_engine::decal::DecalShape::Image, roughness: 0.7, max_distance: 30.0, ..Default::default() });
        e
    }
    fn sound(&mut self, name: &str, clip: &str, pos: Vec3, volume: f32, max: f32) {
        let e = self.ent(name, pos, 0.0, Vec3::ONE);
        self.scene.world.insert(e, AudioSource { clip: clip.into(), volume, looping: true, play_on_start: true, spatial: true, min_distance: 1.5, max_distance: max, ..Default::default() });
    }
    /// The wall of the diner from `a` to `b` (engine xz) with doorways `(centre along the wall,
    /// width)`. The production mesh already has the walls; this thin inner layer closes the gaps
    /// it left (wall ends, the strip under the ceiling) and the wall above each doorway.
    fn wall(&mut self, name: &str, a: Vec3, b: Vec3, gaps: &[(f32, f32)], parent: Entity) {
        const PLASTER: [f32; 4] = [0.36, 0.43, 0.35, 1.0];
        let dir = (b - a).normalize();
        let len = (b - a).length();
        let along_x = dir.x.abs() > 0.5;
        let size = |l: f32, h: f32| if along_x { Vec3::new(l, h, 0.1) } else { Vec3::new(0.1, h, l) };
        let mut cuts: Vec<(f32, f32)> = gaps.iter().map(|(c, w)| (c - w * 0.5, c + w * 0.5)).collect();
        cuts.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut start = 0.0;
        for (s, e) in cuts.iter().copied().chain(std::iter::once((len, len))) {
            if s > start {
                let mid = a + dir * (start + s) * 0.5 + Vec3::Y * H * 0.5;
                self.shell(name, mid, size(s - start, H), PLASTER, parent);
            }
            if e > s {
                // Above the door, up to the ceiling.
                let mid = a + dir * (s + e) * 0.5 + Vec3::Y * (2.2 + (H - 2.2) * 0.5);
                self.shell(&format!("{name} lintel"), mid, size(e - s, H - 2.2), PLASTER, parent);
            }
            start = e;
        }
    }
    /// A door on hinges in a doorway: the hinge object carries the `Door` script and turns; the
    /// leaf model and its collider are its children. `swing` is +1 or -1 (which side it opens to).
    fn door(&mut self, name: &str, center: Vec3, along_x: bool, width: f32, props: Value, parent: Entity) -> Entity {
        // The leaf extends from the hinge along the hinge's local +X.
        let (hinge, yaw) = if along_x { (center - Vec3::X * width * 0.5, 0.0) } else { (center + Vec3::Z * width * 0.5, 90.0) };
        let e = self.ent(name, hinge, yaw, Vec3::ONE);
        hierarchy::set_parent(&mut self.scene.world, e, Some(parent));
        self.scripts(e, vec![script("Door", props)]);
        let leaf = self.model(&format!("{name} leaf"), "models/service_door_01.glb", Vec3::new(width * 0.5, 0.0, 0.0), 0.0, 1.0);
        self.scene.world.get_mut::<Transform>(leaf).unwrap().scale = Vec3::new(width / 0.96, 1.05, 1.0);
        hierarchy::set_parent_local(&mut self.scene.world, leaf, Some(e));
        let col = self.ent(&format!("{name} collider"), Vec3::new(width * 0.5, 1.1, 0.03), 0.0, Vec3::ONE);
        self.scene.world.insert(col, Collider { size: Vec3::new(width, 2.2, 0.08), ..Default::default() });
        hierarchy::set_parent_local(&mut self.scene.world, col, Some(e));
        e
    }
}

fn main() {
    let mut scene = Scene::default();
    let st = &mut scene.settings;
    st.ambient = [0.035, 0.045, 0.045];
    st.sky = [0.02, 0.03, 0.03];
    st.atmosphere.procedural = true;
    st.atmosphere.time = 23.0;
    st.atmosphere.clouds = 0.9;
    st.atmosphere.stars = 0.0;
    st.atmosphere.speed = 0.0;
    st.environment_map = "backgrounds/last_service_night_2k.hdr".into();
    st.environment_intensity = 0.22;
    st.fog = 0.035;
    st.filmic = 0.7;
    st.exposure = 1.1;
    st.view_distance = 160.0;
    st.post.vignette = 0.3;
    st.post.film_grain = 0.06;
    st.post.bloom = 0.3;
    st.post.bloom_threshold = 0.75;
    st.shadows.range = 30.0;
    let mut s = S { scene };

    // ---------- the world: parts of the production mesh, grouped by zone
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/last_service/assets");
    let index: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(root.join("world/parts/index.json")).expect("run tools/import_last_service.py first")).unwrap();
    let environment = s.group("Environment", None);
    let mut zones = std::collections::HashMap::new();
    let tables: Vec<Vec3> = index
        .iter()
        .filter(|p| p["material"] == "table.001" && p["size"][1].as_f64().unwrap_or(0.0) > 0.5)
        .map(|p| Vec3::new(p["position"][0].as_f64().unwrap() as f32, 0.0, p["position"][2].as_f64().unwrap() as f32))
        .collect();
    let mut chairs = 0;
    for p in &index {
        let zone = p["zone"].as_str().unwrap_or("Campus").to_string();
        let parent = *zones.entry(zone.clone()).or_insert_with(|| s.group(&zone, Some(environment)));
        let material = p["material"].as_str().unwrap_or("");
        let file = p["file"].as_str().unwrap();
        let pos = Vec3::new(p["position"][0].as_f64().unwrap() as f32, p["position"][1].as_f64().unwrap() as f32, p["position"][2].as_f64().unwrap() as f32);
        let label = match (p["kind"].as_str(), material) {
            (Some("piece"), "chair") => {
                chairs += 1;
                format!("Chair {chairs}")
            }
            (Some("piece"), _) => format!("Table {}", file.rsplit('_').next().unwrap_or("").trim_end_matches(".glb")),
            _ => material.to_string(),
        };
        let e = s.model(&label, file, pos, 0.0, 1.0);
        hierarchy::set_parent(&mut s.scene.world, e, Some(parent));
        if let Some(m) = s.scene.world.get_mut::<MeshRenderer>(e) {
            m.normal_strength = 0.55;
            m.reflectivity = 0.15;
            m.roughness = 0.9;
        }
        if material == "chair" {
            // A chair faces its table: its back (the tall side) points away from it.
            let back = Vec3::new(p["back"][0].as_f64().unwrap_or(0.0) as f32, 0.0, p["back"][1].as_f64().unwrap_or(0.0) as f32);
            let table = tables.iter().min_by(|a, c| a.distance(pos).total_cmp(&c.distance(pos))).copied().unwrap_or(pos);
            let want = (pos - table).with_y(0.0).normalize_or_zero();
            let yaw = want.x.atan2(want.z) - back.x.atan2(back.z);
            s.scene.world.get_mut::<Transform>(e).unwrap().rotation.y = yaw.to_degrees();
            // Chairs can be pushed around.
            s.scene.world.insert(e, Collider { shape: ShapeKind::Hull, friction: 0.7, ..Default::default() });
            s.scene.world.insert(e, RigidBody { mass: 6.0, linear_damping: 0.6, angular_damping: 0.8, ..Default::default() });
        } else if p["size"][1].as_f64().unwrap_or(0.0) > 0.25 && !decorative(material) {
            s.scene.world.insert(e, Collider { shape: ShapeKind::Mesh, ..Default::default() });
        }
    }
    s.solid("Ground", Vec3::new(5.0, -0.5, -5.0), Vec3::new(140.0, 1.0, 140.0));
    for (c, size) in [
        (Vec3::new(5.0, 2.0, 15.0), Vec3::new(50.0, 4.0, 0.5)),
        (Vec3::new(5.0, 2.0, -24.0), Vec3::new(50.0, 4.0, 0.5)),
        (Vec3::new(-14.0, 2.0, -5.0), Vec3::new(0.5, 4.0, 40.0)),
        (Vec3::new(24.0, 2.0, -5.0), Vec3::new(0.5, 4.0, 40.0)),
    ] {
        s.solid("Limit", c, size);
    }

    // ---------- the diner's shell: the ceiling and the inner wall layer (closes the mesh's gaps)
    let diner = zones["Diner"];
    let shell = s.group("Diner shell", Some(diner));
    s.shell("Ceiling", Vec3::new(6.0, H + 0.04, -7.5), Vec3::new(12.2, 0.08, 15.2), [0.2, 0.21, 0.2, 1.0], shell);
    s.wall("Wall south", b(0.0, 0.0), b(12.0, 0.0), &[(6.0, 1.3)], shell);
    s.wall("Wall east", b(12.0, 0.0), b(12.0, 15.0), &[], shell);
    s.wall("Wall west hall", b(0.0, 0.0), b(0.0, 8.0), &[(6.5, 1.2)], shell);
    s.wall("Wall west kitchen", b(0.0, 8.0), b(0.0, 13.0), &[(2.0, 1.3)], shell);
    s.wall("Wall north kitchen", b(0.0, 13.0), b(7.0, 13.0), &[], shell);
    s.wall("Wall north storage", b(7.0, 15.0), b(12.0, 15.0), &[], shell);
    s.wall("Wall storage west", b(7.0, 13.0), b(7.0, 15.0), &[], shell);
    s.wall("Wall hall-kitchen", b(0.0, 8.0), b(12.0, 8.0), &[(6.5, 1.2)], shell);
    s.wall("Wall kitchen-office", b(7.0, 8.0), b(7.0, 11.0), &[(1.1, 1.1)], shell);
    s.wall("Wall office-storage", b(7.0, 11.0), b(12.0, 11.0), &[(2.0, 1.1)], shell);
    s.wall("Wall kitchen-storage", b(7.0, 11.0), b(7.0, 13.0), &[(1.1, 1.1)], shell);

    // ---------- doors on hinges (the director locks and unlocks them as the story goes)
    let doors = s.group("Doors", Some(diner));
    let locked = |en: &str, ru: &str, swing: f32| json!({ "locked": true, "name_en": en, "name_ru": ru, "angle": 95.0 * swing });
    s.door("Door entrance", b(6.0, 0.0), true, 1.3, json!({ "locked": false, "name_en": "front door", "name_ru": "входную дверь", "angle": -100.0 }), doors);
    s.door("Door kitchen", b(6.5, 8.0), true, 1.2, locked("kitchen door", "дверь на кухню", -1.0), doors);
    s.door("Door office", b(7.0, 9.1), false, 1.1, locked("office door", "дверь конторки", 1.0), doors);
    s.door("Door storage", b(9.0, 11.0), true, 1.1, locked("storage door", "дверь кладовой", -1.0), doors);
    s.door("Door ring", b(7.0, 12.1), false, 1.1, locked("back door", "заднюю дверь", 1.0), doors);
    s.door("Door emergency", b(0.0, 10.0), false, 1.3, locked("emergency exit", "аварийную дверь", -1.0), doors);
    s.door("Door side yard", b(0.0, 6.5), false, 1.2, locked("yard door", "дверь во двор", -1.0), doors);

    // ---------- the kitchen extractor fan: blades that turn, and its hum
    let fan = s.ent("Kitchen fan rotor", b(3.0, 12.72) + Vec3::Y * 2.11, 0.0, Vec3::ONE);
    hierarchy::set_parent(&mut s.scene.world, fan, Some(diner));
    s.scripts(fan, vec![script("Spinner", json!({ "speed": 540.0, "axis": "z", "spin_up": 3.0 }))]);
    let hub = s.ent("Fan hub", Vec3::ZERO, 0.0, Vec3::new(0.09, 0.05, 0.09));
    s.scene.world.insert(hub, Shape::of(ShapeType::Cylinder));
    s.scene.world.get_mut::<Transform>(hub).unwrap().rotation.x = 90.0;
    s.scene.world.insert(hub, MeshRenderer { color: [0.25, 0.24, 0.22, 1.0], metallic: 0.6, roughness: 0.5, ..Default::default() });
    hierarchy::set_parent_local(&mut s.scene.world, hub, Some(fan));
    for i in 0..4 {
        let angle = i as f32 * 90.0;
        let a = angle.to_radians();
        let blade = s.ent("Fan blade", Vec3::new(a.cos() * 0.15, a.sin() * 0.15, 0.0), 0.0, Vec3::new(0.24, 0.075, 0.008));
        s.scene.world.get_mut::<Transform>(blade).unwrap().rotation = Vec3::new(18.0, 0.0, angle);
        s.scene.world.insert(blade, Shape::of(ShapeType::Cube));
        s.scene.world.insert(blade, MeshRenderer { color: [0.42, 0.4, 0.36, 1.0], metallic: 0.7, roughness: 0.45, ..Default::default() });
        hierarchy::set_parent_local(&mut s.scene.world, blade, Some(fan));
    }
    s.sound("Fan hum", "audio/sfx/amb_fan.wav", b(3.0, 12.7) + Vec3::Y * 2.1, 0.35, 7.0);

    // ---------- the player: capsule, eyes with the camera and its shaders, the flashlight
    let player = s.ent("Lida", b(6.0, -8.0) + Vec3::Y * 1.0, 0.0, Vec3::ONE);
    s.scene.world.insert(player, Collider { shape: ShapeKind::Capsule, size: Vec3::new(0.5, 1.7, 0.5), friction: 0.0, ..Default::default() });
    s.scene.world.insert(player, RigidBody { mass: 70.0, lock_rotation: true, linear_damping: 0.0, ccd: true, ..Default::default() });
    s.scripts(player, vec![script("PlayerController", json!({})), script("Interactor", json!({})), script("Inventory", json!({})), script("RoomAcoustics", json!({}))]);
    let eyes = s.ent("Eyes", Vec3::new(0.0, 0.62, 0.0), 0.0, Vec3::ONE);
    hierarchy::set_parent_local(&mut s.scene.world, eyes, Some(player));
    s.scripts(eyes, vec![script("MouseLook", json!({})), script("CameraFx", json!({}))]);
    s.scene.world.insert(
        eyes,
        Camera {
            fov: 68.0,
            near: 0.05,
            far: 220.0,
            effects: vec![
                PostEffect { shader: "shaders/ls/last_service_vhs.wgsl".into(), enabled: true, params: [0.45, 0.35, 0.45, 0.0] },
                PostEffect { shader: "shaders/game/fear_pulse.wgsl".into(), enabled: false, params: [0.0, 110.0, 0.0, 0.0] },
                PostEffect { shader: "shaders/game/blood_vision.wgsl".into(), enabled: false, params: [0.0; 4] },
                PostEffect { shader: "shaders/game/ps1_dither.wgsl".into(), enabled: false, params: [240.0, 32.0, 0.0, 0.0] },
                PostEffect { shader: "shaders/game/dream_haze.wgsl".into(), enabled: false, params: [0.8, 0.0, 0.0, 0.0] },
            ],
            ..Default::default()
        },
    );
    let flashlight = s.ent("Flashlight", Vec3::new(0.18, -0.15, -0.1), 0.0, Vec3::ONE);
    hierarchy::set_parent_local(&mut s.scene.world, flashlight, Some(eyes));
    s.scene.world.insert(flashlight, Light { kind: LightKind::Spot, color: [1.0, 0.93, 0.8, 1.0], intensity: 0.0, range: 16.0, angle: 26.0, shadows: false, ..Default::default() });
    s.scripts(flashlight, vec![script("Flashlight", json!({ "intensity": 5.0 }))]);
    // Menu camera: a slow look at the lit diner from the road.
    let menu = s.ent("Menu camera", Vec3::new(12.5, 1.9, 9.5), 0.0, Vec3::ONE);
    s.scene.world.get_mut::<Transform>(menu).unwrap().look_at(Vec3::new(6.0, 1.6, -2.0), Vec3::Y);
    s.scene.world.insert(menu, Camera { fov: 52.0, primary: false, far: 220.0, effects: vec![PostEffect { shader: "shaders/ls/last_service_vhs.wgsl".into(), enabled: true, params: [0.6, 0.4, 0.5, 0.0] }], ..Default::default() });
    s.scripts(menu, vec![script("MenuCamera", json!({}))]);
    // The story and the screens have objects of their own.
    let game = s.ent("Game", Vec3::ZERO, 0.0, Vec3::ONE);
    s.scripts(game, vec![script("GameDirector", json!({}))]);
    let ui = s.ent("UI", Vec3::ZERO, 0.0, Vec3::ONE);
    s.scripts(ui, vec![script("GameUI", json!({}))]);

    // ---------- light: the warm lamp of the diner, the cold kitchen, a dying street lamp
    let hall = s.light("Hall lamp", Vec3::new(6.0, 2.8, -4.0), [1.0, 0.82, 0.58, 1.0], 3.5, 11.0);
    s.scene.world.insert(hall, tags("flicker hall_power"));
    s.light("Table lamp", Vec3::new(8.3, 2.4, -4.5), [1.0, 0.75, 0.5, 1.0], 2.2, 5.0);
    let counter_lamp = s.light("Counter lamp", Vec3::new(3.0, 2.6, -6.6), [1.0, 0.8, 0.55, 1.0], 2.0, 6.0);
    s.scene.world.insert(counter_lamp, tags("hall_power"));
    let kitchen = s.light("Kitchen lamp", Vec3::new(3.5, 2.8, -10.5), [0.75, 0.95, 0.9, 1.0], 2.4, 8.0);
    s.scene.world.insert(kitchen, tags("flicker"));
    let office = s.light("Office lamp", Vec3::new(9.5, 2.7, -9.6), [0.95, 0.85, 0.65, 1.0], 0.0, 6.0);
    s.scene.world.insert(office, tags("archive_power"));
    let storage = s.light("Storage lamp", Vec3::new(9.5, 2.6, -13.0), [1.0, 0.25, 0.15, 1.0], 1.2, 5.5);
    s.scene.world.insert(storage, tags("emergency"));
    s.light("Street lamp", Vec3::new(9.0, 4.5, 6.0), [1.0, 0.6, 0.3, 1.0], 6.0, 16.0);
    let yard = s.light("Yard lamp", Vec3::new(-2.5, 3.2, -12.0), [0.6, 0.8, 1.0, 1.0], 2.5, 9.0);
    s.scene.world.insert(yard, tags("flicker"));
    let moon = s.ent("Moon", Vec3::new(0.0, 20.0, 0.0), 0.0, Vec3::ONE);
    s.scene.world.get_mut::<Transform>(moon).unwrap().rotation = Vec3::new(-40.0, -30.0, 0.0);
    s.scene.world.insert(moon, Light { color: [0.45, 0.6, 0.65, 1.0], intensity: 0.22, ..Default::default() });

    // ---------- people
    // On the staff step behind the counter.
    let anna = s.model("Anna", "models/characters/female_secretary.fbx", b(1.5, 7.72) + Vec3::Y * 0.12, 0.0, 1.0);
    s.scene.world.get_mut::<MeshRenderer>(anna).unwrap().texture = "models/characters/secretary_tex.png".into();
    let pavel = s.model("Pavel", "models/characters/male_casual.fbx", b(-4.0, 16.5), 0.0, 1.0);
    s.scene.world.get_mut::<MeshRenderer>(pavel).unwrap().texture = "models/characters/man_tex.png".into();
    let monster = s.model("Counter", "models/characters/MonsterPSX.fbx", Vec3::new(0.0, -50.0, 0.0), 0.0, 1.15);
    s.scene.world.get_mut::<MeshRenderer>(monster).unwrap().texture = "models/characters/MonsterPSX_Diffuse.png".into();
    s.scripts(monster, vec![script("MonsterAi", json!({}))]);
    // Skinned and animated: the monster's own clips, procedural ones for Anna and Pavel.
    s.scene.world.insert(monster, Animator { clip: "Idle_Watchful".into(), fade: 0.3, ..Default::default() });
    s.scene.world.insert(anna, Animator { clip: "Wipe".into(), fade: 0.4, ..Default::default() });
    s.scripts(anna, vec![script("Npc", json!({ "speaker": "Anna", "work_clip": "Wipe", "notice": 2.6 }))]);
    s.scene.world.insert(pavel, Animator { clip: "Look".into(), fade: 0.4, ..Default::default() });
    // IK: the Counter and Pavel keep their feet on the real floors; heads turn to Lida (scripts
    // give the point). Anna stands on the staff step, which has no collider, so her feet stay.
    s.scene.world.insert(monster, CharacterIk { look_limit: 85.0, ..Default::default() });
    s.scene.world.insert(monster, Ragdoll { mass: 80.0, damping: 2.0, ..Default::default() });
    s.scene.world.insert(pavel, CharacterIk::default());
    s.scene.world.insert(anna, CharacterIk { feet: false, look_limit: 65.0, ..Default::default() });
    s.scripts(pavel, vec![script("Npc", json!({ "speaker": "Pavel", "work_clip": "Look", "notice": 4.0 }))]);
    let eye_glow = s.light("Counter glow", Vec3::new(0.0, -50.0, 0.0), [1.0, 0.25, 0.15, 1.0], 0.0, 3.5);
    s.scene.world.insert(eye_glow, tags("monster_light"));

    // ---------- things to use (named I_<id>; the game script knows what each does)
    let k = |s: &mut S, e: Entity, size: Vec3| s.usable(e, size);
    let e = s.prop("I_flashlight", "flashlight", b(3.4, 7.05) + Vec3::Y * 1.02, 70.0, 1.6);
    k(&mut s, e, Vec3::new(0.5, 0.3, 0.5));
    let e = s.prop("I_note_N01", "note", b(4.2, 7.0) + Vec3::Y * 1.02, 15.0, 1.4);
    k(&mut s, e, Vec3::new(0.45, 0.3, 0.45));
    let e = s.prop("I_note_N02", "note", b(4.2, 8.45) + Vec3::Y * 1.0, -20.0, 1.4);
    k(&mut s, e, Vec3::new(0.45, 0.3, 0.45));
    let e = s.model("I_anna", "", b(1.5, 7.72), 0.0, 1.0);
    s.scene.world.remove::<MeshRenderer>(e);
    k(&mut s, e, Vec3::new(1.0, 1.8, 1.0));
    for i in 0..3 {
        let e = s.model(&format!("I_plate_{i}"), "models/kitchen/dirty_plate.glb", b(1.4 + i as f32 * 0.42, 9.95) + Vec3::Y * 0.97, i as f32 * 25.0, 1.0);
        k(&mut s, e, Vec3::new(0.4, 0.2, 0.4));
    }
    let e = s.model("I_sink", "", b(1.9, 9.55), 0.0, 1.0);
    s.scene.world.remove::<MeshRenderer>(e);
    k(&mut s, e, Vec3::new(1.0, 1.1, 0.8));
    for (id, mesh, x) in [("I_meal_water", "glass_of_water", 5.95), ("I_meal_bread", "toast", 5.45), ("I_meal_sweet", "chocolate_bar_close", 4.95)] {
        let e = s.model(id, &format!("models/kitchen/{mesh}.glb"), b(x, 8.45) + Vec3::Y * 1.0, 0.0, 1.0);
        k(&mut s, e, Vec3::new(0.35, 0.3, 0.35));
    }
    // Table places: driver (road), teacher (line), girl (ribbon), and the fourth.
    for (id, x, y) in [("I_seat_driver", 7.85, 4.5), ("I_seat_teacher", 8.75, 4.5), ("I_seat_girl", 8.3, 4.95), ("I_seat_fourth", 8.3, 4.05)] {
        let e = s.model(id, "", b(x, y) + Vec3::Y * 0.78, 0.0, 1.0);
        s.scene.world.remove::<MeshRenderer>(e);
        k(&mut s, e, Vec3::new(0.42, 0.25, 0.42));
    }
    for (name, tex, x, y) in [("Token road", "decals/signs/route_17.png", 7.6, 4.5), ("Token line", "decals/signs/night_shift.png", 9.0, 4.5), ("Token ribbon", "decals/signs/lost_property.png", 8.3, 5.2)] {
        s.sign(name, tex, b(x, y) + Vec3::Y * 0.8, Vec3::ZERO, 0.22);
    }
    let e = s.model("I_fuse_box", "models/misc/cardboard_box_1.glb", b(6.45, 7.2) + Vec3::Y * 0.02, 10.0, 1.0);
    s.scene.world.get_mut::<MeshRenderer>(e).unwrap().texture = "models/textures/cardboard_box_1.png".into();
    k(&mut s, e, Vec3::new(0.6, 0.5, 0.6));
    let e = s.prop("I_panel", "fuse_box", b(6.88, 10.0) + Vec3::Y * 1.25, -90.0, 2.2);
    k(&mut s, e, Vec3::new(0.5, 0.8, 0.8));
    s.sign("Max load sign", "decals/signs/max_load_4.png", b(6.9, 10.6) + Vec3::Y * 1.75, Vec3::new(0.0, 0.0, 90.0), 0.45);
    let e = s.prop("I_keycard", "keycard_red", b(3.5, 8.45) + Vec3::Y * 1.02, 30.0, 1.8);
    k(&mut s, e, Vec3::new(0.4, 0.3, 0.4));
    let e = s.prop("I_reader", "card_reader", b(6.88, 8.32) + Vec3::Y * 1.25, -90.0, 2.0);
    k(&mut s, e, Vec3::new(0.4, 0.5, 0.5));
    let e = s.model("I_ledger", "models/misc/notebook_1.glb", b(9.4, 9.6) + Vec3::Y * 0.8, 10.0, 1.3);
    s.scene.world.get_mut::<MeshRenderer>(e).unwrap().texture = "models/textures/notebook_1.png".into();
    k(&mut s, e, Vec3::new(0.5, 0.3, 0.5));
    let e = s.model("I_vera_book", "models/misc/notebook_1.glb", b(9.95, 9.45) + Vec3::Y * 0.8, -25.0, 1.1);
    s.scene.world.get_mut::<MeshRenderer>(e).unwrap().color = [0.55, 0.65, 0.95, 1.0];
    k(&mut s, e, Vec3::new(0.45, 0.3, 0.45));
    s.solid("Office desk", Vec3::new(9.6, 0.38, -9.6), Vec3::new(1.4, 0.76, 0.7));
    // Storage: the lost-property cupboard with the doll behind glass.
    // Against the east wall, its glass front to the room (the north wall has the shelves).
    let cupboard = s.model("Lost property cupboard", "models/locker.glb", b(11.65, 12.0), -90.0, 1.0);
    s.scene.world.insert(cupboard, Collider { shape: ShapeKind::Hull, ..Default::default() });
    s.sign("Lost property sign", "decals/signs/lost_property.png", b(11.38, 12.0) + Vec3::Y * 1.75, Vec3::new(90.0, -90.0, 0.0), 0.45);
    let e = s.prop("I_doll", "doll_porcelain", b(11.45, 12.0) + Vec3::Y * 1.05, -90.0, 2.4);
    k(&mut s, e, Vec3::new(0.6, 0.6, 0.5));
    s.sign("Nina tag", "decals/signs/nina_sedova.png", b(11.38, 12.0) + Vec3::Y * 0.75, Vec3::new(90.0, -90.0, 0.0), 0.3);
    // Yard: workbench with the crowbar, the driver's locker.
    s.model("Yard bench", "models/workbench.glb", b(-3.6, 11.0), 90.0, 1.0);
    let e = s.prop("I_crowbar", "crowbar", b(-3.6, 11.0) + Vec3::Y * 0.92, 20.0, 1.6);
    k(&mut s, e, Vec3::new(0.7, 0.3, 0.5));
    s.model("Driver locker", "models/locker.glb", b(-2.0, 13.4), 0.0, 1.0);
    let e = s.model("I_yard_locker", "", b(-2.0, 13.0), 0.0, 1.0);
    s.scene.world.remove::<MeshRenderer>(e);
    k(&mut s, e, Vec3::new(0.9, 1.8, 0.6));
    let board = s.ent("Locker board", b(-2.0, 13.05) + Vec3::Y * 1.0, 0.0, Vec3::new(0.7, 0.12, 0.04));
    s.scene.world.insert(board, Shape::of(ShapeType::Cube));
    s.scene.world.insert(board, MeshRenderer { color: [0.4, 0.3, 0.2, 1.0], roughness: 0.9, shader: "materials/game/door_wood.kmat".into(), ..Default::default() });
    s.sign("Pavel tag", "decals/signs/pavel_zorin.png", b(-2.0, 13.05) + Vec3::Y * 1.5, Vec3::new(-90.0, 0.0, 0.0), 0.32);
    let e = s.prop("I_key", "key_skeleton", b(-2.0, 13.2) + Vec3::Y * 1.0, 0.0, 2.2);
    k(&mut s, e, Vec3::new(0.4, 0.3, 0.4));
    // Emergency exit with its padlock and sign.
    let e = s.prop("I_padlock", "padlock", b(0.12, 10.0) + Vec3::Y * 1.05, 90.0, 2.5);
    k(&mut s, e, Vec3::new(0.5, 0.6, 1.0));
    s.sign("Emergency sign", "decals/signs/emergency_exit.png", b(0.12, 10.0) + Vec3::Y * 2.4, Vec3::new(0.0, 0.0, -90.0), 0.6);
    // The fourth plate (appears for the ending) and the bus stop.
    let e = s.model("I_plate_fourth", "models/kitchen/clean_plate.glb", b(4.6, 7.0) + Vec3::Y * 1.02, 0.0, 1.0);
    k(&mut s, e, Vec3::new(0.4, 0.25, 0.4));
    s.model("Bus stop sign", "models/stop_sign.glb", Vec3::new(15.0, 0.0, 7.5), 180.0, 1.0);
    s.model("Bus shelter", "models/bus_shelter.glb", Vec3::new(17.5, 0.0, 8.0), 180.0, 1.0);
    let e = s.model("I_bus_stop", "", Vec3::new(15.5, 0.0, 7.8), 0.0, 1.0);
    s.scene.world.remove::<MeshRenderer>(e);
    k(&mut s, e, Vec3::new(2.5, 2.0, 2.0));

    // ---------- marks of the place: notices, damp, footprints to the lost-property cupboard
    s.sign("Staff only", "decals/signs/staff_only.png", b(6.5, 8.05) + Vec3::Y * 2.45, Vec3::new(90.0, 180.0, 0.0), 0.5);
    s.sign("Do not remove", "decals/signs/do_not_remove.png", b(8.3, 5.6) + Vec3::Y * 0.8, Vec3::ZERO, 0.25);
    s.sign("Night shift", "decals/signs/night_shift.png", b(11.9, 4.0) + Vec3::Y * 1.7, Vec3::new(0.0, 0.0, 90.0), 0.7);
    s.sign("Route 17", "decals/signs/route_17.png", b(0.1, 3.0) + Vec3::Y * 1.6, Vec3::new(0.0, 0.0, -90.0), 0.7);
    s.sign("No passengers", "decals/signs/no_passengers.png", b(11.9, 9.5) + Vec3::Y * 1.6, Vec3::new(0.0, 0.0, 90.0), 0.6);
    s.sign("Water supply", "decals/signs/water_supply.png", b(0.1, 9.2) + Vec3::Y * 1.7, Vec3::new(0.0, 0.0, -90.0), 0.5);
    for (i, (x, y, yaw)) in [(9.2f32, 11.3f32, 70.0f32), (10.0, 11.6, 80.0), (10.8, 11.9, 90.0)].iter().enumerate() {
        s.sign("Footprints", &format!("decals/damage/footprints_0{}.png", i % 8), b(*x, *y) + Vec3::Y * 0.02, Vec3::new(0.0, *yaw, 0.0), 0.8);
    }
    for (i, (x, y)) in [(2.0f32, 12.6f32), (0.4, 9.0), (11.5, 14.6), (4.0, 0.3)].iter().enumerate() {
        s.sign("Damp", &format!("decals/damage/damp_0{}.png", i), b(*x, *y) + Vec3::Y * 1.4, Vec3::new(90.0, 0.0, 0.0), 1.6);
    }
    for i in 0..5 {
        let e = s.scene.spawn("Puddle");
        s.scene.world.insert(e, Transform { position: Vec3::new(2.0 + i as f32 * 3.1, 0.02, 4.0 + (i % 2) as f32 * 2.5), rotation: Vec3::new(0.0, i as f32 * 40.0, 0.0), scale: Vec3::new(2.2, 0.5, 1.4) });
        s.scene.world.insert(e, Decal { seed: i + 2, ..kvas_engine::decal::preset("puddle") });
    }

    // ---------- weather and sound beds
    let rain = s.ent("Drizzle", Vec3::new(6.0, 9.0, 4.0), 0.0, Vec3::ONE);
    // Light, short streaks: a heavy curtain reads as lines drawn over the picture.
    s.scene.world.insert(rain, ParticleEmitter { floor_height: 0.02, rate: 320.0, stretch: 0.018, color_start: [0.8, 0.85, 0.95, 0.14], color_end: [0.8, 0.85, 0.95, 0.06], ..ParticleEmitter::preset("drizzle") });
    let mist = s.ent("Mist", Vec3::new(4.0, 0.2, 6.0), 0.0, Vec3::ONE);
    s.scene.world.insert(mist, ParticleEmitter { shape_size: 14.0, soft: 1.2, ..ParticleEmitter::preset("mist") });
    s.sound("Room hum", "audio/sfx/amb_room_hum.wav", Vec3::new(6.0, 2.0, -6.0), 0.35, 14.0);
    s.sound("Forest wind", "audio/sfx/amb_forest_wind.wav", Vec3::new(-6.0, 3.0, -16.0), 0.5, 30.0);
    s.sound("Road wind", "audio/sfx/amb_forest_wind.wav", Vec3::new(8.0, 3.0, 10.0), 0.35, 25.0);

    // Every lamp gets the FlickerLight script: its own intensity is the base, the director feeds
    // `power` from the switch panel. The Counter's glow is driven by the Counter itself.
    let lamps: Vec<(Entity, f32, bool)> = s
        .scene
        .world
        .iter::<Light>()
        .filter(|(_, l)| l.kind == LightKind::Point)
        .map(|(e, l)| (e, l.intensity, s.scene.world.get::<Tags>(e).is_some_and(|t| t.value.contains("flicker"))))
        .filter(|(e, _, _)| hierarchy::name_of(&s.scene.world, *e) != "Counter glow")
        .collect();
    for (e, intensity, flicker) in lamps {
        let (base, power) = if intensity > 0.0 { (intensity, 1.0) } else { (2.2, 0.0) };
        s.scripts(e, vec![script("FlickerLight", json!({ "base": base, "power": power, "flicker": flicker }))]);
    }

    hierarchy::update_globals(&mut s.scene.world);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/last_service/assets/scenes/main.scene.json");
    std::fs::write(&path, s.scene.to_json(&Registry::default())).expect("write scene");
    println!("wrote {}", path.display());
}
