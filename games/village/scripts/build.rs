// Kvas script auto-registration. Do not edit.
//
// Every `.rs` file under `src/` (except lib.rs) can hold scripts: each type with
// `impl Script for Name` becomes a script, with no manual `mod` or `register_scripts!` lists.
//
// Only scripts the game actually uses are compiled, so an unfinished script that is not on any
// object cannot break the build and costs nothing. A script is used when its name appears in a
// scene or prefab under `assets/`, in `.kvas/used_scripts.txt` (written by the editor for the
// open scene), or in the source of another used script (for `attach_script("Name")` or shared
// types). Files without scripts are shared code and are always compiled.
// Set the environment variable KVAS_ALL_SCRIPTS=1 to compile every script (to check them all).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, ext: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = read.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n != "target" && n != "build" && n != ".git") {
                collect(&p, ext, out);
            }
        } else if p.extension().is_some_and(|e| ext.iter().any(|x| e == *x)) {
            out.push(p);
        }
    }
}

/// True when `word` occurs in `text` as a whole identifier.
fn mentions(text: &str, word: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut from = 0;
    while let Some(i) = text[from..].find(word) {
        let start = from + i;
        let end = start + word.len();
        let before = text[..start].chars().next_back().is_some_and(ident);
        let after = text[end..].chars().next().is_some_and(ident);
        if !before && !after {
            return true;
        }
        from = end;
    }
    false
}

struct Def {
    name: String,
    update: bool,
    fixed: bool,
}

struct File {
    path: PathBuf,
    module: String,
    text: String,
    defs: Vec<Def>,
}

/// Scripts declared in a file and whether each overrides `update` / `fixed_update`
/// (scripts without them are never called per frame).
fn defs_of(text: &str) -> Vec<Def> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix("impl Script for ") else { continue };
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if name.is_empty() {
            continue;
        }
        // The impl block ends at the next item that starts at column 0 with `}`.
        let body: Vec<&str> = lines[i + 1..].iter().take_while(|l| !l.starts_with('}')).copied().collect();
        let has = |f: &str| body.iter().any(|l| l.contains(&format!("fn {f}(")));
        out.push(Def { name, update: has("update"), fixed: has("fixed_update") });
    }
    out
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = root.join("src");
    let project = root.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut paths = Vec::new();
    collect(&src, &["rs"], &mut paths);
    let mut files = Vec::new();
    for path in paths {
        let rel = path.strip_prefix(&src).unwrap().to_path_buf();
        if rel == Path::new("lib.rs") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let module: String = rel
            .with_extension("")
            .to_string_lossy()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
            .collect();
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let defs = defs_of(&text);
        files.push(File { path, module: format!("s_{module}"), text, defs });
    }
    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-env-changed=KVAS_ALL_SCRIPTS");

    // Where the game says which scripts it needs.
    let mut usage = String::new();
    let mut data = Vec::new();
    collect(&project.join("assets"), &["json"], &mut data);
    for path in data {
        let name = path.to_string_lossy().to_lowercase();
        if name.contains("scene") || name.contains("prefab") {
            println!("cargo:rerun-if-changed={}", path.display());
            usage.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
        }
    }
    let listed = project.join(".kvas").join("used_scripts.txt");
    println!("cargo:rerun-if-changed={}", listed.display());
    for line in std::fs::read_to_string(&listed).unwrap_or_default().lines() {
        let _ = write!(usage, " \"{}\" ", line.trim());
    }
    let all = std::env::var("KVAS_ALL_SCRIPTS").is_ok_and(|v| v == "1");

    let mut included: Vec<bool> = files
        .iter()
        .map(|f| all || f.defs.is_empty() || f.defs.iter().any(|d| usage.contains(&format!("\"{}\"", d.name))))
        .collect();
    // Scripts that included code refers to are needed too.
    loop {
        let mut changed = false;
        for i in 0..files.len() {
            if included[i] {
                continue;
            }
            let needed = files.iter().zip(&included).any(|(f, inc)| *inc && files[i].defs.iter().any(|d| mentions(&f.text, &d.name)));
            if needed {
                included[i] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut modules = String::new();
    let mut regs = Vec::new();
    for (file, _) in files.iter().zip(&included).filter(|(_, inc)| **inc) {
        let _ = writeln!(modules, "#[path = {:?}]\npub mod {};", file.path.to_string_lossy().replace('\\', "/"), file.module);
        for d in &file.defs {
            regs.push(format!("{}::{} => {}, {}", file.module, d.name, d.update, d.fixed));
        }
    }
    let out = format!("{modules}\nkvas_engine::register_scripts!(@flags {});\n", regs.join("; "));
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("kvas_scripts.rs");
    std::fs::write(dest, out).unwrap();
}
