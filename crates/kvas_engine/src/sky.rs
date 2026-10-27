//! Procedural atmosphere: time of day drives the sun/moon direction, light colour and ambient.
//! The renderer draws the sky itself from the same settings, so light and sky always agree.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Serializable procedural sky, celestial lighting and day/night animation controls.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct SkySettings {
    /// Draw the procedural sky (and light the scene from it) instead of a flat colour.
    pub procedural: bool,
    /// Hour of day, 0..24 (12 = noon).
    pub time: f32,
    /// Game hours that pass per real second while playing (0 = frozen, 1 = a day in 24 s).
    pub speed: f32,
    /// 0 = clear, 1 = overcast.
    pub clouds: f32,
    /// Cloud pattern scrolling speed used by the sky shader.
    pub cloud_speed: f32,
    /// Star brightness at night, 0..1.
    pub stars: f32,
    /// Air haze, 0 (crystal clear) .. 1 (hazy, milky horizon).
    pub haze: f32,
    /// Moon phase: 0 new, 0.5 full, 1 new again.
    pub moon_phase: f32,
    /// Northern-lights strength at night, 0..1.
    pub aurora: f32,
    /// Milky way brightness at night, 0..1.
    pub milky_way: f32,
}

impl Default for SkySettings {
    fn default() -> Self {
        Self {
            procedural: false,
            time: 14.0,
            speed: 0.0,
            clouds: 0.45,
            cloud_speed: 0.02,
            stars: 0.8,
            haze: 0.3,
            moon_phase: 0.5,
            aurora: 0.0,
            milky_way: 0.7,
        }
    }
}

/// Unit vector pointing from the world towards the sun. Rises in +X at 6:00, peaks at noon, sets in -X at 18:00.
pub fn sun_direction(hour: f32) -> Vec3 {
    let a = (hour - 6.0) / 24.0 * std::f32::consts::TAU;
    Vec3::new(a.cos(), a.sin(), 0.35).normalize()
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// Fraction of the moon's disc that is lit: 0 at new moon, 1 at full moon.
pub fn moon_illumination(phase: f32) -> f32 {
    0.5 - 0.5 * (std::f32::consts::TAU * phase).cos()
}

/// The strongest light in the sky right now: `(direction towards it, linear colour, intensity)`.
/// The sun by day (orange near the horizon), a dim blue moon at night.
pub fn main_light(sky: &SkySettings) -> (Vec3, [f32; 3], f32) {
    let sun = sun_direction(sky.time);
    let e = sun.y;
    let day = smoothstep(-0.06, 0.18, e);
    let warm = mix3([1.0, 0.5, 0.25], [1.0, 0.96, 0.9], smoothstep(0.02, 0.45, e));
    let cloud_dim = 1.0 - 0.55 * sky.clouds.clamp(0.0, 1.0);
    if day > 0.001 {
        (sun, warm, 1.35 * day * cloud_dim)
    } else {
        (-sun, [0.55, 0.65, 1.0], 0.03 + 0.15 * moon_illumination(sky.moon_phase))
    }
}

/// Hemisphere ambient light colour for the time of day.
pub fn ambient(sky: &SkySettings) -> [f32; 3] {
    let e = sun_direction(sky.time).y;
    let day = smoothstep(-0.1, 0.25, e);
    let dusk = (1.0 - (e * 4.0).abs().min(1.0)) * 0.5;
    let base = mix3([0.025, 0.035, 0.08], [0.3, 0.34, 0.42], day);
    mix3(base, [0.4, 0.28, 0.25], dusk * day.max(0.3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_follows_the_day() {
        assert!(sun_direction(12.0).y > 0.9);
        assert!(sun_direction(0.0).y < -0.9);
        assert!(sun_direction(6.0).y.abs() < 0.05);
        assert!(sun_direction(6.0).x > 0.9 && sun_direction(18.0).x < -0.9);
    }

    #[test]
    fn moon_phase_dims_night_light() {
        let full = main_light(&SkySettings { time: 0.0, moon_phase: 0.5, ..Default::default() }).2;
        let new = main_light(&SkySettings { time: 0.0, moon_phase: 0.0, ..Default::default() }).2;
        assert!(full > new * 3.0 && moon_illumination(0.5) > 0.99 && moon_illumination(0.0) < 0.01);
    }

    #[test]
    fn light_is_bright_at_noon_and_dim_at_night() {
        let noon = main_light(&SkySettings { time: 12.0, ..Default::default() });
        let night = main_light(&SkySettings { time: 0.0, ..Default::default() });
        assert!(noon.2 > 1.0 && night.2 < 0.3);
        assert!(noon.0.y > 0.0 && night.0.y > 0.0, "the moon also shines from above");
        assert!(
            ambient(&SkySettings { time: 12.0, ..Default::default() })[0]
                > ambient(&SkySettings { time: 0.0, ..Default::default() })[0] * 4.0
        );
    }
}
