//! Archive validation, loose/packed parity and transactional persistence regressions.
use kvas_engine::{
    assets::{self, Assets},
    project::{Project, ProjectConfig},
    registry::Registry,
    scene::Scene,
};
use std::{
    borrow::Cow,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("kvas_asset_api_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        if self.0.is_absolute()
            && self.0.starts_with(std::env::temp_dir())
            && self.0.file_name().unwrap().to_string_lossy().starts_with("kvas_asset_api_")
        {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn archive_rebuild_is_deterministic_and_excludes_its_own_output() {
    let dir = TempDir::new();
    std::fs::create_dir(dir.0.join("textures")).unwrap();
    std::fs::write(dir.0.join("textures/a.txt"), b"original").unwrap();
    std::fs::write(dir.0.join("z.txt"), b"z").unwrap();
    let out = dir.0.join("game.pak");
    let extra = [("textures/a.txt", b"override".to_vec()), ("textures/a.txt", b"last wins".to_vec())];
    assert_eq!(assets::pack(&dir.0, &out, &extra).unwrap(), 2);
    let first = std::fs::read(&out).unwrap();
    assert_eq!(assets::pack(&dir.0, &out, &extra).unwrap(), 2);
    assert_eq!(first, std::fs::read(&out).unwrap());
    assert!(first.starts_with(b"KVASPAK2"));
    let packed = Assets::open_pak(&out).unwrap();
    assert_eq!(packed.list(), vec!["textures/a.txt", "z.txt"]);
    assert_eq!(packed.list_prefix("textures/"), vec!["textures/a.txt"]);
    assert!(packed.list_prefix("text").is_empty());
    assert_eq!(packed.read_string("textures\\a.txt").unwrap(), "last wins");
    assert_eq!(packed.byte_len("textures/a.txt").unwrap(), 9);
    assert_eq!(packed.read_bytes("z.txt").unwrap().as_ref(), b"z");
    assert!(!packed.exists("missing"));
    assert!(packed.modified("z.txt").is_none());
}

#[test]
fn asset_paths_normalize_internal_parents_but_never_escape_root() {
    assert_eq!(assets::normalize_path("models/../textures\\a.png").unwrap(), "textures/a.png");
    for invalid in ["", ".", "..", "../secret", "a/../../secret", "/absolute", "C:\\secret", "\\\\server\\share", "a\0b"] {
        assert!(assets::normalize_path(invalid).is_err(), "{invalid:?}");
    }
    let dir = TempDir::new();
    let loose = Assets::Dir(dir.0.clone());
    std::fs::write(dir.0.join("valid.txt"), b"valid").unwrap();
    assert_eq!(loose.read_string("sub/../valid.txt").unwrap(), "valid");
    assert!(matches!(loose.read_bytes("valid.txt").unwrap(), Cow::Owned(_)));
    assert!(loose.read("../outside.txt").is_err());
    assert!(!loose.exists("."));
}

fn one_entry(name: &str, offset: u64, length: u64, body: &[u8]) -> Vec<u8> {
    let mut bytes = b"KVASPAK1".to_vec();
    bytes.extend(1u32.to_le_bytes());
    bytes.extend((name.len() as u32).to_le_bytes());
    bytes.extend(name.as_bytes());
    bytes.extend(offset.to_le_bytes());
    bytes.extend(length.to_le_bytes());
    bytes.extend(body);
    bytes
}

#[test]
fn malformed_archive_tables_fail_without_panicking() {
    for bytes in [
        vec![],
        b"KVASPAK1".to_vec(),
        one_entry("a", 0, 1, b"x"),
        one_entry("a", u64::MAX, 1, b"x"),
        one_entry("../bad", 38, 1, b"x"),
        one_entry("a", 33, u64::MAX, b"x"),
    ] {
        assert!(Assets::from_pak_bytes(bytes).is_err());
    }
    let valid = one_entry("a", 33, 1, b"x");
    assert_eq!(Assets::from_pak_bytes(valid.clone()).unwrap().read("a").unwrap(), b"x");
    for cut in 0..valid.len() {
        assert!(Assets::from_pak_bytes(valid[..cut].to_vec()).is_err());
    }
    // Exercise all byte values in each table position; success is allowed only
    // when the resulting table still satisfies the documented invariants.
    for index in 0..valid.len() {
        for replacement in [0, 1, 31, 127, 255] {
            let mut mutated = valid.clone();
            mutated[index] = replacement;
            if let Ok(pak) = Assets::from_pak_bytes(mutated) {
                for name in pak.list() {
                    pak.read(&name).unwrap();
                }
            }
        }
    }
}

#[test]
fn scene_and_project_saves_replace_existing_files_and_leave_no_temporary_files() {
    let dir = TempDir::new();
    let reg = Registry::default();
    let path = dir.0.join("main.scene.json");
    let mut scene = Scene::default();
    scene.spawn("first");
    scene.save(&reg, &path).unwrap();
    scene.spawn("second");
    scene.save(&reg, &path).unwrap();
    let (loaded, warnings) = Scene::load(&reg, &path).unwrap();
    assert!(warnings.is_empty());
    assert_eq!(loaded.world.len(), 2);
    let mut project = Project { root: dir.0.clone(), config: ProjectConfig::default() };
    project.save().unwrap();
    project.config.name = "replaced".into();
    project.save().unwrap();
    assert_eq!(Project::open(&dir.0).unwrap().config.name, "replaced");
    assert!(std::fs::read_dir(&dir.0).unwrap().all(|entry| !entry.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
    assert!(project.config.validate().is_empty());
    project.config.start_scene = "../escape.json".into();
    project.config.input[0].keys.push("Typo".into());
    assert_eq!(project.config.validate().len(), 2);
}

fn quad_gltf(mode: u32, cyclic: bool) -> (String, Vec<u8>) {
    let positions = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
    let mut binary = Vec::new();
    for vertex in positions {
        for value in vertex {
            binary.extend(value.to_le_bytes());
        }
    }
    for index in [0u16, 1, 2, 3] {
        binary.extend(index.to_le_bytes());
    }
    let node = if cyclic { serde_json::json!({"mesh":0,"children":[0]}) } else { serde_json::json!({"mesh":0,"scale":[-1,1,1]}) };
    let document = serde_json::json!({
        "asset":{"version":"2.0"}, "scene":0, "scenes":[{"nodes":[0]}], "nodes":[node],
        "buffers":[{"uri":"data%20file.bin","byteLength":binary.len()}],
        "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":48},{"buffer":0,"byteOffset":48,"byteLength":8}],
        "accessors":[{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[0,0,0],"max":[1,1,0]},
                     {"bufferView":1,"componentType":5123,"count":4,"type":"SCALAR"}],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1,"mode":mode}]}]
    });
    (document.to_string(), binary)
}

