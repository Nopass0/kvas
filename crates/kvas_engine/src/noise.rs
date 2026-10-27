//! Procedural noise for scripts, shaders-on-the-CPU and generated textures.
//!
//! Everything is a pure function of its inputs (no global state), so it is safe to call from
//! script libraries and always deterministic for a given `seed`.
//!
//! ```
//! use kvas_engine::noise::{Noise, NoiseKind};
//! let (x, z) = (1.0, 2.0);
//! let n = Noise { kind: NoiseKind::Ridged, frequency: 0.8, octaves: 5, ..Default::default() };
//! let height = n.sample2(x, z);          // -1..1
//! let mask = n.sample01(x, z, 0.0);      // 0..1
//! ```

use crate::choice_enum;
use glam::{Vec2, Vec3};

choice_enum!(
    /// Basic noise flavours; all can be combined with fractal octaves and domain warping.
    NoiseKind { Value, Perlin, Worley, Ridged, Billow, Cellular } default Perlin
);

fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn lattice(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    hash(hash(hash(x as u32 ^ seed.wrapping_mul(0x9e37_79b1)).wrapping_add(y as u32)).wrapping_add(z as u32))
}

fn unit(h: u32) -> f32 {
    (h >> 8) as f32 / 16_777_216.0
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smooth value noise, -1..1.
pub fn value3(p: Vec3, seed: u32) -> f32 {
    if !p.is_finite() {
        return 0.0;
    }
    let (i, f) = (p.floor(), p - p.floor());
    let (x, y, z) = (i.x as i32, i.y as i32, i.z as i32);
    let (u, v, w) = (fade(f.x), fade(f.y), fade(f.z));
    let c = |dx: i32, dy: i32, dz: i32| unit(lattice(x.wrapping_add(dx), y.wrapping_add(dy), z.wrapping_add(dz), seed)) * 2.0 - 1.0;
    lerp(
        lerp(lerp(c(0, 0, 0), c(1, 0, 0), u), lerp(c(0, 1, 0), c(1, 1, 0), u), v),
        lerp(lerp(c(0, 0, 1), c(1, 0, 1), u), lerp(c(0, 1, 1), c(1, 1, 1), u), v),
        w,
    )
}

fn gradient(h: u32) -> Vec3 {
    // 12 edge directions of a cube, as in Perlin's improved noise.
    const G: [Vec3; 12] = [
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(-1.0, 1.0, 0.0),
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(-1.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, -1.0),
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 1.0),
        Vec3::new(0.0, -1.0, 1.0),
        Vec3::new(0.0, 1.0, -1.0),
        Vec3::new(0.0, -1.0, -1.0),
    ];
    G[(h % 12) as usize]
}

/// Classic gradient (Perlin) noise, roughly -1..1.
pub fn perlin3(p: Vec3, seed: u32) -> f32 {
    if !p.is_finite() {
        return 0.0;
    }
    let (i, f) = (p.floor(), p - p.floor());
    let (x, y, z) = (i.x as i32, i.y as i32, i.z as i32);
    let (u, v, w) = (fade(f.x), fade(f.y), fade(f.z));
    let d = |dx: i32, dy: i32, dz: i32| {
        gradient(lattice(x.wrapping_add(dx), y.wrapping_add(dy), z.wrapping_add(dz), seed))
            .dot(f - Vec3::new(dx as f32, dy as f32, dz as f32))
    };
    lerp(
        lerp(lerp(d(0, 0, 0), d(1, 0, 0), u), lerp(d(0, 1, 0), d(1, 1, 0), u), v),
        lerp(lerp(d(0, 0, 1), d(1, 0, 1), u), lerp(d(0, 1, 1), d(1, 1, 1), u), v),
        w,
    ) * 0.9
}

/// Distances to the nearest and second nearest feature point (Worley / cellular noise).
pub fn worley3(p: Vec3, seed: u32) -> (f32, f32) {
    if !p.is_finite() {
        return (0.0, 0.0);
    }
    let (i, f) = (p.floor(), p - p.floor());
    let (mut f1, mut f2) = (8.0f32, 8.0f32);
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (x, y, z) = ((i.x as i32).wrapping_add(dx), (i.y as i32).wrapping_add(dy), (i.z as i32).wrapping_add(dz));
                let point =
                    Vec3::new(unit(lattice(x, y, z, seed)), unit(lattice(x, y, z, seed ^ 0x51ed)), unit(lattice(x, y, z, seed ^ 0xa2b3)));
                let d = (Vec3::new(dx as f32, dy as f32, dz as f32) + point - f).length();
                if d < f1 {
                    f2 = f1;
                    f1 = d;
                } else if d < f2 {
                    f2 = d;
                }
            }
        }
    }
    (f1, f2)
}

/// Full-featured fractal noise. `sample*` return -1..1, `sample01*` return 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Noise {
    /// Base noise family used for each octave.
    pub kind: NoiseKind,
    /// Deterministic lattice seed.
    pub seed: u32,
    /// Features per world unit.
    pub frequency: f32,
    /// Number of layered octaves (1 = plain noise).
    pub octaves: u32,
    /// Frequency multiplier between octaves.
    pub lacunarity: f32,
    /// Amplitude multiplier between octaves.
    pub gain: f32,
    /// Domain warp strength (0 = off): bends the lookup position with more noise.
    pub warp: f32,
}

