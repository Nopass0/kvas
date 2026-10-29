//! Embeds the files of `library/` (starter scripts and shaders) into the editor.
use std::path::Path;

fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            walk(&p, root, out);
        } else if p.extension().is_some_and(|x| x == "rs" || x == "wgsl" || x == "py" || x == "json" || x == "kmat") {
            let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
            out.push((rel, p.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/")));
        }
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("library");
    println!("cargo:rerun-if-changed=library");
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    let mut src = String::from("pub const FILES: &[(&str, &str)] = &[\n");
    for (rel, abs) in files {
        src.push_str(&format!("    ({rel:?}, include_str!({abs:?})),\n"));
    }
    src.push_str("];\n");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("library_files.rs");
    std::fs::write(out, src).unwrap();

    // Engine sources for the in-editor API reference (rustdoc comments of every public item).
    let engine = Path::new(env!("CARGO_MANIFEST_DIR")).join("../kvas_engine/src");
    println!("cargo:rerun-if-changed=../kvas_engine/src");
    let mut sources = Vec::new();
    walk(&engine, &engine, &mut sources);
    let mut api = String::from("pub const ENGINE_SOURCES: &[(&str, &str)] = &[\n");
    for (rel, abs) in sources.iter().filter(|(rel, _)| rel.ends_with(".rs")) {
        api.push_str(&format!("    ({rel:?}, include_str!({abs:?})),\n"));
    }
    api.push_str("];\n");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("engine_sources.rs");
    std::fs::write(out, api).unwrap();
}
