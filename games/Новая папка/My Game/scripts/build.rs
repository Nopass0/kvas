// Kvas script auto-registration. Do not edit.
//
// Every `.rs` file under `src/` (except lib.rs) becomes a module, and every type with
// `impl Script for Name` becomes a script, with no manual `mod` or `register_scripts!` lists.
// Just create `src/my_script.rs` (or a sub-folder) and the editor will see the script.

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
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = root.join("src");
    let mut files = Vec::new();
    collect(&src, &mut files);
    let mut modules = String::new();
    let mut names = Vec::new();
    for file in files {
        let rel = file.strip_prefix(&src).unwrap();
        if rel == Path::new("lib.rs") {
            continue;
        }
        let module: String = rel
            .with_extension("")
            .to_string_lossy()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
            .collect();
        let module = format!("s_{module}");
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        let _ = writeln!(modules, "#[path = {:?}]\npub mod {module};", file.to_string_lossy().replace('\\', "/"));
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("impl Script for ") {
                let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if !name.is_empty() {
                    names.push(format!("{module}::{name}"));
                }
            }
        }
        println!("cargo:rerun-if-changed={}", file.display());
    }
    println!("cargo:rerun-if-changed={}", src.display());
    let out = format!("{modules}\nkvas_engine::register_scripts!({});\n", names.join(", "));
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("kvas_scripts.rs");
    std::fs::write(dest, out).unwrap();
}
