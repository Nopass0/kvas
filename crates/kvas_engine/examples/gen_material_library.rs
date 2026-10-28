//! Writes the built-in materials as `.kmat` files for the editor's starter library.
//! `cargo run -p kvas_engine --example gen_material_library`
use kvas_engine::material_presets::library;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../kvas_editor/library/materials");
    let _ = std::fs::remove_dir_all(&root);
    let all = library();
    for p in &all {
        let dir = root.join(p.category);
        std::fs::create_dir_all(&dir).expect("create folder");
        std::fs::write(dir.join(format!("{}.kmat", p.name)), p.graph.to_json()).expect("write material");
    }
    println!("{} materials -> {}", all.len(), root.display());
    // The demo's lab marble is the node-built marble too (open it to see every step).
    let lab = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/demo/assets/materials/lab/marble.kmat");
    let marble = kvas_engine::material_presets_v4::marble([0.9, 0.9, 0.88], [0.28, 0.3, 0.34], 3.0, 1.6);
    std::fs::write(&lab, marble.to_json()).expect("write lab marble");
}
