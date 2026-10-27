use kvas_engine::assets::Assets;
use std::collections::HashMap;
use std::time::SystemTime;

/// Derived GPU keys depend on the source files rather than their cache prefixes.
fn dependencies(key: &str) -> Vec<String> {
    let key = key.strip_prefix("srgb:").or_else(|| key.strip_prefix("data:")).unwrap_or(key);
    if key.starts_with(kvas_engine::materials::ORM_PREFIX) {
        return kvas_engine::materials::parse_orm_key(key).into_iter().flatten().filter(|path| !path.is_empty()).collect();
    }
    if ["builtin:", "primitive:", "proc:", "noise:", "cloudvol:", "dyn:"].iter().any(|prefix| key.starts_with(prefix)) {
        return Vec::new();
    }
    vec![key.split('#').next().unwrap_or(key).to_owned()]
}

struct Entry<T> {
    value: Option<T>,
    stamps: Vec<(String, Option<SystemTime>)>,
}

/// Lazily loaded assets keyed by path. A failed load is remembered (no retry spam) and
/// reported once; changed files are dropped by `invalidate_changed` so they reload (hot reload).
pub struct AssetCache<T> {
    map: HashMap<String, Entry<T>>,
}

impl<T> Default for AssetCache<T> {
    fn default() -> Self {
        Self { map: HashMap::new() }
    }
}

impl<T> AssetCache<T> {
    pub fn load(&mut self, key: &str, assets: &Assets, errors: &mut Vec<String>, load: impl FnOnce() -> Result<T, String>) {
        self.load_with_dependencies(key, assets, errors, || load().map(|value| (value, Vec::new())));
    }

    pub fn load_with_dependencies(
        &mut self,
        key: &str,
        assets: &Assets,
        errors: &mut Vec<String>,
        load: impl FnOnce() -> Result<(T, Vec<String>), String>,
    ) {
        if self.map.contains_key(key) {
            return;
        }
        let mut paths = dependencies(key);
        let value = match load() {
            Ok((value, extra_paths)) => {
                paths.extend(extra_paths);
                Some(value)
            }
            Err(e) => {
                errors.push(e);
                None
            }
        };
        paths.sort();
        paths.dedup();
        let stamps = paths
            .into_iter()
            .map(|path| {
                let stamp = assets.modified(&path);
                (path, stamp)
            })
            .collect();
        self.map.insert(key.to_string(), Entry { value, stamps });
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.map.get(key)?.value.as_ref()
    }

    /// Returns true if anything was dropped.
    pub fn invalidate_changed(&mut self, assets: &Assets) -> bool {
        let before = self.map.len();
        self.map.retain(|_, e| e.stamps.iter().all(|(path, stamp)| assets.modified(path) == *stamp));
        self.map.len() != before
    }

    /// Keys currently cached (loaded or failed).
    pub fn keys(&self) -> Vec<String> {
        self.map.keys().cloned().collect()
    }

    /// Drops one entry so it is loaded again the next time it is needed.
    pub fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_assets_retry_after_the_file_is_created() {
        let root = std::env::temp_dir();
        let name = format!(
            "kvas_cache_{}_{}.txt",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        );
        let path = root.join(&name);
        let assets = Assets::Dir(root);
        let mut cache = AssetCache::<usize>::default();
        let mut errors = Vec::new();
        cache.load(&name, &assets, &mut errors, || Err("missing".into()));
        cache.load(&name, &assets, &mut errors, || panic!("failed loads must be remembered"));
        assert_eq!(errors.len(), 1);
        assert!(cache.get(&name).is_none());
        assert!(!cache.invalidate_changed(&assets));
        std::fs::write(&path, "created").unwrap();
        assert!(cache.invalidate_changed(&assets));
        cache.load(&name, &assets, &mut errors, || Ok(7));
        assert_eq!(cache.get(&name), Some(&7));
        std::fs::remove_file(path).unwrap();
        assert!(cache.invalidate_changed(&assets));
    }

    #[test]
    fn derived_colour_data_and_orm_keys_track_source_dependencies() {
        assert_eq!(dependencies("srgb:textures/a.png"), vec!["textures/a.png"]);
        assert_eq!(dependencies("data:textures/a.png"), vec!["textures/a.png"]);
        assert_eq!(dependencies("shaders/a.wgsl#t"), vec!["shaders/a.wgsl"]);
        let orm = kvas_engine::materials::orm_key("ao.png", "rough.png", "metal.png", "");
        assert_eq!(dependencies(&format!("data:{orm}")), vec!["ao.png", "rough.png", "metal.png"]);
        assert!(dependencies("data:noise:Perlin:parameters").is_empty());
        let root = std::env::temp_dir();
        let name = format!("kvas_derived_cache_{}.png", std::process::id());
        let path = root.join(&name);
        let assets = Assets::Dir(root);
        let key = format!("data:{name}");
        let mut cache = AssetCache::default();
        cache.load(&key, &assets, &mut Vec::new(), || Ok(1));
        std::fs::write(&path, b"source").unwrap();
        assert!(cache.invalidate_changed(&assets));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn model_dependencies_are_watched_even_when_initially_missing() {
        let root = std::env::temp_dir();
        let name = format!("kvas_model_dependency_{}.bin", std::process::id());
        let path = root.join(&name);
        let assets = Assets::Dir(root);
        let mut cache = AssetCache::default();
        cache.load_with_dependencies("model.gltf", &assets, &mut Vec::new(), || Ok((5, vec![name.clone()])));
        assert!(!cache.invalidate_changed(&assets));
        std::fs::write(&path, b"now available").unwrap();
        assert!(cache.invalidate_changed(&assets));
        std::fs::remove_file(path).unwrap();
    }
}
