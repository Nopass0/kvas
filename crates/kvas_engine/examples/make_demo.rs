//! Generates the demo project: `cargo run -p kvas_engine --example make_demo -- games/demo`
//! It doubles as an example of building scenes through the engine API.

use glam::Vec3;
use kvas_engine::prelude::*;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;
use std::path::Path;

fn script(name: &str, props: serde_json::Value) -> Scripts {
    Scripts(vec![ScriptSlot { name: name.into(), props: props.as_object().cloned().unwrap_or_default() }])
}

fn mesh(scene: &mut Scene, name: &str, mesh: &str, pos: Vec3, scale: Vec3, color: [f32; 4]) -> Entity {
    let e = hierarchy::spawn_named(&mut scene.world, name);
    scene.world.insert(e, Transform { position: pos, scale, ..Default::default() });
    scene.world.insert(e, MeshRenderer { mesh: format!("primitive:{mesh}"), color, ..Default::default() });
    e
}

fn body(scene: &mut Scene, e: Entity, shape: ShapeKind, mass: f32) {
    scene.world.insert(e, RigidBody { mass, ..Default::default() });
    scene.world.insert(e, Collider { shape, size: if shape == ShapeKind::Capsule { Vec3::new(1.0, 2.0, 1.0) } else { Vec3::ONE }, ..Default::default() });
}

fn torus_obj(major: f32, minor: f32) -> String {
    let (nu, nv) = (36, 16);
    let mut s = String::from("# torus\n");
    let mut normals = String::new();
    for i in 0..nu {
        for j in 0..nv {
            let (u, v) = (i as f32 / nu as f32 * std::f32::consts::TAU, j as f32 / nv as f32 * std::f32::consts::TAU);
            let (cx, cy, cz) = (u.cos(), 0.0, u.sin());
            let r = major + minor * v.cos();
            s += &format!("v {} {} {}\n", r * cx, minor * v.sin() + cy, r * cz);
            normals += &format!("vn {} {} {}\n", v.cos() * cx, v.sin(), v.cos() * cz);
        }
    }
    s += &normals;
    for i in 0..nu {
        for j in 0..nv {
            let idx = |a: usize, b: usize| (a % nu) * nv + (b % nv) + 1;
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1));
            s += &format!("f {a}//{a} {d}//{d} {c}//{c} {b}//{b}\n");
        }
    }
    s
}

fn checker_png(path: &Path) {
    let img = image::RgbaImage::from_fn(256, 256, |x, y| {
        let on = ((x / 32) + (y / 32)) % 2 == 0;
        let v = if on { 205 } else { 150 };
        image::Rgba([v, v, v + 10, 255])
    });
    img.save(path).expect("write checker.png");
}

const WAVE_SHADER: &str = "// Custom shader: the engine prelude (globals, object, lighting helpers) is prepended automatically.
// Required entry points: vs_main and fs_main.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var p = v.position;
    let t = globals.time.x;
    p.y += sin(p.x * 14.0 + t * 2.0) * 0.03 + cos(p.z * 11.0 + t * 1.5) * 0.03;
    var out: VsOut;
    let world = object.model * vec4<f32>(p, 1.0);
    out.clip = globals.view_proj * world;
    out.world_pos = world.xyz;
    out.normal = vec3<f32>(0.0, 1.0, 0.0);
    out.uv = v.uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(vec3<f32>(sin(in.uv.x * 40.0 + globals.time.x * 2.0) * 0.25, 1.0, cos(in.uv.y * 33.0 + globals.time.x * 1.5) * 0.25));
    let rgb = lighting(in.world_pos, n, object.color.rgb, 0.15);
    return vec4<f32>(to_display(rgb), 0.85);
}
";

