//! Scene = ECS world + environment settings, saved as readable JSON.

use crate::components::*;
use crate::hierarchy;
use crate::registry::Registry;
use glam::Vec3;
use kvas_ecs::{Entity, World};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Serializable scene environment. Missing JSON fields use these defaults.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SceneSettings {
    /// Ambient RGB illumination, normally in the 0..1 range.
    pub ambient: [f32; 3],
    /// Fallback background RGB colour when the atmosphere is disabled.
    pub sky: [f32; 3],
    /// World-space gravity acceleration in metres per second squared.
    pub gravity: Vec3,
    /// Cel shading: 0 = off, otherwise the number of light bands (3 = anime look).
    pub toon_bands: u32,
    /// Colour saturation multiplier; 1 preserves the rendered colours.
    pub saturation: f32,
    /// Display contrast multiplier; 1 is neutral.
    pub contrast: f32,
    /// Exposure multiplier applied before display conversion; 1 is neutral.
    pub exposure: f32,
    /// PlayStation-style look: wobbling vertex snapping, point-sampled textures, dithered 15-bit colour.
    pub psx: bool,
    /// Render resolution scale (0.25..1). Below 1 the picture is upscaled with point filtering.
    pub render_scale: f32,
    /// Procedural sky and time of day.
    pub atmosphere: crate::sky::SkySettings,
    /// Sun shadow quality and look.
    pub shadows: crate::fx::ShadowSettings,
    /// Screen-space effects (bloom, blur, motion blur, flares, grain...).
    pub post: crate::fx::PostSettings,
    /// Upscaling method used when `render_scale` is below 1.
    pub upscaler: crate::fx::Upscaler,
    /// Experimental optical-flow midpoint presentation in Game view (not NVIDIA DLSS).
    /// Alternates a synthesized frame with its real frame, adding visual latency.
    pub frame_generation: bool,
    /// Sharpening of the FSR-style upscaler, 0..1.
    pub sharpness: f32,
    /// Objects farther than this many metres are neither drawn nor kept in GPU memory (0 = unlimited).
    pub view_distance: f32,
    /// Filmic (ACES-style) tone mapping strength: 0 = plain clamp, 1 = full highlight roll-off.
    pub filmic: f32,
    /// Distance haze density (0 = off, 0.005 light, 0.02 thick). The colour follows the sky.
    pub fog: f32,
    /// Equirectangular .hdr/.png/.jpg asset used as background and mirror environment; empty uses sky.
    pub environment_map: String,
    /// Environment rotation around +Y, in degrees.
    pub environment_rotation: f32,
    /// Environment radiance multiplier.
    pub environment_intensity: f32,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            ambient: [0.25, 0.27, 0.32],
            sky: [0.35, 0.5, 0.75],
            gravity: Vec3::new(0.0, -9.81, 0.0),
            toon_bands: 0,
            saturation: 1.0,
            contrast: 1.0,
            exposure: 1.0,
            psx: false,
            render_scale: 1.0,
            atmosphere: Default::default(),
            shadows: Default::default(),
            post: Default::default(),
            upscaler: Default::default(),
            frame_generation: false,
            sharpness: 0.5,
            view_distance: 0.0,
            filmic: 0.7,
            fog: 0.0,
            environment_map: String::new(),
            environment_rotation: 0.0,
            environment_intensity: 1.0,
        }
    }
}

/// ECS world plus environment settings. Default registers every built-in component.
///
/// Serialized scenes omit transient entities and rebuild world transforms on load.
/// Entity handles are remapped on load and must not be persisted outside the scene.
pub struct Scene {
    /// Live entity/component storage, including runtime-only components.
    pub world: World,
    /// Scene lighting, physics and display configuration.
    pub settings: SceneSettings,
    /// Bounded next-frame script messages. Not serialized or copied into scene snapshots.
    pub signals: crate::signals::SignalBus,
    /// Sound requests for the shell's audio backend. Not serialized.
    pub audio: crate::audio::AudioBus,
}

impl Default for Scene {
    fn default() -> Self {
        let mut world = World::new();
        register_all(&mut world);
        Scene { world, settings: SceneSettings::default(), signals: Default::default(), audio: Default::default() }
    }
}

#[derive(Serialize, Deserialize)]
struct ComponentData {
    #[serde(rename = "type")]
    kind: String,
    data: Value,
}

