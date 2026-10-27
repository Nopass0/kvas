//! Texture painting: blend several surface textures with a painted mask, so dirt can be brushed
//! along a road edge and fade into grass without a hard seam.
//!
//! A [`TexturePaint`] component names four layer textures and a mask. The mask is an RGBA image:
//! red, green and blue are the weights of layers 2, 3 and 4; whatever is left is layer 1. The
//! editor's brush edits the mask in memory ([`PaintMaps`]) and saves it to `assets/paint/<name>.png`.
//!
//! ```
//! use kvas_engine::paint::{PaintMaps, BrushMode};
//! let maps = PaintMaps::global();
//! maps.create("doc-example", 32, 32);
//! maps.brush("doc-example", [0.5, 0.5], 0.2, 1.0, 0.5, BrushMode::Layer(1));
//! let (_, rgba, _) = maps.snapshot("doc-example").unwrap();
//! assert!(rgba[(16 * 32 + 16) * 4] > 200); // the red channel (layer 2) at the centre
//! ```

use crate::assets::Assets;
use crate::components::*;
use crate::reflect::{AssetKind, Field};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// What the brush does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrushMode {
    /// Paint layer 0..=3 (0 is the base layer: it erases the others).
    Layer(usize),
    /// Soften the mask (blur under the brush).
    Smooth,
}

/// One painted mask kept in memory.
#[derive(Clone)]
pub struct PaintMap {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// RGBA8 pixels, row-major.
    pub rgba: Vec<u8>,
    /// Counter that grows with every edit.
    pub version: u64,
    /// Edited since the last save.
    pub dirty: bool,
}

/// All painted masks of the running editor / game, by name.
pub struct PaintMaps {
    maps: Mutex<HashMap<String, PaintMap>>,
}

