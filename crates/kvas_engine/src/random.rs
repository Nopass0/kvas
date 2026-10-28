//! A tiny deterministic random generator for scripts (xorshift64*), so gameplay does not need
//! an external crate. Keep one [`Rng`] in your script struct for a repeatable stream.
//!
//! ```
//! use kvas_engine::random::Rng;
//! let mut rng = Rng::new(42);
//! let a = rng.f32();
//! assert!((0.0..1.0).contains(&a));
//! assert_eq!(Rng::new(42).f32(), a, "same seed, same stream");
//! assert!((5..10).contains(&rng.range_i32(5, 10)));
//! ```

use glam::Vec3;

/// Deterministic pseudo-random numbers. Not suitable for cryptography.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(0x9E37_79B9_7F4A_7C15)
    }
}

impl Rng {
    /// A generator for the given seed (any value, zero included).
    pub fn new(seed: u64) -> Self {
        // SplitMix64 scramble so nearby seeds give unrelated streams.
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Self { state: (z ^ (z >> 31)) | 1 }
    }

    /// Next raw 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `0.0..1.0`.
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in `min..max` (swapped bounds are accepted).
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.f32()
    }

    /// Uniform integer in `min..max` (exclusive end); returns `min` for an empty range.
    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next_u64() % (max as i64 - min as i64) as u64) as i32
    }

    /// True with probability `p` (0..1).
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    /// A random element, or `None` for an empty slice.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            items.get(self.range_i32(0, items.len() as i32) as usize)
        }
    }

    /// A uniformly distributed unit vector.
    pub fn unit_vec3(&mut self) -> Vec3 {
        let z = self.range(-1.0, 1.0);
        let a = self.range(0.0, std::f32::consts::TAU);
        let r = (1.0 - z * z).sqrt();
        Vec3::new(r * a.cos(), z, r * a.sin())
    }

    /// A uniformly distributed point inside a sphere of the given radius.
    pub fn in_sphere(&mut self, radius: f32) -> Vec3 {
        self.unit_vec3() * radius * self.f32().cbrt()
    }

    /// A random point on a horizontal circle (XZ plane) of the given radius.
    pub fn on_circle(&mut self, radius: f32) -> Vec3 {
        let a = self.range(0.0, std::f32::consts::TAU);
        Vec3::new(a.cos(), 0.0, a.sin()) * radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_are_distinct_and_in_range() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
        for _ in 0..1000 {
            assert!((0.0..1.0).contains(&a.f32()));
            assert!((-3..3).contains(&a.range_i32(-3, 3)));
            assert!(a.in_sphere(2.0).length() <= 2.0001);
        }
        assert_eq!(a.range_i32(4, 4), 4);
        assert!(a.pick::<i32>(&[]).is_none());
    }
}