#[derive(Serialize, Deserialize)]
struct EntityData {
    parent: Option<usize>,
    components: Vec<ComponentData>,
}

#[derive(Serialize, Deserialize, Default)]
struct SceneFile {
    #[serde(default)]
    settings: SceneSettings,
    entities: Vec<EntityData>,
}

impl Scene {
    /// Serializes persistent entities and registered components as pretty JSON.
    /// Runtime-only state is omitted. Prefer [`Self::try_to_json`] when saving files.
    pub fn to_json(&self, reg: &Registry) -> String {
        self.try_to_json(reg).unwrap_or_default()
    }

    /// Serializes a scene, reporting JSON encoding errors instead of returning an empty string.
    /// Component serializers in the registry may omit components they cannot encode.
    pub fn try_to_json(&self, reg: &Registry) -> Result<String, String> {
        let entities: Vec<Entity> = self.world.entities().into_iter().filter(|&e| !self.world.has::<Transient>(e)).collect();
        let settings_json = serde_json::to_value(&self.settings).map_err(|e| format!("scene settings: {e}"))?;
        serde_json::from_value::<SceneSettings>(settings_json)
            .map_err(|e| format!("scene settings cannot round-trip through JSON: {e}"))?;
        for &entity in &entities {
            let errors = reg.validate_entity(&self.world, entity);
            if !errors.is_empty() {
                return Err(format!("entity {}: {}", entity.index(), errors.join("; ")));
            }
        }
        let index: HashMap<Entity, usize> = entities.iter().enumerate().map(|(i, &e)| (e, i)).collect();
        let entities = entities
            .iter()
            .map(|&e| EntityData {
                parent: hierarchy::parent_of(&self.world, e).and_then(|p| index.get(&p).copied()),
                components: reg
                    .components
                    .iter()
                    .filter_map(|c| (c.to_json)(&self.world, e).map(|data| ComponentData { kind: c.name.into(), data }))
                    .collect(),
            })
            .collect();
        serde_json::to_string_pretty(&SceneFile { settings: self.settings.clone(), entities })
            .map_err(|e| format!("scene serialization error: {e}"))
    }

    /// Returns the scene and any non-fatal warnings (unknown or malformed components).
    pub fn from_json(reg: &Registry, text: &str) -> Result<(Scene, Vec<String>), String> {
        let file: SceneFile = serde_json::from_str(text).map_err(|e| format!("scene parse error: {e}"))?;
        let mut scene = Scene { settings: file.settings, ..Scene::default() };
        reg.register_world(&mut scene.world);
        let mut warnings = Vec::new();
        let spawned: Vec<Entity> = file.entities.iter().map(|_| scene.world.spawn()).collect();
        let mut parents = Vec::new();
        for (data, &e) in file.entities.into_iter().zip(&spawned) {
            for c in data.components {
                match reg.get(&c.kind) {
                    Some(info) => {
                        if let Err(err) = (info.from_json)(&mut scene.world, e, c.data) {
                            warnings.push(err);
                        }
                    }
                    None => warnings.push(format!("unknown component '{}'", c.kind)),
                }
            }
            parents.push((e, data.parent));
        }
        // Resolve separately so validation is independent of entity/component order.
        let mut groups: Vec<usize> = (0..spawned.len()).collect();
        fn root(groups: &mut [usize], mut index: usize) -> usize {
            while groups[index] != index {
                groups[index] = groups[groups[index]];
                index = groups[index];
            }
            index
        }
        // Each child has at most one parent. Joining an already-connected pair
        // would close a directed cycle; union-find checks this without walking
        // every ancestor of every node in a deeply nested imported scene.
        for (child_index, (e, parent)) in parents.into_iter().enumerate() {
            if let Some(index) = parent {
                match spawned.get(index).copied() {
                    Some(p) => {
                        let child_root = root(&mut groups, child_index);
                        let parent_root = root(&mut groups, index);
                        if child_root == parent_root {
                            warnings.push(format!("entity {}: cyclic parent {index} ignored", e.index()));
                        } else {
                            groups[child_root] = parent_root;
                            scene.world.insert(e, Parent(p));
                        }
                    }
                    None => warnings.push(format!("entity {}: invalid parent index {index} ignored", e.index())),
                }
            }
        }
        hierarchy::update_globals(&mut scene.world);
        Ok((scene, warnings))
    }