#[test]
fn gltf_triangle_strips_external_uri_spaces_and_mirrored_winding_import_correctly() {
    use glam::Vec3;
    use kvas_engine::mesh::MeshData;
    let dir = TempDir::new();
    std::fs::create_dir(dir.0.join("models")).unwrap();
    let (text, binary) = quad_gltf(5, false);
    std::fs::write(dir.0.join("models/test.gltf"), text).unwrap();
    std::fs::write(dir.0.join("models/data file.bin"), binary).unwrap();
    let mesh = MeshData::load(&Assets::Dir(dir.0.clone()), "models/test.gltf").unwrap();
    assert_eq!(mesh.triangle_count(), 2);
    assert_eq!(mesh.dependencies, vec!["models/data file.bin"]);
    assert!((mesh.surface_area().unwrap() - 1.0).abs() < 1e-5);
    for triangle in mesh.indices.as_chunks::<3>().0.iter() {
        let a = &mesh.vertices[triangle[0] as usize];
        let b = &mesh.vertices[triangle[1] as usize];
        let c = &mesh.vertices[triangle[2] as usize];
        let normal = (Vec3::from(b.pos) - Vec3::from(a.pos)).cross(Vec3::from(c.pos) - Vec3::from(a.pos));
        assert!(normal.dot(Vec3::from(a.normal)) > 0.0);
    }
    let out = dir.0.join("game.pak");
    assets::pack(&dir.0.join("models"), &out, &[]).unwrap();
    let packed = Assets::open_pak(&out).unwrap();
    assert_eq!(MeshData::load(&packed, "test.gltf").unwrap().triangle_count(), 2);
}

#[test]
fn gltf_bad_indices_truncated_buffers_cycles_and_nontriangle_modes_are_rejected() {
    use kvas_engine::mesh::MeshData;
    let dir = TempDir::new();
    let assets = Assets::Dir(dir.0.clone());
    let (text, binary) = quad_gltf(5, false);
    let model = dir.0.join("test.gltf");
    let data = dir.0.join("data file.bin");
    std::fs::write(&model, &text).unwrap();
    std::fs::write(&data, &binary[..12]).unwrap();
    assert!(MeshData::load(&assets, "test.gltf").is_err());
    let mut bad_indices = binary.clone();
    bad_indices[48..50].copy_from_slice(&99u16.to_le_bytes());
    std::fs::write(&data, bad_indices).unwrap();
    assert!(MeshData::load(&assets, "test.gltf").is_err());
    std::fs::write(&data, binary).unwrap();
    let (cycle, _) = quad_gltf(5, true);
    std::fs::write(&model, cycle).unwrap();
    assert!(MeshData::load(&assets, "test.gltf").is_err());
    let (lines, _) = quad_gltf(1, false);
    std::fs::write(&model, lines).unwrap();
    assert!(MeshData::load(&assets, "test.gltf").is_err());
}

#[test]
fn invalid_scene_data_does_not_overwrite_a_valid_saved_scene() {
    use kvas_engine::components::Transform;
    let dir = TempDir::new();
    let path = dir.0.join("safe.scene.json");
    let registry = Registry::default();
    let mut scene = Scene::default();
    let entity = scene.spawn("valid");
    scene.save(&registry, &path).unwrap();
    let valid = std::fs::read(&path).unwrap();
    scene.world.get_mut::<Transform>(entity).unwrap().position.x = f32::NAN;
    assert!(scene.try_to_json(&registry).is_err());
    assert!(scene.save(&registry, &path).is_err());
    assert!(!scene.validate(&registry).is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), valid);
    scene.world.get_mut::<Transform>(entity).unwrap().position.x = 0.0;
    scene.settings.exposure = f32::INFINITY;
    assert!(scene.save(&registry, &path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), valid);
    assert!(Assets::Dir(dir.0.join("missing")).try_list().is_err());
    assert!(assets::pack(&dir.0.join("missing"), &dir.0.join("bad.pak"), &[]).is_err());
}
