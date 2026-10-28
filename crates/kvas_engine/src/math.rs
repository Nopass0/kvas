//! Small scalar helpers for gameplay scripts: interpolation, smoothing and angles.
//!
//! All functions are `f32`, allocation-free and safe for any finite input.
//!
//! ```
//! use kvas_engine::math::*;
//! assert_eq!(lerp(0.0, 10.0, 0.25), 2.5);
//! assert_eq!(remap(5.0, 0.0, 10.0, 100.0, 200.0), 150.0);
//! assert_eq!(approach(0.0, 10.0, 3.0), 3.0);
//! assert!((wrap_degrees(190.0) + 170.0).abs() < 1e-4);
//! ```

/// Linear interpolation: `a` at `t = 0`, `b` at `t = 1` (not clamped).
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Where `v` lies between `a` and `b` (0 at `a`, 1 at `b`); 0 when `a == b`.
pub fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (v - a) / (b - a)
    }
}

/// Maps `v` from the range `in_a..in_b` to `out_a..out_b` (not clamped).
pub fn remap(v: f32, in_a: f32, in_b: f32, out_a: f32, out_b: f32) -> f32 {
    lerp(out_a, out_b, inverse_lerp(in_a, in_b, v))
}

/// Like [`remap`] but the result never leaves `out_a..out_b`.
pub fn remap_clamped(v: f32, in_a: f32, in_b: f32, out_a: f32, out_b: f32) -> f32 {
    lerp(out_a, out_b, inverse_lerp(in_a, in_b, v).clamp(0.0, 1.0))
}

/// Hermite smoothstep between two edges (0 below `e0`, 1 above `e1`).
pub fn smoothstep(e0: f32, e1: f32, v: f32) -> f32 {
    let t = inverse_lerp(e0, e1, v).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Moves `current` towards `target` by at most `max_delta` (never overshoots).
pub fn approach(current: f32, target: f32, max_delta: f32) -> f32 {
    let d = target - current;
    if d.abs() <= max_delta {
        target
    } else {
        current + d.signum() * max_delta
    }
}

/// Frame-rate independent exponential smoothing. Larger `lambda` is snappier;
/// call every frame with the frame's `dt`: `x = damp(x, target, 8.0, ctx.dt())`.
pub fn damp(current: f32, target: f32, lambda: f32, dt: f32) -> f32 {
    lerp(current, target, 1.0 - (-lambda.max(0.0) * dt.max(0.0)).exp())
}

/// Wraps an angle in degrees into `-180..180`.
pub fn wrap_degrees(angle: f32) -> f32 {
    let a = angle.rem_euclid(360.0);
    if a > 180.0 {
        a - 360.0
    } else {
        a
    }
}

/// Shortest signed difference `to - from` in degrees (`-180..180`).
pub fn delta_degrees(from: f32, to: f32) -> f32 {
    wrap_degrees(to - from)
}

/// Rotates an angle in degrees towards `target` along the short way by at most `max_delta`.
pub fn approach_degrees(current: f32, target: f32, max_delta: f32) -> f32 {
    current + approach(0.0, delta_degrees(current, target), max_delta)
}

/// Critically damped spring towards `target` (like a camera "smooth damp"): returns the new value and the
/// new velocity. Keep the velocity between calls. `smooth_time` is roughly the seconds to arrive.
///
/// ```
/// use kvas_engine::math::smooth_damp;
/// let (mut x, mut v) = (0.0, 0.0);
/// for _ in 0..120 { (x, v) = smooth_damp(x, 10.0, v, 0.3, 1.0 / 60.0); }
/// assert!((x - 10.0).abs() < 0.05);
/// ```
pub fn smooth_damp(current: f32, target: f32, velocity: f32, smooth_time: f32, dt: f32) -> (f32, f32) {
    let omega = 2.0 / smooth_time.max(1e-4);
    let x = omega * dt;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let change = current - target;
    let temp = (velocity + omega * change) * dt;
    let new_velocity = (velocity - omega * temp) * exp;
    let out = target + (change + temp) * exp;
    // Never overshoot the target.
    if (target - current > 0.0) == (out > target) {
        return (target, 0.0);
    }
    (out, new_velocity)
}

/// Smooth S curve `3t² - 2t³` of a value clamped to 0..1.
pub fn ease_smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Even smoother S curve `6t⁵ - 15t⁴ + 10t³` with zero acceleration at both ends.
pub fn ease_smoother(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_behave() {
        assert_eq!(inverse_lerp(2.0, 2.0, 5.0), 0.0);
        assert_eq!(remap_clamped(20.0, 0.0, 10.0, 0.0, 1.0), 1.0);
        assert_eq!(smoothstep(0.0, 1.0, 0.5), 0.5);
        assert_eq!(approach(9.0, 10.0, 3.0), 10.0);
        assert!(damp(0.0, 1.0, 10.0, 0.016) > 0.1);
        assert!((approach_degrees(350.0, 10.0, 5.0) - 355.0).abs() < 1e-4);
    }
}
