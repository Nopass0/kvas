//! Systems that build or simulate procedural geometry (cloth, voxel smoke and clouds, grass).
//! They write a `DynamicMesh` that the renderer uploads.

use kvas_ecs::World;
use serde::Serialize;

/// Stable hash of a component's serialized parameters; used to detect edits that need a rebuild.
pub fn signature<T: Serialize>(value: &T) -> u64 {
    serde_json::to_string(value)
        .unwrap_or_default()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

/// Tiny deterministic random generator (xorshift64*).
#[derive(Clone)]
pub struct Rng(u64);

impl Rng {
    /// Creates a nonzero generator state from any seed, including zero.
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
    }

    /// Returns the next deterministic 64-bit pseudorandom value.
    /// This generator is for procedural content, not cryptographic use.
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// Uniform in 0..1.
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / 16_777_216.0
    }
}

/// Runs every procedural system. `simulate` is false in the editor outside Play mode: simulations
/// then only show their initial state.
pub fn update(world: &mut World, time: f32, dt: f32, simulate: bool) {
    crate::meshmod::update(world);
    crate::paint::update(world, crate::meshmod::assets().as_ref());
    crate::grass::update(world);
    crate::tree::update(world);
    crate::voxel::update_clouds(world, dt, simulate);
    crate::voxel::update_smoke(world, dt, simulate);
    crate::cloth::update(world, time, dt, simulate);
    crate::particles::update(world, time, dt, simulate);
    crate::water::update_water(world, time);
    crate::mesh_edit::update(world);
}
