//! Prints the skeleton, skin and animation clips of FBX files (a quick look before using them).
//! `cargo run -p kvas_engine --example fbx_skeleton -- file.fbx [more.fbx]`
fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("read");
        let scene = ufbx::load_memory(&bytes, ufbx::LoadOpts::default()).expect("parse");
        println!("== {path}");
        println!("nodes {} meshes {} bones {} skins {} clips {}", scene.nodes.len(), scene.meshes.len(), scene.bones.len(), scene.skin_deformers.len(), scene.anim_stacks.len());
        for stack in scene.anim_stacks.iter() {
            println!("  clip '{}' {:.2}..{:.2}s", stack.element.name, stack.time_begin, stack.time_end);
        }
        for skin in scene.skin_deformers.iter() {
            println!("  skin: {} clusters, {} vertices", skin.clusters.len(), skin.vertices.len());
            let names: Vec<String> = skin.clusters.iter().map(|c| c.bone_node.as_ref().map_or(String::from("?"), |n| n.element.name.to_string())).collect();
            println!("  bones: {}", names.join(", "));
        }
        for mesh in scene.meshes.iter() {
            println!("  mesh '{}' {} vertices, {} faces", mesh.element.name, mesh.num_vertices, mesh.num_faces);
        }
    }
}