impl PaintMaps {
    /// The process-wide registry (the renderer reads masks from here).
    pub fn global() -> &'static PaintMaps {
        static MAPS: OnceLock<PaintMaps> = OnceLock::new();
        MAPS.get_or_init(|| PaintMaps { maps: Mutex::new(HashMap::new()) })
    }

    /// Creates an empty mask (all base layer) unless one exists.
    pub fn create(&self, name: &str, width: u32, height: u32) {
        let mut maps = self.maps.lock().unwrap();
        maps.entry(name.to_string()).or_insert_with(|| PaintMap { width, height, rgba: vec![0; (width * height * 4) as usize], version: 1, dirty: false });
        // The alpha channel stays opaque so the image previews correctly.
        if let Some(m) = maps.get_mut(name) {
            for px in m.rgba.chunks_exact_mut(4) {
                px[3] = 255;
            }
        }
    }

    /// Whether a mask is loaded.
    pub fn exists(&self, name: &str) -> bool {
        self.maps.lock().unwrap().contains_key(name)
    }

    /// Loads `paint/<name>.png` from the project, or creates a fresh mask.
    pub fn load_or_create(&self, name: &str, assets: &Assets, size: u32) {
        if self.exists(name) {
            return;
        }
        let loaded = assets.read(&format!("paint/{name}.png")).ok().and_then(|b| image::load_from_memory(&b).ok());
        match loaded {
            Some(img) => {
                let rgba = img.to_rgba8();
                self.maps.lock().unwrap().insert(name.to_string(), PaintMap { width: rgba.width(), height: rgba.height(), rgba: rgba.into_raw(), version: 1, dirty: false });
            }
            None => self.create(name, size, size),
        }
    }

    /// `(size, pixels, version)` of a mask, for uploading to the GPU.
    pub fn snapshot(&self, name: &str) -> Option<([u32; 2], Vec<u8>, u64)> {
        let maps = self.maps.lock().unwrap();
        let m = maps.get(name)?;
        Some(([m.width, m.height], m.rgba.clone(), m.version))
    }

    /// Current version of a mask (0 when missing).
    pub fn version(&self, name: &str) -> u64 {
        self.maps.lock().unwrap().get(name).map_or(0, |m| m.version)
    }

    /// Applies one brush dab at `uv` (0..1). `radius` is in uv units, `strength` 0..1 per dab,
    /// `hardness` 0 (soft falloff) .. 1 (hard edge).
    pub fn brush(&self, name: &str, uv: [f32; 2], radius: f32, strength: f32, hardness: f32, mode: BrushMode) {
        let mut maps = self.maps.lock().unwrap();
        let Some(m) = maps.get_mut(name) else { return };
        let (w, h) = (m.width as i32, m.height as i32);
        let (cx, cy) = (uv[0] * w as f32, uv[1] * h as f32);
        let r = (radius * w as f32).max(1.0);
        let (x0, x1) = (((cx - r).floor() as i32).max(0), ((cx + r).ceil() as i32).min(w - 1));
        let (y0, y1) = (((cy - r).floor() as i32).max(0), ((cy + r).ceil() as i32).min(h - 1));
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = (((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt()) / r;
                if d >= 1.0 {
                    continue;
                }
                let edge = hardness.clamp(0.0, 0.98);
                let fall = if d <= edge { 1.0 } else { let t = (d - edge) / (1.0 - edge); let s = 1.0 - t; s * s * (3.0 - 2.0 * s) };
                let amount = fall * strength.clamp(0.0, 1.0);
                let i = ((y * w + x) * 4) as usize;
                let mut weights = [m.rgba[i] as f32 / 255.0, m.rgba[i + 1] as f32 / 255.0, m.rgba[i + 2] as f32 / 255.0];
                match mode {
                    BrushMode::Layer(0) => weights.iter_mut().for_each(|v| *v = (*v - amount).max(0.0)),
                    BrushMode::Layer(k) => {
                        let k = (k - 1).min(2);
                        weights[k] = (weights[k] + amount).min(1.0);
                        let total: f32 = weights.iter().sum();
                        if total > 1.0 {
                            // Take the surplus from the other layers in proportion to what they hold.
                            let others: f32 = total - weights[k];
                            let keep = (1.0 - weights[k]).max(0.0);
                            for (j, v) in weights.iter_mut().enumerate() {
                                if j != k && others > 0.0 {
                                    *v *= keep / others;
                                }
                            }
                        }
                    }
                    BrushMode::Smooth => {
                        let mut avg = [0.0f32; 3];
                        let mut n = 0.0;
                        for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2), (-1, -1), (1, 1), (-1, 1), (1, -1)] {
                            let (nx, ny) = ((x + dx).clamp(0, w - 1), (y + dy).clamp(0, h - 1));
                            let j = ((ny * w + nx) * 4) as usize;
                            for c in 0..3 {
                                avg[c] += m.rgba[j + c] as f32 / 255.0;
                            }
                            n += 1.0;
                        }
                        for c in 0..3 {
                            weights[c] += (avg[c] / n - weights[c]) * amount;
                        }
                    }
                }
                for c in 0..3 {
                    m.rgba[i + c] = (weights[c] * 255.0 + 0.5) as u8;
                }
            }
        }
        m.version += 1;
        m.dirty = true;
    }

    /// Fills the whole mask with one layer (a quick base coat).
    pub fn fill(&self, name: &str, layer: usize) {
        let mut maps = self.maps.lock().unwrap();
        if let Some(m) = maps.get_mut(name) {
            for px in m.rgba.chunks_exact_mut(4) {
                px[..3].copy_from_slice(&[0, 0, 0]);
                if layer >= 1 {
                    px[(layer - 1).min(2)] = 255;
                }
            }
            m.version += 1;
            m.dirty = true;
        }
    }

    /// Writes every edited mask to `<assets>/paint/<name>.png`. Returns how many were saved.
    pub fn save_all(&self, assets_dir: &std::path::Path) -> usize {
        let mut maps = self.maps.lock().unwrap();
        let mut saved = 0;
        for (name, m) in maps.iter_mut().filter(|(_, m)| m.dirty) {
            let dir = assets_dir.join("paint");
            let _ = std::fs::create_dir_all(&dir);
            if let Some(img) = image::RgbaImage::from_raw(m.width, m.height, m.rgba.clone()) {
                if img.save(dir.join(format!("{name}.png"))).is_ok() {
                    m.dirty = false;
                    saved += 1;
                }
            }
        }
        saved
    }

    /// Names of masks with unsaved edits.
    pub fn dirty_names(&self) -> Vec<String> {
        self.maps.lock().unwrap().iter().filter(|(_, m)| m.dirty).map(|(n, _)| n.clone()).collect()
    }
}

