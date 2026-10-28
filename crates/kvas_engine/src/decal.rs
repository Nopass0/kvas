//! Projected decals: a box that paints whatever surface lies inside it (any shape of geometry,
//! no UVs needed). The box projects along its local -Y axis, so its +Y should point out of the
//! surface; its X and Z scale give the size, its Y scale the depth it reaches.
//!
//! A decal can carry an image, a normal map and an ORM map, or draw a built-in procedural shape,
//! and blends with the surface in one of several modes (normal, multiply, lighten, overlay...).
//! Decals are drawn after the opaque scene by `builtin:decal`, sorted by `order`, culled by the
//! view frustum and by `max_distance`, and grouped by image so thousands stay cheap.
//!
//! ```
//! use kvas_engine::decal::{Decal, DecalBlend};
//! let d = Decal { blend: DecalBlend::Multiply, opacity: 0.8, ..Default::default() };
//! assert_eq!(d.shader_params()[0], DecalBlend::Multiply as i32 as f32);
//! ```

use crate::components::Component;
use crate::reflect::{AssetKind, Field};
use serde::{Deserialize, Serialize};

crate::choice_enum! {
    /// How a decal combines with the surface under it.
    DecalBlend { Normal, Multiply, Screen, Lighten, Darken, Overlay, Add, SoftLight, ReliefOnly, ColorDodge, ColorBurn, Tint } default Normal
}

crate::choice_enum! {
    /// Built-in shapes for decals without an image.
    DecalShape { Image, Blob, Scorch, BulletHole, Cracks, Puddle, Grunge, PaintSplat, Moss, Rust, Leaks, Ring } default Image
}

/// A projected decal. Put it on an entity and move, turn and scale it like any object.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Decal {
    /// Image (its alpha cuts the shape). Empty: the procedural `shape` is drawn.
    pub texture: String,
    /// Optional tangent-space normal map (relief of the decal).
    pub normal_map: String,
    /// Optional occlusion / roughness / metallic map.
    pub orm_map: String,
    /// Procedural shape used when there is no image.
    pub shape: DecalShape,
    /// How the decal combines with the surface.
    pub blend: DecalBlend,
    /// Tint (multiplied with the image), linear RGBA.
    pub color: [f32; 4],
    /// Overall strength 0..1.
    pub opacity: f32,
    /// Surface roughness under the decal.
    pub roughness: f32,
    /// Metalness under the decal.
    pub metallic: f32,
    /// Glow added on top (lights up in the dark).
    pub emission: f32,
    /// Strength of the normal map or of the shape's relief.
    pub normal_strength: f32,
    /// Fade on surfaces turned away from the projection, 0 = paint everything, 1 = only faces
    /// looking straight along the decal.
    pub angle_fade: f32,
    /// Random variation of the procedural shapes.
    pub seed: i32,
    /// Higher numbers draw on top of lower ones.
    pub order: i32,
    /// Not drawn beyond this distance from the camera (0 = always); saves time with many decals.
    pub max_distance: f32,
}

impl Default for Decal {
    fn default() -> Self {
        Self {
            texture: String::new(),
            normal_map: String::new(),
            orm_map: String::new(),
            shape: DecalShape::Blob,
            blend: DecalBlend::Normal,
            color: [0.35, 0.05, 0.04, 1.0],
            opacity: 1.0,
            roughness: 0.6,
            metallic: 0.0,
            emission: 0.0,
            normal_strength: 1.0,
            angle_fade: 0.3,
            seed: 1,
            order: 0,
            max_distance: 80.0,
        }
    }
}

impl Decal {
    /// The eight shader numbers of the decal (`object.user0`, `object.user1` of `builtin:decal`).
    pub fn shader_params(&self) -> [f32; 8] {
        [
            self.blend as i32 as f32,
            self.shape as i32 as f32,
            self.angle_fade.clamp(0.0, 1.0),
            self.normal_strength,
            self.seed as f32,
            self.emission,
            (!self.normal_map.is_empty()) as i32 as f32,
            (!self.orm_map.is_empty()) as i32 as f32,
        ]
    }
}