    /// Loads a UTF-8 JSON scene from disk, preserving non-fatal component/hierarchy warnings.
    pub fn load(reg: &Registry, path: &Path) -> Result<(Scene, Vec<String>), String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_json(reg, &text)
    }

    /// Saves persistent scene data. Parent directories must already exist.
    pub fn save(&self, reg: &Registry, path: &Path) -> Result<(), String> {
        let text = self.try_to_json(reg)?;
        crate::file_io::atomic_write(path, text.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Creates an independent persistent snapshot, omitting transient/runtime-only state.
    /// Returns the same warnings as loading the serialized scene.
    pub fn snapshot(&self, reg: &Registry) -> Result<(Scene, Vec<String>), String> {
        Self::from_json(reg, &self.try_to_json(reg)?)
    }

    /// Reports persistence errors, dangling parents and hierarchy cycles without mutation.
    /// Missing transforms are allowed by the ECS but reported here for editor-visible entities.
    /// This explicit diagnostic pass does not run automatically on every gameplay tick.
    pub fn validate(&self, reg: &Registry) -> Vec<String> {
        let mut issues = Vec::new();
        if let Err(error) = serde_json::to_value(&self.settings).and_then(serde_json::from_value::<SceneSettings>) {
            issues.push(format!("scene settings: {error}"));
        }
        for entity in self.world.iter_entities() {
            if self.world.has::<Transient>(entity) {
                continue;
            }
            issues.extend(reg.validate_entity(&self.world, entity).into_iter().map(|error| format!("entity {}: {error}", entity.index())));
            if !self.world.has::<Transform>(entity) {
                issues.push(format!("entity {} has no Transform", entity.index()));
            }
            let mut current = Some(entity);
            let mut seen = std::collections::HashSet::new();
            while let Some(e) = current {
                if !self.world.is_alive(e) {
                    issues.push(format!("entity {} has a dangling parent", entity.index()));
                    break;
                }
                if !seen.insert(e) {
                    issues.push(format!("entity {} has a cyclic parent chain", entity.index()));
                    break;
                }
                current = self.world.get::<Parent>(e).map(|p| p.0);
            }
        }
        issues
    }

    /// Spawns a named entity with an identity local transform.
    pub fn spawn(&mut self, name: &str) -> Entity {
        hierarchy::spawn_named(&mut self.world, name)
    }

    /// First exact name match, if present. Entity names need not be unique.
    pub fn find(&self, name: &str) -> Option<Entity> {
        hierarchy::find_by_name(&self.world, name)
    }

    /// Removes an entity and its complete subtree, including transient children.
    pub fn despawn(&mut self, entity: Entity) {
        hierarchy::despawn_recursive(&mut self.world, entity);
    }
}

/// Deep-copies registered components of a non-transient subtree.
/// The copy is a sibling of the original; only the root gets a ` (Copy)` suffix.
/// Runtime-only components are omitted and name-based references are not remapped.
/// Traversal terminates even for malformed cyclic input hierarchies.
///
/// # Panics
/// Panics when `e` is stale. The destination world must register every copied type.
pub fn duplicate(world: &mut World, reg: &Registry, e: Entity) -> Entity {
    assert!(world.is_alive(e), "cannot duplicate a stale entity");
    let mut originals = vec![e];
    let mut cursor = 0;
    let mut seen = std::collections::HashSet::from([e]);
    while cursor < originals.len() {
        let current = originals[cursor];
        for child in hierarchy::children(world, current) {
            if seen.insert(child) {
                originals.push(child);
            }
        }
        cursor += 1;
    }
    let mut copies = HashMap::new();
    for &original in &originals {
        let copy = world.spawn();
        reg.copy_entity(world, original, copy);
        copies.insert(original, copy);
    }
    let copy = copies[&e];
    if let Some(n) = world.get_mut::<Name>(copy) {
        n.0.push_str(" (Copy)");
    }
    for original in originals {
        if let Some(parent) = hierarchy::parent_of(world, original) {
            let target = if original == e { (!copies.contains_key(&parent)).then_some(parent) } else { copies.get(&parent).copied() };
            if let Some(target) = target {
                hierarchy::set_parent_local(world, copies[&original], Some(target));
            }
        }
    }
    copy
}