/// Four textures blended by a painted mask. Put it on an object with a mesh (a plane, a terrain
/// chunk, a road strip). The component takes over the object's material.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TexturePaint {
    /// Base layer (where nothing is painted), usually grass.
    pub layer1: String,
    /// Layer painted with the red channel (dirt, sand...).
    pub layer2: String,
    /// Layer painted with the green channel.
    pub layer3: String,
    /// Layer painted with the blue channel.
    pub layer4: String,
    /// Name of the painted mask (saved as `paint/<name>.png`). Empty = the object's name.
    pub mask: String,
    /// Texture repeats per metre of the surface.
    pub tiling: f32,
    /// Noise that roughens the borders between layers, 0..1.
    pub breakup: f32,
    /// Mask resolution when a new mask is created.
    pub resolution: i32,
}

impl Default for TexturePaint {
    fn default() -> Self {
        Self { layer1: String::new(), layer2: String::new(), layer3: String::new(), layer4: String::new(), mask: String::new(), tiling: 0.25, breakup: 0.35, resolution: 512 }
    }
}

impl Component for TexturePaint {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::asset("layer1", &mut self.layer1, AssetKind::Texture),
            Field::asset("layer2", &mut self.layer2, AssetKind::Texture),
            Field::asset("layer3", &mut self.layer3, AssetKind::Texture),
            Field::asset("layer4", &mut self.layer4, AssetKind::Texture),
            Field::text("mask", &mut self.mask),
            Field::f32("tiling", &mut self.tiling),
            Field::f32("breakup", &mut self.breakup),
            Field::i32("resolution", &mut self.resolution),
        ]
    }
}

impl TexturePaint {
    /// The mask name for an entity called `entity_name`.
    pub fn mask_name(&self, entity_name: &str) -> String {
        let n = if self.mask.trim().is_empty() { entity_name } else { self.mask.trim() };
        n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
    }
}

/// Prefix of the mask texture keys the renderer understands (`paint:<name>#<version>`).
pub const PAINT_PREFIX: &str = "paint:";
/// Prefix of the layer atlas keys (`splat:<a>|<b>|<c>|<d>`).
pub const SPLAT_PREFIX: &str = "splat:";

/// Texture key of a mask at its current version (changes with every edit so the GPU copy refreshes).
pub fn paint_key(name: &str) -> String {
    format!("{PAINT_PREFIX}{name}#{}", PaintMaps::global().version(name))
}

/// Keeps the material of every painted object in sync with its component and mask.
pub fn update(world: &mut kvas_ecs::World, assets: Option<&Assets>) {
    for e in world.entities_with::<TexturePaint>() {
        let Some(tp) = world.get::<TexturePaint>(e).cloned() else { continue };
        let name = tp.mask_name(&world.get::<Name>(e).map(|n| n.0.clone()).unwrap_or_else(|| "mask".into()));
        let maps = PaintMaps::global();
        if !maps.exists(&name) {
            match assets {
                Some(a) => maps.load_or_create(&name, a, tp.resolution.clamp(64, 2048) as u32),
                None => maps.create(&name, tp.resolution.clamp(64, 2048) as u32, tp.resolution.clamp(64, 2048) as u32),
            }
        }
        if !world.has::<MeshRenderer>(e) {
            world.insert(e, MeshRenderer::default());
        }
        let key = paint_key(&name);
        let atlas = format!("{SPLAT_PREFIX}{}|{}|{}|{}", tp.layer1, tp.layer2, tp.layer3, tp.layer4);
        let m = world.get_mut::<MeshRenderer>(e).unwrap();
        if m.texture != key {
            m.texture = key;
        }
        if m.normal_map != atlas {
            m.normal_map = atlas;
        }
        m.shader = "builtin:splat".into();
        m.tiling = tp.tiling;
        m.user[0] = tp.breakup;
        m.cast_shadows = false;
    }
}