impl Default for Noise {
    fn default() -> Self {
        Self { kind: NoiseKind::Perlin, seed: 0, frequency: 1.0, octaves: 4, lacunarity: 2.0, gain: 0.5, warp: 0.0 }
    }
}

impl Noise {
    /// Creates fractal noise of the chosen family with default sampling parameters.
    pub fn new(kind: NoiseKind) -> Self {
        Self { kind, ..Default::default() }
    }
    /// Builder: changes the deterministic lattice seed.
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
    /// Builder: sets features per input unit.
    pub fn with_frequency(mut self, f: f32) -> Self {
        self.frequency = f;
        self
    }
    /// Builder: sets the octave count; sampling clamps it to a bounded range.
    pub fn with_octaves(mut self, o: u32) -> Self {
        self.octaves = o;
        self
    }
    /// Builder: sets domain-warp strength; zero disables it.
    pub fn with_warp(mut self, w: f32) -> Self {
        self.warp = w;
        self
    }

    fn single(&self, p: Vec3, seed: u32) -> f32 {
        match self.kind {
            NoiseKind::Value => value3(p, seed),
            NoiseKind::Perlin => perlin3(p, seed),
            NoiseKind::Worley => {
                let (f1, _) = worley3(p, seed);
                1.0 - 2.0 * f1.min(1.0)
            }
            NoiseKind::Cellular => {
                let (f1, f2) = worley3(p, seed);
                ((f2 - f1) * 2.0).min(1.0) * 2.0 - 1.0
            }
            NoiseKind::Ridged => 1.0 - 2.0 * perlin3(p, seed).abs().min(1.0),
            NoiseKind::Billow => 2.0 * perlin3(p, seed).abs().min(1.0) - 1.0,
        }
    }

    fn fractal(&self, p: Vec3, seed: u32) -> f32 {
        let (mut sum, mut amp, mut norm, mut q) = (0.0, 1.0, 0.0, p);
        for o in 0..self.octaves.clamp(1, 32) {
            sum += amp * self.single(q, seed.wrapping_add(o * 101));
            norm += amp;
            amp *= self.gain;
            q *= self.lacunarity;
        }
        sum / norm
    }

    /// Samples three-dimensional fractal noise, nominally in -1..1.
    /// Octaves are bounded to 1..32; non-finite inputs or results return zero.
    pub fn sample3(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut p = Vec3::new(x, y, z) * self.frequency;
        if self.warp > 0.0 {
            let offset = Vec3::new(
                self.fractal(p, self.seed ^ 0x1111),
                self.fractal(p + Vec3::splat(17.3), self.seed ^ 0x2222),
                self.fractal(p + Vec3::splat(41.7), self.seed ^ 0x3333),
            );
            p += offset * self.warp;
        }
        let sample = self.fractal(p, self.seed);
        if sample.is_finite() {
            sample.clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    /// Samples the XY plane of the three-dimensional noise at Z = 0.
    pub fn sample2(&self, x: f32, y: f32) -> f32 {
        self.sample3(x, y, 0.0)
    }

    /// Samples noise remapped and clamped into 0..1.
    pub fn sample01(&self, x: f32, y: f32, z: f32) -> f32 {
        self.sample3(x, y, z) * 0.5 + 0.5
    }

    /// Divergence-free 2D flow field (for swirling particles, smoke, water).
    pub fn curl2(&self, x: f32, y: f32) -> Vec2 {
        let e = 0.01 / self.frequency.max(1e-3);
        let dx = (self.sample2(x + e, y) - self.sample2(x - e, y)) / (2.0 * e);
        let dy = (self.sample2(x, y + e) - self.sample2(x, y - e)) / (2.0 * e);
        Vec2::new(dy, -dx)
    }

    /// RGBA8 image with two colours blended by the noise value (a at 0, b at 1).
    /// Returns an empty image on invalid dimensions. Use [`Self::try_image`] for errors.
    pub fn image(&self, width: u32, height: u32, a: [f32; 3], b: [f32; 3]) -> Vec<u8> {
        self.try_image(width, height, a, b).unwrap_or_default()
    }

    /// Generates RGBA8 pixels with checked dimensions and a 16-million-pixel budget.
    /// Rejects zero sizes, non-finite colours and allocation-size overflow.
    pub fn try_image(&self, width: u32, height: u32, a: [f32; 3], b: [f32; 3]) -> Result<Vec<u8>, String> {
        let pixels = (width as usize).checked_mul(height as usize).ok_or("image dimensions overflow")?;
        if pixels == 0 || pixels > 16_777_216 || a.iter().chain(b.iter()).any(|c| !c.is_finite()) {
            return Err("noise image requires finite colours and 1..16777216 pixels".into());
        }
        let mut out = Vec::with_capacity(pixels * 4);
        for y in 0..height {
            for x in 0..width {
                let t = self.sample01(x as f32 / width as f32, y as f32 / height as f32, 0.0);
                for c in 0..3 {
                    out.push(((lerp(a[c], b[c], t)).clamp(0.0, 1.0) * 255.0) as u8);
                }
                out.push(255);
            }
        }
        Ok(out)
    }
}

/// Prefix for serialized procedural noise texture keys.
pub const TEXTURE_PREFIX: &str = "noise:";

/// A texture description that fits in `MeshRenderer.texture`, e.g. from a script:
/// `mr.texture = NoiseTexture::new(Noise::new(NoiseKind::Worley)).key();`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoiseTexture {
    /// Fractal noise parameters sampled across the generated square.
    pub noise: Noise,
    /// Square output image edge length in pixels.
    pub size: u32,
    /// RGB colour at noise value zero, normally in 0..1.
    pub dark: [f32; 3],
    /// RGB colour at noise value one, normally in 0..1.
    pub light: [f32; 3],
}

impl NoiseTexture {
    /// Creates a 256-pixel grayscale texture descriptor for this noise.
    pub fn new(noise: Noise) -> Self {
        Self { noise, size: 256, dark: [0.05, 0.05, 0.06], light: [0.95, 0.95, 0.97] }
    }

