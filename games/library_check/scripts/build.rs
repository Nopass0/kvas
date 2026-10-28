//! Generates a module for every starter script of the editor library (hand-written and generated),
//! so `cargo build` compiles all of them and they stay valid.
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = read.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../../crates/kvas_editor/library/scripts");
    let root = root.canonicalize().unwrap();
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    collect(&root, &mut files);
    let mut src = String::new();
    for file in files {
        let rel = file.strip_prefix(&root).unwrap().with_extension("");
        let name: String = rel.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        let path = file.to_string_lossy().replace('\\', "/");
        let path = path.strip_prefix("//?/").unwrap_or(&path).to_string();
        writeln!(src, "#[path = {path:?}]\nmod {name};\n#[allow(unused_imports)]\npub use {name}::*;").unwrap();
    }
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("library_modules.rs"), src).unwrap();
}
