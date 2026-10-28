//! Kvas gameplay engine: scenes, physics, procedural geometry and hot-reloadable scripts.
//!
//! # Start here
//! - [`prelude`] contains the common script-facing types.
//! - [`scene::Scene`] owns registered ECS components and environment settings.
//! - [`runtime::Runtime`] runs the same simulation in the editor and the player.
//! - [`script::Script`] defines gameplay callbacks; [`script::Ctx`] accesses host state.
//! - [`assets::Assets`] reads loose project assets or a packed game archive.
//!
//! # Units and lifecycle
//! Positions and collider sizes use metres; time uses seconds; [`components::Transform`]
//! stores Euler angles in degrees, in Y-X-Z order. The world is right-handed with
//! +Y up and local -Z forward. Refresh cached world matrices with
//! [`hierarchy::update_globals`] after editing transforms outside the runtime.
//!
//! # Native scripts
//! Host and script libraries must use identical engine sources, dependencies and
//! compiler. Rebuild scripts after changing the engine. Script callbacks share
//! host-owned built-in components through `Ctx`; creating custom ECS storage in
//! an unloadable library would leave dangling vtables after hot reload.
//!
//! ```
//! use kvas_engine::{prelude::*, scene::Scene, runtime::Runtime};
//! let mut scene = Scene::default();
//! let cube = scene.spawn("Spinning cube");
//! scene.world.insert(cube, MeshRenderer::default());
//! scene.world.insert(cube, Spin { speed: Vec3::new(0.0, 90.0, 0.0) });
//! let mut runtime = Runtime::new(scene, None);
//! runtime.tick(1.0 / 60.0);
//! assert!(runtime.scene.world.get::<Transform>(cube).unwrap().rotation.y > 0.0);
//! ```
#![warn(missing_docs)]

pub mod assets;
pub mod animation;
pub mod audio;
pub mod cloth;
pub mod components;
pub mod events;
pub mod fx;
mod file_io;
pub mod gameplay;
pub mod grass;
pub mod gui;
pub mod hierarchy;
pub mod input;
pub mod interaction;
pub mod material_graph;
pub mod material_nodes;
pub mod material_nodes_ext;
pub mod material_presets;
pub mod materials;
pub mod math;
pub mod mesh;
pub mod mesh_deform;
pub mod mesh_edit;
pub mod mesh_uv;
pub mod meshmod;
pub mod modifiers;
pub mod noise;
pub mod neural;
pub mod paint;
pub mod params;
pub mod particles;
pub mod physics;
pub mod procedural;
pub mod project;
pub mod random;
pub mod reflect;
pub mod registry;
pub mod runtime;
pub mod scene;
pub mod script;
mod script_ext;
pub mod signals;
pub mod sky;
pub mod spatial;
pub mod timer;
pub mod tween;
pub mod voxel;
mod voxel_mesh;
pub mod water;

#[doc = include_str!("../../../docs/ENGINE_API.md")]
pub mod guide {}

/// Everything a script author needs.
pub mod prelude {
    pub use crate::audio::{AudioBus, AudioSource, SoundId};
    pub use crate::cloth::{Cloth, ClothPin};
    pub use crate::components::*;
    pub use crate::events::EventQueue;
    pub use crate::fx::{PostEffect, PostSettings, ShadowSettings, Upscaler};
    pub use crate::gameplay::{Lifetime, Tags, Visibility};
    pub use crate::grass::Grass;
    pub use crate::hierarchy;
    pub use crate::input::Hud;
    pub use crate::math::{approach, approach_degrees, damp, ease_smooth, ease_smoother, smooth_damp, delta_degrees, inverse_lerp, lerp, remap, remap_clamped, smoothstep, wrap_degrees};
    pub use crate::random::Rng;
    pub use crate::paint::TexturePaint;
    pub use crate::particles::{EmitterShape, ParticleEmitter};
    pub use crate::params::{Color, EntityRef, Local, MeshRef, Param, SoundRef, TextureRef};
    pub use crate::{fields, script, script_params};
    pub use crate::input::{Input, Key, MouseButton};
    pub use crate::interaction::{press, Slider, Switch, SwitchKind};
    pub use crate::meshmod::{MeshModifiers, ModAxis, Modifier};
    pub use crate::modifiers::{hsv_to_rgb, Bob, ColorCycle, Pulse, Spin, Wobble};
    pub use crate::noise::{Noise, NoiseKind, NoiseTexture};
    pub use crate::physics::{QueryOptions, RayHit};
    pub use crate::reflect::Field;
    pub use crate::register_scripts;
    pub use crate::script::{Ctx, Script};
    pub use crate::signals::{Signal, SignalBus, SignalError};
    pub use crate::spatial::{Aabb, Frustum, Ray};
    pub use crate::timer::{Timer, TimerMode};
    pub use crate::tween::{Ease, MoveTween, TweenMode};
    pub use crate::voxel::{VoxelClouds, VoxelSmoke};
    pub use crate::water::{VoxelWater, WaterStyle};
    pub use glam::{Mat4, Quat, Vec2, Vec3};
    pub use serde_json::json;
    pub use kvas_ecs::{Entity, World};
}
