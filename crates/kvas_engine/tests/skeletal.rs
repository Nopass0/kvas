//! Skinned characters of LAST SERVICE: the rest pose skins back onto the model exactly, the
//! monster's clips come from its file, the humanoids get procedural ones, and playing a clip
//! moves vertices. Skipped when the game's assets were not imported.

use glam::Vec3;
use kvas_engine::assets::Assets;
use kvas_engine::skeletal;
use std::path::PathBuf;

fn assets() -> Option<Assets> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games/last_service/assets");
    root.join("models/characters/MonsterPSX.fbx").exists().then(|| Assets::Dir(root))
}

fn max_gap(a: &kvas_engine::mesh::MeshData, b: &kvas_engine::mesh::MeshData) -> f32 {
    a.vertices.iter().zip(&b.vertices).map(|(x, y)| Vec3::from(x.pos).distance(Vec3::from(y.pos))).fold(0.0, f32::max)
}

#[test]
fn characters_skin_and_animate() {
    let Some(assets) = assets() else {
        eprintln!("LAST SERVICE assets are not imported; skipping");
        return;
    };
    for (file, clips) in [
        ("models/characters/MonsterPSX.fbx", &["Idle_Watchful", "Walk_Nervous", "Run_Frantic", "Attack_Lunge"][..]),
        ("models/characters/female_secretary.fbx", &["Idle", "Walk", "Talk", "Wipe", "Sit"][..]),
        ("models/characters/male_casual.fbx", &["Idle", "Walk", "Look"][..]),
    ] {
        let model = skeletal::load(&assets, file).unwrap_or_else(|e| panic!("{file}: {e}"));
        // The rest pose must reproduce the static model.
        let mut skinned = model.mesh.clone();
        model.skin_into(&model.rest_pose(), &mut skinned);
        let gap = max_gap(&skinned, &model.mesh);
        assert!(gap < 1e-3, "{file}: rest pose moves vertices by {gap}");
        for name in clips {
            let clip = model.clip(name).unwrap_or_else(|| panic!("{file}: no clip {name}; has {:?}", model.clips.iter().map(|c| &c.name).collect::<Vec<_>>()));
            let mut pose = Vec::new();
            model.sample(clip, clip.duration * 0.37, true, &mut pose);
            let mut moved = model.mesh.clone();
            model.skin_into(&pose, &mut moved);
            let gap = max_gap(&moved, &model.mesh);
            let size = model.mesh.bounds();
            let height = size.1.y - size.0.y;
            assert!(gap > 0.005 * height, "{file}: {name} does not move the mesh ({gap})");
            assert!(gap < 2.5 * height, "{file}: {name} tears the mesh apart ({gap})");
            assert!(moved.vertices.iter().all(|v| v.pos.iter().all(|c| c.is_finite())));
        }
    }
}
