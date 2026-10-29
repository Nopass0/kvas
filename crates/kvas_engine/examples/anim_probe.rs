//! Prints where the animated characters of a scene end up (bounds of their skinned meshes).
//! `cargo run -p kvas_engine --example anim_probe -- <project> <scene>`
use kvas_engine::{assets::Assets, components::*, hierarchy, registry::Registry, scene::Scene};

fn main() {
    let mut args = std::env::args().skip(1);
    let root = std::path::PathBuf::from(args.next().expect("project"));
    let scene_path = args.next().expect("scene");
    let assets = Assets::Dir(root.join("assets"));
    kvas_engine::meshmod::set_assets(assets.clone());
    let (mut scene, _) = Scene::from_json(&Registry::default(), &assets.read_string(&scene_path).unwrap()).unwrap();
    for step in 0..10 {
        kvas_engine::skeletal::update(&mut scene.world, step as f32 / 30.0, 1.0 / 30.0, true);
    }
    for (e, dm) in scene.world.iter::<DynamicMesh>() {
        let (lo, hi) = dm.data.bounds();
        let anim = scene.world.get::<Animator>(e).map(|a| (a.clip.clone(), a.clips.len()));
        println!("{}: bounds {lo:?}..{hi:?} vertices {} anim {anim:?}", hierarchy::name_of(&scene.world, e), dm.data.vertices.len());
    }
    for (e, a) in scene.world.iter::<Animator>() {
        if !scene.world.has::<DynamicMesh>(e) {
            println!("{}: animator without skinned mesh (clip {})", hierarchy::name_of(&scene.world, e), a.clip);
        }
    }
}