    /// Builder: replaces the low/high RGB colours.
    pub fn colors(mut self, dark: [f32; 3], light: [f32; 3]) -> Self {
        self.dark = dark;
        self.light = light;
        self
    }

    /// Encodes the complete descriptor as a procedural texture cache key.
    pub fn key(&self) -> String {
        use crate::reflect::Choice;
        let n = &self.noise;
        format!(
            "{TEXTURE_PREFIX}{}:{},{},{},{},{},{},{},{},{},{},{},{},{}",
            NoiseKind::OPTIONS[n.kind.to_index()],
            n.seed,
            n.frequency,
            n.octaves,
            n.lacunarity,
            n.gain,
            n.warp,
            self.size,
            self.dark[0],
            self.dark[1],
            self.dark[2],
            self.light[0],
            self.light[1],
            self.light[2]
        )
    }

    /// Parses a procedural texture key; malformed input returns None.
    pub fn parse(key: &str) -> Option<NoiseTexture> {
        use crate::reflect::Choice;
        let (kind, rest) = key.strip_prefix(TEXTURE_PREFIX)?.split_once(':')?;
        let kind = NoiseKind::OPTIONS.iter().position(|o| *o == kind).map(NoiseKind::from_index)?;
        let n: Vec<&str> = rest.split(',').collect();
        if n.len() != 13 {
            return None;
        }
        let real = |i: usize| n[i].parse::<f32>().ok().filter(|v| v.is_finite());
        Some(NoiseTexture {
            noise: Noise {
                kind,
                seed: n[0].parse().ok()?,
                frequency: real(1)?,
                octaves: n[2].parse().ok()?,
                lacunarity: real(3)?,
                gain: real(4)?,
                warp: real(5)?,
            },
            size: n[6].parse::<u32>().ok()?.clamp(8, 2048),
            dark: [real(7)?, real(8)?, real(9)?],
            light: [real(10)?, real(11)?, real(12)?],
        })
    }

    /// Generates tightly packed RGBA8 pixels in row-major order, clamping size to 8..2048.
    pub fn rgba(&self) -> Vec<u8> {
        let size = self.size.clamp(8, 2048);
        self.noise.image(size, size, self.dark, self.light)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_in_range() {
        for kind in [NoiseKind::Value, NoiseKind::Perlin, NoiseKind::Worley, NoiseKind::Ridged, NoiseKind::Billow, NoiseKind::Cellular] {
            let n = Noise { kind, octaves: 3, warp: 0.4, ..Default::default() }.with_seed(7);
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for i in 0..2000 {
                let (x, y) = ((i % 50) as f32 * 0.37, (i / 50) as f32 * 0.41);
                let v = n.sample2(x, y);
                assert!((-1.0..=1.0).contains(&v), "{kind:?} out of range: {v}");
                assert_eq!(v, n.sample2(x, y), "{kind:?} not deterministic");
                lo = lo.min(v);
                hi = hi.max(v);
            }
            assert!(hi - lo > 0.3, "{kind:?} is nearly constant ({lo}..{hi})");
        }
    }

    #[test]
    fn different_seeds_differ_and_noise_is_continuous() {
        let a = Noise::default().with_seed(1).sample2(3.3, 4.4);
        let b = Noise::default().with_seed(2).sample2(3.3, 4.4);
        assert_ne!(a, b);
        let n = Noise::default();
        assert!((n.sample2(1.0, 1.0) - n.sample2(1.0005, 1.0)).abs() < 0.01);
    }

    #[test]
    fn texture_key_roundtrip() {
        let t = NoiseTexture::new(Noise::new(NoiseKind::Worley).with_seed(5).with_frequency(8.0)).colors([0.1, 0.2, 0.3], [0.9, 0.8, 0.7]);
        assert_eq!(NoiseTexture::parse(&t.key()), Some(t));
        assert_eq!(t.rgba().len(), (t.size * t.size * 4) as usize);
    }
}