impl Component for Decal {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::asset("texture", &mut self.texture, AssetKind::Texture),
            Field::asset("normal_map", &mut self.normal_map, AssetKind::Texture),
            Field::asset("orm_map", &mut self.orm_map, AssetKind::Texture),
            Field::choice("shape", &mut self.shape),
            Field::choice("blend", &mut self.blend),
            Field::color("color", &mut self.color),
            Field::f32("opacity", &mut self.opacity),
            Field::f32("roughness", &mut self.roughness),
            Field::f32("metallic", &mut self.metallic),
            Field::f32("emission", &mut self.emission),
            Field::f32("normal_strength", &mut self.normal_strength),
            Field::f32("angle_fade", &mut self.angle_fade),
            Field::i32("seed", &mut self.seed),
            Field::i32("order", &mut self.order),
            Field::f32("max_distance", &mut self.max_distance),
        ]
    }
}

/// A ready look for common uses: `"blood"`, `"scorch"`, `"bullet_hole"`, `"cracks"`, `"puddle"`,
/// `"dirt"`, `"paint"`, `"moss"`, `"rust"`, `"leaks"`, `"graffiti_glow"`, `"light_spot"`.
pub fn preset(name: &str) -> Decal {
    let d = Decal::default();
    match name {
        "scorch" => Decal { shape: DecalShape::Scorch, blend: DecalBlend::Multiply, color: [0.05, 0.04, 0.035, 1.0], roughness: 0.95, ..d },
        "bullet_hole" => Decal { shape: DecalShape::BulletHole, color: [0.06, 0.055, 0.05, 1.0], normal_strength: 2.0, roughness: 0.9, ..d },
        "cracks" => Decal { shape: DecalShape::Cracks, blend: DecalBlend::Multiply, color: [0.12, 0.11, 0.1, 1.0], normal_strength: 1.5, ..d },
        "puddle" => Decal { shape: DecalShape::Puddle, color: [0.25, 0.24, 0.22, 1.0], roughness: 0.04, opacity: 0.85, angle_fade: 0.7, ..d },
        "dirt" => Decal { shape: DecalShape::Grunge, blend: DecalBlend::Multiply, color: [0.45, 0.38, 0.28, 1.0], opacity: 0.8, ..d },
        "paint" => Decal { shape: DecalShape::PaintSplat, color: [0.9, 0.15, 0.4, 1.0], roughness: 0.35, ..d },
        "moss" => Decal { shape: DecalShape::Moss, color: [0.1, 0.25, 0.05, 1.0], roughness: 0.95, normal_strength: 1.2, ..d },
        "rust" => Decal { shape: DecalShape::Rust, color: [0.45, 0.18, 0.06, 1.0], roughness: 0.9, ..d },
        "leaks" => Decal { shape: DecalShape::Leaks, blend: DecalBlend::Multiply, color: [0.35, 0.33, 0.3, 1.0], angle_fade: 0.0, ..d },
        "graffiti_glow" => Decal { shape: DecalShape::Ring, color: [0.2, 0.9, 1.0, 1.0], emission: 3.0, ..d },
        "light_spot" => Decal { shape: DecalShape::Blob, blend: DecalBlend::Lighten, color: [1.0, 0.85, 0.6, 1.0], opacity: 0.6, ..d },
        _ => d,
    }
}

/// Names accepted by [`preset`].
pub const PRESETS: &[&str] = &["blood", "scorch", "bullet_hole", "cracks", "puddle", "dirt", "paint", "moss", "rust", "leaks", "graffiti_glow", "light_spot"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_and_params_round_trip() {
        for name in PRESETS {
            let d = preset(name);
            let json = serde_json::to_string(&d).unwrap();
            let back: Decal = serde_json::from_str(&json).unwrap();
            assert_eq!(back.shader_params(), d.shader_params(), "{name}");
        }
        let d = Decal { normal_map: "n.png".into(), ..Default::default() };
        assert_eq!(d.shader_params()[6], 1.0);
    }
}