const RIM_SHADER: &str = "// Rim-glow shader that reuses the engine's standard vertex stage and lighting.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let view_dir = normalize(globals.camera_pos.xyz - in.world_pos);
    let rim = pow(1.0 - max(dot(n, view_dir), 0.0), 3.0);
    let pulse = 0.6 + 0.4 * sin(globals.time.x * 3.0);
    let lit = lighting(in.world_pos, n, object.color.rgb, 0.3);
    return vec4<f32>(to_display(lit + rim * pulse * vec3<f32>(0.1, 0.7, 1.0)), 1.0);
}
";

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| "games/demo".into());
    let root = Path::new(&root);
    for dir in ["assets/scenes", "assets/models", "assets/textures", "assets/shaders"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    checker_png(&root.join("assets/textures/checker.png"));
    std::fs::write(root.join("assets/models/torus.obj"), torus_obj(0.8, 0.3)).unwrap();
    std::fs::write(root.join("assets/shaders/wave.wgsl"), WAVE_SHADER).unwrap();
    std::fs::write(root.join("assets/shaders/rim.wgsl"), RIM_SHADER).unwrap();

    let mut scene = Scene::default();

    let sun = hierarchy::spawn_named(&mut scene.world, "Sun");
    scene.world.insert(sun, Transform { position: Vec3::new(0.0, 12.0, 0.0), rotation: Vec3::new(-50.0, -30.0, 0.0), ..Default::default() });
    scene.world.insert(sun, Light { intensity: 1.15, ..Default::default() });

    let cam = hierarchy::spawn_named(&mut scene.world, "Main Camera");
    scene.world.insert(cam, Transform { position: Vec3::new(0.0, 7.0, 15.0), rotation: Vec3::new(-20.0, 0.0, 0.0), ..Default::default() });
    scene.world.insert(cam, Camera::default());
    scene.world.insert(cam, script("FollowCamera", serde_json::json!({ "target": "Player", "offset": [0.0, 5.0, 9.0], "smoothing": 4.0 })));

    let ground = mesh(&mut scene, "Ground", "cube", Vec3::new(0.0, -0.5, 0.0), Vec3::new(80.0, 1.0, 80.0), [0.85, 0.9, 0.85, 1.0]);
    if let Some(m) = scene.world.get_mut::<MeshRenderer>(ground) {
        m.texture = "textures/checker.png".into();
        m.tiling = 40.0;
        m.roughness = 0.9;
    }
    scene.world.insert(ground, Collider::default());

    let player = mesh(&mut scene, "Player", "sphere", Vec3::new(0.0, 1.0, 6.0), Vec3::ONE, [1.0, 0.55, 0.15, 1.0]);
    body(&mut scene, player, ShapeKind::Sphere, 1.0);
    if let Some(rb) = scene.world.get_mut::<RigidBody>(player) {
        rb.linear_damping = 0.4;
        rb.angular_damping = 0.4;
    }
    if let Some(m) = scene.world.get_mut::<MeshRenderer>(player) {
        m.roughness = 0.25;
    }
    scene.world.insert(player, script("PlayerBall", serde_json::json!({ "force": 22.0, "jump": 6.5, "camera": "Main Camera" })));

    // Pyramid of boxes.
    let pyramid = hierarchy::spawn_named(&mut scene.world, "Pyramid");
    let rows = 5;
    for row in 0..rows {
        for i in 0..(rows - row) {
            let x = (i as f32 - (rows - row - 1) as f32 * 0.5) * 1.05 - 6.0;
            let c = [0.3 + 0.14 * row as f32, 0.55, 0.9 - 0.14 * row as f32, 1.0];
            let b = mesh(&mut scene, &format!("Box {row}-{i}"), "cube", Vec3::new(x, 0.5 + row as f32 * 1.0, -4.0), Vec3::ONE, c);
            body(&mut scene, b, ShapeKind::Box, 1.0);
            hierarchy::set_parent(&mut scene.world, b, Some(pyramid));
        }
    }

    // Pendulum: ball joint to the world, plus a visual rod parented to the weight.
    let weight = mesh(&mut scene, "Pendulum Weight", "sphere", Vec3::new(12.0, 8.0, 0.0), Vec3::splat(1.4), [0.75, 0.75, 0.8, 1.0]);
    body(&mut scene, weight, ShapeKind::Sphere, 6.0);
    scene.world.insert(weight, Joint { kind: JointKind::Ball, anchor: Vec3::new(-2.0, 0.0, 0.0), ..Default::default() });
    let rod = mesh(&mut scene, "Rod", "cylinder", Vec3::ZERO, Vec3::new(0.05, 2.0, 0.05), [0.3, 0.3, 0.3, 1.0]);
    hierarchy::set_parent(&mut scene.world, rod, Some(weight));
    scene.world.insert(rod, Transform { position: Vec3::new(-1.0, 0.0, 0.0), rotation: Vec3::new(0.0, 0.0, 90.0), scale: Vec3::new(0.05, 2.0, 0.05) });

    // Chain of linked spheres hanging from a world anchor.
    let mut previous: Option<String> = None;
    for i in 0..6 {
        let name = format!("Chain {i}");
        let link = mesh(&mut scene, &name, "sphere", Vec3::new(-7.2 + i as f32 * 0.8, 9.0, 2.0), Vec3::splat(0.6), [0.9, 0.8, 0.3, 1.0]);
        body(&mut scene, link, ShapeKind::Sphere, 0.5);
        scene.world.insert(link, Joint { kind: JointKind::Ball, target: previous.clone().unwrap_or_default(), anchor: Vec3::new(-0.8, 0.0, 0.0), ..Default::default() });
        previous = Some(name);
    }

    // Hinged door.
    let door = mesh(&mut scene, "Door", "cube", Vec3::new(3.0, 1.5, -9.0), Vec3::new(2.0, 3.0, 0.2), [0.6, 0.35, 0.2, 1.0]);
    body(&mut scene, door, ShapeKind::Box, 4.0);
    scene.world.insert(door, Joint { kind: JointKind::Hinge, axis: Vec3::Y, anchor: Vec3::new(-0.5, 0.0, 0.0), use_limits: true, limit_min: -100.0, limit_max: 100.0, ..Default::default() });
    let post = mesh(&mut scene, "Door Post", "cylinder", Vec3::new(2.0, 1.6, -9.0), Vec3::new(0.25, 3.2, 0.25), [0.25, 0.25, 0.28, 1.0]);
    scene.world.insert(post, Collider { shape: ShapeKind::Cylinder, ..Default::default() });

    // Spring box.
    let spring = mesh(&mut scene, "Spring Box", "cube", Vec3::new(-6.0, 3.0, 6.0), Vec3::splat(1.2), [0.9, 0.3, 0.4, 1.0]);
    body(&mut scene, spring, ShapeKind::Box, 1.0);
    scene.world.insert(spring, Joint { kind: JointKind::Spring, anchor: Vec3::new(0.0, 3.0, 0.0), rest_length: 2.0, stiffness: 80.0, damping: 3.0, ..Default::default() });

    // Moving platform (kinematic) with a crate on top.
    let platform = mesh(&mut scene, "Platform", "cube", Vec3::new(8.0, 0.4, 7.0), Vec3::new(4.0, 0.4, 3.0), [0.35, 0.4, 0.5, 1.0]);
    scene.world.insert(platform, RigidBody { kind: BodyKind::Kinematic, ..Default::default() });
    scene.world.insert(platform, Collider::default());
    scene.world.insert(platform, script("Mover", serde_json::json!({ "axis": [1.0, 0.0, 0.0], "amplitude": 4.0, "speed": 0.8 })));
    let crate_box = mesh(&mut scene, "Crate", "cube", Vec3::new(8.0, 1.3, 7.0), Vec3::splat(0.9), [0.8, 0.6, 0.3, 1.0]);
    body(&mut scene, crate_box, ShapeKind::Box, 1.0);

    // Compound body: a barbell built from three colliders in a hierarchy.
    let bar = mesh(&mut scene, "Barbell", "cylinder", Vec3::new(-12.0, 4.0, -4.0), Vec3::new(0.2, 3.0, 0.2), [0.5, 0.5, 0.55, 1.0]);
    scene.world.insert(bar, Transform { position: Vec3::new(-12.0, 4.0, -4.0), rotation: Vec3::new(0.0, 0.0, 90.0), scale: Vec3::new(0.2, 3.0, 0.2) });
    body(&mut scene, bar, ShapeKind::Cylinder, 3.0);
    for (i, x) in [-1.5f32, 1.5].iter().enumerate() {
        let plate = mesh(&mut scene, &format!("Plate {i}"), "sphere", Vec3::ZERO, Vec3::splat(1.0), [0.2, 0.2, 0.25, 1.0]);
        hierarchy::set_parent(&mut scene.world, plate, Some(bar));
        // Local space of the bar: its Y axis runs along the (rotated) length.
        scene.world.insert(plate, Transform { position: Vec3::new(0.0, *x, 0.0), scale: Vec3::new(5.0, 0.33, 5.0), ..Default::default() });
        scene.world.insert(plate, Collider { shape: ShapeKind::Sphere, size: Vec3::new(1.0, 1.0, 1.0), ..Default::default() });
    }

    // Shader showcases and imported model.
    let wave = mesh(&mut scene, "Water", "plane", Vec3::new(18.0, 0.08, -8.0), Vec3::new(10.0, 1.0, 10.0), [0.1, 0.45, 0.75, 1.0]);
    scene.world.get_mut::<MeshRenderer>(wave).unwrap().shader = "shaders/wave.wgsl".into();
    let gem = mesh(&mut scene, "Torus", "torus", Vec3::new(6.0, 2.5, 4.0), Vec3::ONE, [0.8, 0.9, 1.0, 1.0]);
    {
        let m = scene.world.get_mut::<MeshRenderer>(gem).unwrap();
        m.mesh = "models/torus.obj".into();
        m.shader = "shaders/rim.wgsl".into();
    }
    scene.world.insert(gem, script("Spinner", serde_json::json!({ "speed": [30.0, 60.0, 0.0] })));

    // Procedural marble texture (generated from noise at load time) on a floating sphere.
    let marble = mesh(&mut scene, "Marble", "sphere", Vec3::new(-2.0, 2.2, 3.0), Vec3::splat(1.6), [1.0, 1.0, 1.0, 1.0]);
    {
        let tex = NoiseTexture::new(Noise::new(NoiseKind::Ridged).with_frequency(3.0).with_octaves(5).with_warp(0.8)).colors([0.08, 0.1, 0.2], [0.95, 0.9, 0.85]);
        let m = scene.world.get_mut::<MeshRenderer>(marble).unwrap();
        m.texture = tex.key();
        m.roughness = 0.2;
    }
    scene.world.insert(marble, Bob { amplitude: 0.35, speed: 0.4, ..Default::default() });
    scene.world.insert(marble, Spin { speed: Vec3::new(0.0, 20.0, 0.0) });
    let jelly = mesh(&mut scene, "Jelly", "cone", Vec3::new(2.5, 2.0, 3.0), Vec3::splat(1.2), [0.9, 0.3, 0.6, 1.0]);
    scene.world.insert(jelly, ColorCycle::default());
    scene.world.insert(jelly, Wobble::default());
    scene.world.insert(jelly, Pulse::default());

    // Glass and chrome: reflections of the procedural sky.
    let glass = mesh(&mut scene, "Glass Sphere", "sphere", Vec3::new(-4.5, 1.0, 6.5), Vec3::splat(2.0), [0.78, 0.92, 1.0, 0.16]);
    {
        let m = scene.world.get_mut::<MeshRenderer>(glass).unwrap();
        (m.roughness, m.reflectivity, m.cast_shadows) = (0.03, 1.0, false);
    }
    let chrome = mesh(&mut scene, "Chrome Sphere", "sphere", Vec3::new(-1.5, 1.0, 9.0), Vec3::splat(2.0), [0.92, 0.92, 0.96, 1.0]);
    {
        let m = scene.world.get_mut::<MeshRenderer>(chrome).unwrap();
        (m.roughness, m.metallic, m.reflectivity) = (0.08, 1.0, 1.0);
    }
    scene.settings.atmosphere = kvas_engine::sky::SkySettings { procedural: true, time: 16.5, speed: 0.0, clouds: 0.5, ..Default::default() };

    // A spotlight on the pyramid.
    let spot = hierarchy::spawn_named(&mut scene.world, "Spotlight");
    scene.world.insert(spot, Transform { position: Vec3::new(-6.0, 7.0, 0.0), rotation: Vec3::new(-72.0, 0.0, 0.0), ..Default::default() });
    scene.world.insert(spot, Light { kind: LightKind::Spot, color: [1.0, 0.85, 0.6, 1.0], intensity: 9.0, range: 22.0, angle: 28.0, softness: 0.45, shadows: false });

    // Lights.
    let lamp = hierarchy::spawn_named(&mut scene.world, "Lamp");
    scene.world.insert(lamp, Transform::at(Vec3::new(-4.0, 3.0, 2.0)));
    scene.world.insert(lamp, Light { kind: LightKind::Point, color: [1.0, 0.6, 0.25, 1.0], intensity: 2.5, range: 12.0, ..Default::default() });
    scene.world.insert(lamp, script("Flicker", serde_json::json!({ "amount": 0.35 })));
    let lamp_marker = mesh(&mut scene, "Lamp Bulb", "sphere", Vec3::ZERO, Vec3::splat(0.25), [1.0, 0.8, 0.4, 1.0]);
    scene.world.get_mut::<MeshRenderer>(lamp_marker).unwrap().emissive = 2.0;
    hierarchy::set_parent(&mut scene.world, lamp_marker, Some(lamp));

    let spawner = hierarchy::spawn_named(&mut scene.world, "Spawner (press E)");
    scene.world.insert(spawner, Transform::at(Vec3::new(0.0, 10.0, 0.0)));
    scene.world.insert(spawner, script("BoxSpawner", serde_json::json!({})));

    let reg = Registry::default();
    std::fs::write(root.join("assets/scenes/main.scene.json"), scene.to_json(&reg)).unwrap();
    Project { root: root.to_path_buf(), config: ProjectConfig { name: "Kvas Demo".into(), start_scene: "scenes/main.scene.json".into(), scripts_package: "demo_scripts".into(), ..Default::default() } }
        .save()
        .unwrap();
    println!("demo project written to {}", root.display());
}
