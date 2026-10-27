//! PBR material sets: finds the maps that belong to a model or texture by file name, and builds the
//! texture keys the renderer understands.
//!
//! Supported naming (the part before the suffix is the shared stem):
//! * Unity / HDRP packs: `<stem>_AlbedoTransparency.png`, `_MetallicSmoothness.png` (R = metallic,
//!   A = smoothness), `_Normal.png`.
//! * Substance / generic: `_BaseColor`, `_Normal` (or `_NormalGL`), `_Roughness`, `_Metallic`,
//!   `_AmbientOcclusion` / `_AO`.

use crate::assets::Assets;

/// Texture key prefix: a packed occlusion / roughness / metallic texture composed from separate maps.
pub const ORM_PREFIX: &str = "orm:";

/// Discovered texture maps for one physically based material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PbrSet {
    /// Base-colour texture asset path, or empty if none was discovered.
    pub albedo: String,
    /// OpenGL tangent-space normal map path, or empty if none was discovered.
    pub normal: String,
    /// Key for `MeshRenderer.orm_map` (empty if the set has no roughness/metal/AO information).
    pub orm: String,
}

/// Builds an `orm:` key. Each part is an asset path; `smooth` is a Unity MetallicSmoothness map
/// (metal from red, smoothness from alpha) that supplies both metal and roughness.
pub fn orm_key(ao: &str, rough: &str, metal: &str, smooth: &str) -> String {
    if ao.is_empty() && rough.is_empty() && metal.is_empty() && smooth.is_empty() {
        return String::new();
    }
    format!("{ORM_PREFIX}ao={ao}|rough={rough}|metal={metal}|smooth={smooth}")
}

/// Splits an `orm:` key back into `(ao, rough, metal, smooth)`.
pub fn parse_orm_key(key: &str) -> Option<[String; 4]> {
    let body = key.strip_prefix(ORM_PREFIX)?;
    let mut out: [String; 4] = Default::default();
    for part in body.split('|') {
        let (name, value) = part.split_once('=')?;
        match name {
            "ao" => out[0] = value.into(),
            "rough" => out[1] = value.into(),
            "metal" => out[2] = value.into(),
            "smooth" => out[3] = value.into(),
            _ => return None,
        }
    }
    Some(out)
}

fn lower(s: &str) -> String {
    s.to_ascii_lowercase()
}

/// Maps found next to `path` (a model or any texture of the set).
pub fn discover(assets: &Assets, path: &str) -> PbrSet {
    let (dir, file) = path.rsplit_once('/').map_or(("", path), |(d, f)| (d, f));
    let stem = file.rsplit_once('.').map_or(file, |(s, _)| s);
    // Strip a map suffix so any member of a set finds the others.
    let stem = [
        "_albedotransparency",
        "_basecolor",
        "_albedo",
        "_diffuse",
        "_normal",
        "_metallicsmoothness",
        "_roughness",
        "_metallic",
        "_ambientocclusion",
        "_ao",
        "_height",
    ]
    .iter()
    .find_map(|suffix| lower(stem).strip_suffix(suffix).map(|s| stem[..s.len()].to_string()))
    .unwrap_or_else(|| stem.to_string());
    let stem_l = lower(&stem);
    let siblings: Vec<String> = assets.list().into_iter().filter(|p| p.rsplit_once('/').map_or("", |(d, _)| d) == dir).collect();
    let find = |suffixes: &[&str]| -> String {
        for s in suffixes {
            let want = format!("{stem_l}{s}");
            let hit = siblings.iter().find(|p| {
                let f = lower(p.rsplit('/').next().unwrap_or(p));
                f.rsplit_once('.').is_some_and(|(name, ext)| name == want && matches!(ext, "png" | "jpg" | "jpeg"))
            });
            if let Some(h) = hit {
                return h.clone();
            }
        }
        String::new()
    };
    let albedo = find(&["_albedotransparency", "_basecolor", "_albedo", "_diffuse"]);
    let normal = find(&["_normal", "_normalgl"]);
    let (smooth, rough, metal, ao) =
        (find(&["_metallicsmoothness"]), find(&["_roughness"]), find(&["_metallic"]), find(&["_ambientocclusion", "_ao"]));
    PbrSet { albedo, normal, orm: orm_key(&ao, &rough, &metal, &smooth) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orm_keys_roundtrip() {
        let key = orm_key("a b/ao.png", "r.png", "", "");
        assert_eq!(parse_orm_key(&key), Some(["a b/ao.png".into(), "r.png".into(), "".into(), "".into()]));
        assert_eq!(orm_key("", "", "", ""), "");
    }

    #[test]
    fn discovers_unity_and_substance_sets() {
        let dir = std::env::temp_dir().join(format!("kvas_pbr_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("m")).unwrap();
        for f in [
            "trn_Barrel.fbx",
            "trn_Barrel_AlbedoTransparency.png",
            "trn_Barrel_Normal.png",
            "trn_Barrel_MetallicSmoothness.png",
            "Brick_BaseColor.png",
            "Brick_Roughness.png",
            "Brick_AmbientOcclusion.png",
            "Brick_Normal.png",
        ] {
            std::fs::write(dir.join("m").join(f), b"x").unwrap();
        }
        let assets = Assets::Dir(dir.clone());
        let unity = discover(&assets, "m/trn_Barrel.fbx");
        assert_eq!((unity.albedo.as_str(), unity.normal.as_str()), ("m/trn_Barrel_AlbedoTransparency.png", "m/trn_Barrel_Normal.png"));
        assert!(unity.orm.contains("smooth=m/trn_Barrel_MetallicSmoothness.png"));
        let sub = discover(&assets, "m/Brick_Normal.png");
        assert_eq!(sub.albedo, "m/Brick_BaseColor.png");
        assert!(sub.orm.contains("rough=m/Brick_Roughness.png") && sub.orm.contains("ao=m/Brick_AmbientOcclusion.png"));
        std::fs::remove_dir_all(dir).ok();
    }
}
