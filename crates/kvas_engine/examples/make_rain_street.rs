//! Writes `games/demo/assets/scenes/rain_night.scene.json`: a wet street at night in heavy rain —
//! sodium street lamps, a neon sign, shop windows, puddles with screen-space reflections and drop
//! ripples, rain streaks with splashes, a rainy window pane, mist, and camera shaders.
//! `cargo run -p kvas_engine --example make_rain_street`
use glam::Vec3;
use kvas_engine::decal::{preset, Decal, DecalBlend, DecalShape};
use kvas_engine::fx::PostEffect;
use kvas_engine::prelude::*;
use kvas_engine::registry::Registry;
use kvas_engine::scene::Scene;

fn block(scene: &mut Scene, name: &str, pos: Vec3, size: Vec3, color: [f32; 4], rough: f32, shader: &str) -> Entity {
    let e = scene.spawn(name);
    scene.world.insert(e, Transform { position: pos, scale: size, ..Default::default() });
    scene.world.insert(e, Shape::of(ShapeType::Cube));
    scene.world.insert(e, MeshRenderer { color, roughness: rough, reflectivity: if rough < 0.3 { 0.8 } else { 0.1 }, shader: shader.into(), ..Default::default() });
    e
}

fn light(scene: &mut Scene, name: &str, pos: Vec3, color: [f32; 4], intensity: f32, range: f32) {
    let e = scene.spawn(name);
    scene.world.insert(e, Transform::at(pos));
    scene.world.insert(e, Light { kind: LightKind::Point, color, intensity, range, ..Default::default() });
}

