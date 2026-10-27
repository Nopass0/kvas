//! Prints bounds, triangle counts and material info of models: `inspect_models <dir> <name>...`
use kvas_engine::assets::Assets;
use kvas_engine::mesh::MeshData;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("directory");
    let assets = Assets::Dir(dir.into());
    for name in args {
        match MeshData::load(&assets, &name) {
            Ok(m) => {
                let (min, max) = m.bounds();
                println!("{name}: size {:.2} x {:.2} x {:.2}, min {:.2},{:.2},{:.2}, tris {}, albedo {:?}, color {:?}", max.x - min.x, max.y - min.y, max.z - min.z, min.x, min.y, min.z, m.indices.len() / 3, m.albedo.as_ref().map(|a| (a.width, a.height)), m.base_color);
            }
            Err(e) => println!("{name}: {e}"),
        }
    }
}
