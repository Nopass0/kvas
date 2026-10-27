//! Per-scene shadow and post-processing settings. Plain data: the renderer reads them every
//! frame, scripts change them through [`crate::script::Ctx`] (`ctx.post()`, `ctx.shadows()`).
//!
//! ```
//! use kvas_engine::fx::PostSettings;
//! let mut post = PostSettings::default();
//! post.bloom = 0.6;
//! assert!(post.any());
//! ```

use serde::{Deserialize, Serialize};

crate::choice_enum!(
    /// How a reduced render scale is brought back to the window size.
    Upscaler { Bilinear, Fsr, Dlss } default Fsr
);

/// Sun shadow quality and look.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct ShadowSettings {
    /// Master switch for sun shadows.
    pub enabled: bool,
    /// Edge softness: 0.5 crisp, 1 normal, 3+ very soft (more filter taps are used automatically).
    pub softness: f32,
    /// How dark shadows get, 0 = invisible .. 1 = fully dark.
    pub strength: f32,
    /// Depth bias; raise it if surfaces show striping ("shadow acne").
    pub bias: f32,
    /// Half-width in metres of the area around the camera that receives shadows.
    pub range: f32,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self { enabled: true, softness: 1.0, strength: 1.0, bias: 0.0004, range: 40.0 }
    }
}

/// A custom full-screen shader applied to the camera image (see `shaders/fx` in a project).
///
/// The WGSL file defines `fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32>`
/// and may call `scene_color(uv)` to sample the picture elsewhere. `params` carries the four numbers below.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct PostEffect {
    /// Asset path of the WGSL file.
    pub shader: String,
    /// Switch an effect off without removing it.
    pub enabled: bool,
    /// Free numbers handed to the shader as `params`.
    pub params: [f32; 4],
}

impl Default for PostEffect {
    fn default() -> Self {
        Self { shader: String::new(), enabled: true, params: [1.0, 0.0, 0.0, 0.0] }
    }
}

/// Screen-space effects applied after the scene is drawn. 0 turns an effect off.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct PostSettings {
    /// Glow around bright areas.
    pub bloom: f32,
    /// Brightness above which bloom starts (0..2).
    pub bloom_threshold: f32,
    /// Size of the bloom halo, 0.5..4.
    pub bloom_radius: f32,
    /// Lens flare: ghost images and streaks from very bright lights and the sun.
    pub lens_flare: f32,
    /// Full-screen gaussian-style blur strength (menus, underwater, damage).
    pub blur: f32,
    /// Blur along camera motion; follows how fast the camera turns and moves.
    pub motion_blur: f32,
    /// Darkening towards the screen corners.
    pub vignette: f32,
    /// Colour fringing towards the corners (cheap lens look).
    pub chromatic_aberration: f32,
    /// Animated film grain.
    pub film_grain: f32,
    /// Sharpening of fine detail.
    pub sharpen: f32,
    /// Tint multiplied over the final picture (linear RGB), white = none.
    pub tint: [f32; 3],
    /// Custom shader effects, applied in order after the built-in ones.
    pub effects: Vec<PostEffect>,
}

impl Default for PostSettings {
    fn default() -> Self {
        Self {
            bloom: 0.0,
            bloom_threshold: 0.8,
            bloom_radius: 1.5,
            lens_flare: 0.0,
            blur: 0.0,
            motion_blur: 0.0,
            vignette: 0.0,
            chromatic_aberration: 0.0,
            film_grain: 0.0,
            sharpen: 0.0,
            tint: [1.0; 3],
            effects: Vec::new(),
        }
    }
}

impl PostSettings {
    /// Whether any custom shader effect is switched on.
    pub fn any_custom(&self) -> bool {
        self.effects.iter().any(|e| e.enabled && !e.shader.is_empty())
    }

    /// Whether any effect needs the extra render pass.
    pub fn any(&self) -> bool {
        self.any_builtin() || self.any_custom()
    }

    /// Whether any built-in effect (bloom, blur, tint...) is on.
    pub fn any_builtin(&self) -> bool {
        self.bloom > 0.0
            || self.lens_flare > 0.0
            || self.blur > 0.0
            || self.motion_blur > 0.0
            || self.vignette > 0.0
            || self.chromatic_aberration > 0.0
            || self.film_grain > 0.0
            || self.sharpen > 0.0
            || self.tint != [1.0; 3]
    }

    /// Ready-made looks for the editor's preset buttons and for scripts.
    pub fn preset(name: &str) -> Self {
        match name {
            "cinematic" => Self { bloom: 0.45, bloom_threshold: 0.75, lens_flare: 0.35, vignette: 0.35, chromatic_aberration: 0.25, film_grain: 0.12, motion_blur: 0.4, ..Default::default() },
            "dreamy" => Self { bloom: 1.0, bloom_threshold: 0.55, bloom_radius: 2.6, blur: 0.12, vignette: 0.2, ..Default::default() },
            "crisp" => Self { sharpen: 0.6, bloom: 0.15, vignette: 0.1, ..Default::default() },
            "retro" => Self { chromatic_aberration: 0.6, film_grain: 0.3, vignette: 0.5, tint: [1.0, 0.95, 0.82], ..Default::default() },
            _ => Self::default(),
        }
    }
}