fn main() {
    let mut scene = Scene::default();
    let s = &mut scene.settings;
    s.atmosphere.procedural = true;
    s.atmosphere.time = 23.3;
    s.atmosphere.clouds = 0.95;
    s.atmosphere.stars = 0.0;
    s.ambient = [0.05, 0.06, 0.08];
    s.fog = 0.03;
    s.post.bloom = 0.55;
    s.post.bloom_threshold = 0.65;
    s.post.vignette = 0.35;
    s.post.film_grain = 0.08;

    let moon = scene.spawn("Moonlight");
    scene.world.insert(moon, Transform { rotation: Vec3::new(-35.0, 140.0, 0.0), ..Default::default() });
    scene.world.insert(moon, Light { intensity: 0.12, color: [0.55, 0.65, 1.0, 1.0], ..Default::default() });

    // Street, kerbs, pavements and two rows of buildings.
    block(&mut scene, "Road", Vec3::new(0.0, -0.05, 0.0), Vec3::new(8.0, 0.1, 80.0), [0.05, 0.05, 0.055, 1.0], 0.18, "");
    for side in [-1.0f32, 1.0] {
        block(&mut scene, "Pavement", Vec3::new(side * 6.0, 0.05, 0.0), Vec3::new(4.0, 0.3, 80.0), [0.16, 0.16, 0.17, 1.0], 0.35, "");
        block(&mut scene, "Kerb", Vec3::new(side * 4.05, 0.08, 0.0), Vec3::new(0.15, 0.36, 80.0), [0.35, 0.35, 0.36, 1.0], 0.6, "");
        for i in 0..8 {
            let z = -35.0 + i as f32 * 10.0;
            let h = 9.0 + ((i * 7 + if side > 0.0 { 3 } else { 0 }) % 5) as f32 * 3.0;
            let shade = 0.1 + ((i * 3) % 4) as f32 * 0.03;
            block(&mut scene, "Building", Vec3::new(side * 12.0, h * 0.5, z), Vec3::new(8.0, h, 9.6), [shade, shade * 0.95, shade * 0.9, 1.0], 0.85, "");
            // Lit windows.
            for fl in 0..(h as i32 / 3) {
                for w in 0..3 {
                    let lit = (i * 13 + fl as usize * 7 + w * 5) % 4 != 0;
                    let warm = (i + w) % 3 == 0;
                    let c = if !lit { [0.02, 0.02, 0.03, 1.0] } else if warm { [1.0, 0.72, 0.4, 1.0] } else { [0.6, 0.75, 1.0, 1.0] };
                    let e = block(&mut scene, "Window", Vec3::new(side * 7.98, 1.8 + fl as f32 * 3.0, z - 3.0 + w as f32 * 3.0), Vec3::new(0.05, 1.4, 1.6), c, 0.1, "");
                    if lit {
                        scene.world.get_mut::<MeshRenderer>(e).unwrap().emissive = 1.6;
                    }
                }
            }
        }
    }
    // Street lamps: pole, head and a warm sodium light.
    for i in 0..5 {
        let z = -28.0 + i as f32 * 14.0;
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        block(&mut scene, "Lamp pole", Vec3::new(side * 4.6, 2.5, z), Vec3::new(0.12, 5.0, 0.12), [0.1, 0.1, 0.11, 1.0], 0.5, "");
        let head = block(&mut scene, "Lamp head", Vec3::new(side * 3.9, 5.0, z), Vec3::new(1.4, 0.15, 0.4), [1.0, 0.65, 0.3, 1.0], 0.4, "");
        scene.world.get_mut::<MeshRenderer>(head).unwrap().emissive = 3.0;
        light(&mut scene, "Street light", Vec3::new(side * 3.6, 4.7, z), [1.0, 0.62, 0.3, 1.0], 9.0, 16.0);
    }
    // Shop with a neon sign and a rainy window.
    block(&mut scene, "Shop front", Vec3::new(-7.95, 1.6, 2.0), Vec3::new(0.1, 3.2, 6.0), [0.08, 0.08, 0.09, 1.0], 0.4, "");
    let pane = block(&mut scene, "Shop window", Vec3::new(-7.85, 1.5, 2.0), Vec3::new(0.04, 2.2, 4.6), [0.75, 0.85, 1.0, 1.0], 0.05, "shaders/library/glass_rain.wgsl");
    scene.world.get_mut::<MeshRenderer>(pane).unwrap().cast_shadows = false;
    block(&mut scene, "Shop inside", Vec3::new(-9.5, 1.5, 2.0), Vec3::new(3.0, 3.0, 5.6), [0.9, 0.55, 0.3, 1.0], 0.8, "");
    let neon = scene.spawn("Neon sign");
    scene.world.insert(neon, Transform { position: Vec3::new(-7.7, 4.0, 2.0), rotation: Vec3::new(0.0, 0.0, 90.0), scale: Vec3::new(2.0, 1.0, 2.0) });
    scene.world.insert(neon, Decal { shape: DecalShape::Ring, color: [1.0, 0.15, 0.55, 1.0], emission: 5.0, angle_fade: 0.0, ..Default::default() });
    light(&mut scene, "Neon glow", Vec3::new(-6.8, 4.0, 2.0), [1.0, 0.2, 0.6, 1.0], 4.0, 9.0);
    light(&mut scene, "Shop light", Vec3::new(-7.0, 1.8, 2.0), [1.0, 0.7, 0.4, 1.0], 3.0, 7.0);

    // Puddles along the road and pavements, wet streaks on walls.
    for (i, (x, z, s)) in [(-1.5f32, -6.0f32, 3.5f32), (2.2, 1.0, 2.6), (-2.6, 7.0, 4.0), (1.0, 12.0, 3.0), (-5.5, 4.0, 2.0), (6.2, -3.0, 2.4), (0.0, -14.0, 5.0), (2.5, -20.0, 3.0)].iter().enumerate() {
        let e = scene.spawn("Puddle");
        scene.world.insert(e, Transform { position: Vec3::new(*x, 0.05, *z), rotation: Vec3::new(0.0, i as f32 * 47.0, 0.0), scale: Vec3::new(*s, 0.6, *s * 0.7) });
        scene.world.insert(e, Decal { seed: i as i32 + 3, ..preset("puddle") });
    }
    for i in 0..6 {
        let e = scene.spawn("Wall leaks");
        scene.world.insert(e, Transform { position: Vec3::new(-8.0, 6.0, -30.0 + i as f32 * 11.0), rotation: Vec3::new(0.0, 0.0, -90.0), scale: Vec3::new(5.0, 1.0, 6.0) });
        scene.world.insert(e, Decal { blend: DecalBlend::Multiply, seed: i, ..preset("leaks") });
    }

    // Rain with streaks and splashes around the camera, and low mist.
    let rain = scene.spawn("Rain");
    scene.world.insert(rain, Transform::at(Vec3::new(0.0, 9.0, 0.0)));
    scene.world.insert(rain, ParticleEmitter { floor_height: 0.02, soft: 0.2, ..ParticleEmitter::preset("heavy_rain") });
    let mist = scene.spawn("Mist");
    scene.world.insert(mist, Transform::at(Vec3::new(0.0, 0.3, 0.0)));
    scene.world.insert(mist, ParticleEmitter { shape_size: 16.0, soft: 1.5, ..ParticleEmitter::preset("mist") });

    // Camera on the pavement, with night grade and drops on the lens.
    let cam = scene.spawn("Camera");
    let mut t = Transform::at(Vec3::new(2.5, 1.65, 14.0));
    t.look_at(Vec3::new(-1.5, 2.2, -4.0), Vec3::Y);
    scene.world.insert(cam, t);
    scene.world.insert(
        cam,
        Camera {
            fov: 55.0,
            far: 300.0,
            effects: vec![
                PostEffect { shader: "shaders/library/fx/cinematic/sodium_night.wgsl".into(), enabled: true, params: [0.8, 0.0, 0.0, 0.0] },
                PostEffect { shader: "shaders/library/fx/cinematic/rain_on_lens.wgsl".into(), enabled: true, params: [0.55, 1.0, 1.0, 0.0] },
            ],
            ..Default::default()
        },
    );
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/demo/assets/scenes/rain_night.scene.json");
    std::fs::write(&path, scene.to_json(&Registry::default())).expect("write scene");
    println!("wrote {}", path.display());
}
