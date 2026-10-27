//! Records the compiler version and a hash of the engine sources. Scripts are only loaded when
//! both match the host's, because host and script library share engine data structures in memory.

use std::path::{Path, PathBuf};
use std::{env, fs};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut files = Vec::new();
    for dir in [manifest.join("src"), manifest.join("../kvas_ecs/src")] {
        println!("cargo:rerun-if-changed={}", dir.display());
        collect(&dir, &mut files);
    }
    for path in
        [manifest.join("build.rs"), manifest.join("Cargo.toml"), manifest.join("../kvas_ecs/Cargo.toml"), manifest.join("../../Cargo.toml")]
    {
        println!("cargo:rerun-if-changed={}", path.display());
        files.push(path);
    }
    files.sort();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        for byte in name.bytes().chain(fs::read(file).unwrap_or_default()) {
            hash = (hash ^ byte as u64).wrapping_mul(0x0100_0000_01b3);
        }
    }
    println!("cargo:rustc-env=KVAS_SRC_HASH={hash:016x}");
    let mut config = String::new();
    for key in [
        "TARGET",
        "CARGO_CFG_TARGET_FEATURE",
        "CARGO_CFG_TARGET_POINTER_WIDTH",
        "CARGO_CFG_TARGET_ENDIAN",
        "CARGO_CFG_PANIC",
        "CARGO_ENCODED_RUSTFLAGS",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        config.push_str(key);
        config.push('=');
        config.push_str(&env::var(key).unwrap_or_default());
        config.push('|');
    }
    println!("cargo:rustc-env=KVAS_BUILD_CONFIG={config}");

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let version = std::process::Command::new(rustc)
        .arg("-vV")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).replace(['\r', '\n'], "|"))
        .unwrap_or_default();
    println!("cargo:rustc-env=KVAS_RUSTC_VERSION={version}");
}
