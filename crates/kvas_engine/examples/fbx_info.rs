use kvas_engine::{assets::Assets, mesh::MeshData};
fn main() {
    let root = std::env::args().nth(1).unwrap();
    let assets = Assets::Dir(std::path::PathBuf::from(&root));
    for name in std::env::args().skip(2) {
        let m = MeshData::load(&assets, &name).unwrap();
        let (lo, hi) = m.bounds();
        println!("{name}: {lo:?} {hi:?} parts {} textured {:?}", m.parts.len(), m.parts.iter().map(|p| p.albedo.is_some()).collect::<Vec<_>>());
    }
}
