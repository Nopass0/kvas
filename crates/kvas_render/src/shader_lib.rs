//! Shader source preprocessing: `#include "path.wgsl"` pulls shared functions from another file
//! (project path, or one of the built-in libraries below), so effects and materials can reuse
//! helpers instead of copying them.
//!
//! ```wgsl
//! #include "builtin:lib/color"
//! fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
//!     return lib_hue_shift(color, params.x * time);
//! }
//! ```

use kvas_engine::assets::Assets;

/// Colour helpers: luma, HSV, hue shift, saturation, contrast, vignette, ACES.
pub const LIB_COLOR: &str = "
fn lib_luma(c: vec3<f32>) -> f32 { return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722)); }
fn lib_rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4<f32>(c.bg, k.wz), vec4<f32>(c.gb, k.xy), step(c.b, c.g));
    let q = mix(vec4<f32>(p.xyw, c.r), vec4<f32>(c.r, p.yzx), step(p.x, c.r));
    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10;
    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}
fn lib_hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let p = abs(fract(vec3<f32>(c.x) + vec3<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - vec3<f32>(3.0));
    return c.z * mix(vec3<f32>(1.0), clamp(p - vec3<f32>(1.0), vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}
fn lib_hue_shift(c: vec3<f32>, turns: f32) -> vec3<f32> {
    var h = lib_rgb_to_hsv(c);
    h.x = fract(h.x + turns);
    return lib_hsv_to_rgb(h);
}
fn lib_saturation(c: vec3<f32>, amount: f32) -> vec3<f32> { return mix(vec3<f32>(lib_luma(c)), c, amount); }
fn lib_contrast(c: vec3<f32>, amount: f32) -> vec3<f32> { return (c - vec3<f32>(0.5)) * amount + vec3<f32>(0.5); }
fn lib_vignette(uv: vec2<f32>, strength: f32) -> f32 {
    let d = uv - vec2<f32>(0.5);
    return 1.0 - strength * smoothstep(0.3, 0.9, length(d) * 1.4);
}
";

/// Hash and value noise on the GPU: `lib_hash21`, `lib_noise`, `lib_fbm`.
pub const LIB_NOISE: &str = "
fn lib_hash21(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(123.34, 456.21));
    let r = q + dot(q, q + 45.32);
    return fract(r.x * r.y);
}
fn lib_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(lib_hash21(i), lib_hash21(i + vec2<f32>(1.0, 0.0)), u.x), mix(lib_hash21(i + vec2<f32>(0.0, 1.0)), lib_hash21(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}
fn lib_fbm(p: vec2<f32>) -> f32 {
    var a = 0.5;
    var v = 0.0;
    var q = p;
    for (var i = 0; i < 4; i = i + 1) {
        v += a * lib_noise(q);
        q = q * 2.03 + vec2<f32>(17.0, 9.0);
        a *= 0.5;
    }
    return v;
}
// Cell noise: x = distance to the nearest feature point, y = random value of that cell.
fn lib_voronoi(p: vec2<f32>) -> vec2<f32> {
    let i = floor(p);
    let f = fract(p);
    var best = 8.0;
    var id = 0.0;
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let g = vec2<f32>(f32(x), f32(y));
            let o = vec2<f32>(lib_hash21(i + g), lib_hash21(i + g + vec2<f32>(7.1, 3.7)));
            let d = length(g + o - f);
            if (d < best) {
                best = d;
                id = lib_hash21(i + g + vec2<f32>(1.3, 9.2));
            }
        }
    }
    return vec2<f32>(best, id);
}
";

/// Small maths helpers: `lib_remap`, `lib_pulse`, `lib_rot2`.
pub const LIB_MATH: &str = "
fn lib_remap(v: f32, a: f32, b: f32, c: f32, d: f32) -> f32 { return c + (v - a) / (b - a) * (d - c); }
fn lib_pulse(t: f32, hz: f32) -> f32 { return 0.5 + 0.5 * sin(t * hz * 6.2831853); }
fn lib_rot2(p: vec2<f32>, a: f32) -> vec2<f32> { let s = sin(a); let c = cos(a); return vec2<f32>(c * p.x - s * p.y, s * p.x + c * p.y); }
";

/// Helpers used by the node editor's generated shaders (`builtin:lib/nodes`).
pub const LIB_NODES: &str = "
fn lib_ridged(p: vec2<f32>) -> f32 {
    var a = 0.5;
    var v = 0.0;
    var q = p;
    for (var i = 0; i < 4; i = i + 1) {
        v += a * (1.0 - abs(lib_noise(q) * 2.0 - 1.0));
        q = q * 2.03 + vec2<f32>(17.0, 9.0);
        a *= 0.5;
    }
    return v;
}
fn lib_turbulence(p: vec2<f32>) -> f32 {
    var a = 0.5;
    var v = 0.0;
    var q = p;
    for (var i = 0; i < 4; i = i + 1) {
        v += a * abs(lib_noise(q) * 2.0 - 1.0);
        q = q * 2.03 + vec2<f32>(17.0, 9.0);
        a *= 0.5;
    }
    return v;
}
fn lib_caustic(p: vec2<f32>, t: f32) -> f32 {
    var q = p;
    var v = 0.0;
    for (var i = 0; i < 3; i = i + 1) {
        q = q + vec2<f32>(sin(q.y * 1.7 + t), cos(q.x * 1.3 + t)) * 0.6;
        v += abs(sin(q.x * 2.0) * sin(q.y * 2.0));
    }
    return pow(1.0 - clamp(v / 3.0, 0.0, 1.0), 4.0) * 2.0;
}
fn lib_hex(p: vec2<f32>) -> f32 {
    let s = vec2<f32>(1.0, 1.7320508);
    let a = (p - s * floor(p / s)) - s * 0.5;
    let q = p + s * 0.5;
    let b = (q - s * floor(q / s)) - s * 0.5;
    let g = select(b, a, dot(a, a) < dot(b, b));
    let ag = abs(g);
    return max(dot(ag, vec2<f32>(0.5, 0.8660254)), ag.x);
}
fn lib_wood(p: vec3<f32>, wobble: f32) -> f32 {
    let q = p.xz + vec2<f32>(lib_noise(p.xy * 2.0), lib_noise(p.zy * 2.0)) * wobble;
    let r = length(q) * 6.0;
    return fract(r) * 0.6 + lib_noise(p.xz * 30.0) * 0.1 + 0.2;
}
fn lib_polar(uv: vec2<f32>) -> vec2<f32> {
    let d = uv - vec2<f32>(0.5);
    return vec2<f32>(atan2(d.y, d.x) / 6.2831853 + 0.5, length(d) * 2.0);
}
fn lib_flipbook(uv: vec2<f32>, cols: f32, rows: f32, frame: f32) -> vec2<f32> {
    let total = max(cols * rows, 1.0);
    let f = frame - total * floor(frame / total);
    let cx = f - cols * floor(f / cols);
    let cy = floor(f / cols);
    return (fract(uv) + vec2<f32>(cx, cy)) / vec2<f32>(cols, rows);
}
fn lib_overlay(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return select(vec3<f32>(1.0) - 2.0 * (vec3<f32>(1.0) - a) * (vec3<f32>(1.0) - b), 2.0 * a * b, a < vec3<f32>(0.5));
}
fn lib_screen(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(1.0) - (vec3<f32>(1.0) - a) * (vec3<f32>(1.0) - b);
}
fn lib_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}
fn lib_sdf_box(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let d = abs(p) - half_size;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}
fn lib_bump(h: f32, p: vec3<f32>, n: vec3<f32>, strength: f32) -> vec3<f32> {
    let dpx = dpdx(p);
    let dpy = dpdy(p);
    let r1 = cross(dpy, n);
    let r2 = cross(n, dpx);
    let det = dot(dpx, r1);
    let grad = sign(det) * (dpdx(h) * r1 + dpdy(h) * r2);
    return normalize(abs(det) * n - strength * grad + vec3<f32>(0.0000001));
}
fn lib_normal_map(n: vec3<f32>, p: vec3<f32>, uv: vec2<f32>, m: vec3<f32>, strength: f32) -> vec3<f32> {
    let dp1 = dpdx(p);
    let dp2 = dpdy(p);
    let duv1 = dpdx(uv);
    let duv2 = dpdy(uv);
    let dp2perp = cross(dp2, n);
    let dp1perp = cross(n, dp1);
    let t = dp2perp * duv1.x + dp1perp * duv2.x;
    let b = dp2perp * duv1.y + dp1perp * duv2.y;
    let invmax = inverseSqrt(max(max(dot(t, t), dot(b, b)), 0.00000001));
    let tm = m * 2.0 - vec3<f32>(1.0);
    return normalize(t * invmax * tm.x * strength + b * invmax * tm.y * strength + n * tm.z);
}

// ---- third wave: 3D noises, patterns, blend modes
fn lib_h31(p: vec3<f32>) -> f32 {
    var q = fract(p * 0.1031);
    q = q + dot(q, q.zyx + vec3<f32>(31.32));
    return fract((q.x + q.y) * q.z);
}
fn lib_h33(p: vec3<f32>) -> vec3<f32> {
    var q = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    q = q + dot(q, q.yxz + vec3<f32>(33.33));
    return fract((q.xxy + q.yxx) * q.zyx);
}
fn lib_vnoise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(lib_h31(i), lib_h31(i + vec3<f32>(1.0, 0.0, 0.0)), u.x), mix(lib_h31(i + vec3<f32>(0.0, 1.0, 0.0)), lib_h31(i + vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y);
    let b = mix(mix(lib_h31(i + vec3<f32>(0.0, 0.0, 1.0)), lib_h31(i + vec3<f32>(1.0, 0.0, 1.0)), u.x), mix(lib_h31(i + vec3<f32>(0.0, 1.0, 1.0)), lib_h31(i + vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y);
    return mix(a, b, u.z);
}
fn lib_grad(i: vec3<f32>, f: vec3<f32>) -> f32 {
    let g = normalize(lib_h33(i) * 2.0 - vec3<f32>(1.0) + vec3<f32>(0.0001));
    return dot(g, f);
}
fn lib_gnoise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let x0 = mix(mix(lib_grad(i, f), lib_grad(i + vec3<f32>(1.0, 0.0, 0.0), f - vec3<f32>(1.0, 0.0, 0.0)), u.x), mix(lib_grad(i + vec3<f32>(0.0, 1.0, 0.0), f - vec3<f32>(0.0, 1.0, 0.0)), lib_grad(i + vec3<f32>(1.0, 1.0, 0.0), f - vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y);
    let x1 = mix(mix(lib_grad(i + vec3<f32>(0.0, 0.0, 1.0), f - vec3<f32>(0.0, 0.0, 1.0)), lib_grad(i + vec3<f32>(1.0, 0.0, 1.0), f - vec3<f32>(1.0, 0.0, 1.0)), u.x), mix(lib_grad(i + vec3<f32>(0.0, 1.0, 1.0), f - vec3<f32>(0.0, 1.0, 1.0)), lib_grad(i + vec3<f32>(1.0, 1.0, 1.0), f - vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y);
    return clamp(mix(x0, x1, u.z) * 0.9 + 0.5, 0.0, 1.0);
}
// kind: 0 fBm, 1 ridged, 2 billow.
fn lib_fractal(p: vec3<f32>, octaves: f32, lacunarity: f32, gain: f32, kind: f32) -> f32 {
    var amp = 0.5;
    var sum = 0.0;
    var norm = 0.0;
    var q = p;
    for (var i = 0; i < 8; i = i + 1) {
        if (f32(i) >= octaves) { break; }
        var n = lib_vnoise3(q);
        if (kind > 1.5) {
            n = abs(n * 2.0 - 1.0);
        } else if (kind > 0.5) {
            n = 1.0 - abs(n * 2.0 - 1.0);
            n = n * n;
        }
        sum = sum + n * amp;
        norm = norm + amp;
        q = q * lacunarity + vec3<f32>(17.0, 9.0, 13.0);
        amp = amp * gain;
    }
    return sum / max(norm, 0.0001);
}
// x = nearest distance, y = second nearest, z = random id of the nearest cell.
fn lib_voronoi3(p: vec3<f32>, randomness: f32) -> vec3<f32> {
    let i = floor(p);
    let f = fract(p);
    var f1 = 8.0;
    var f2 = 8.0;
    var id = 0.0;
    for (var z = -1; z <= 1; z = z + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            for (var x = -1; x <= 1; x = x + 1) {
                let g = vec3<f32>(f32(x), f32(y), f32(z));
                let o = vec3<f32>(0.5) + (lib_h33(i + g) - vec3<f32>(0.5)) * randomness;
                let d = length(g + o - f);
                if (d < f1) {
                    f2 = f1;
                    f1 = d;
                    id = lib_h31(i + g + vec3<f32>(3.1, 7.7, 1.3));
                } else if (d < f2) {
                    f2 = d;
                }
            }
        }
    }
    return vec3<f32>(f1, f2, id);
}
fn lib_voro_out(v: vec3<f32>, mode: f32) -> f32 {
    if (mode < 0.5) { return clamp(v.x, 0.0, 1.0); }
    if (mode < 1.5) { return clamp(v.y - v.x, 0.0, 1.0); }
    if (mode < 2.5) { return v.z; }
    return clamp(v.y * 0.5, 0.0, 1.0);
}
fn lib_cracks(p: vec3<f32>, width: f32) -> f32 {
    let v = lib_voronoi3(p, 1.0);
    return 1.0 - smoothstep(0.0, width, v.y - v.x);
}
fn lib_warp(p: vec3<f32>, amount: f32) -> f32 {
    let q = vec3<f32>(lib_fractal(p + vec3<f32>(5.2, 1.3, 8.1), 3.0, 2.0, 0.5, 0.0), lib_fractal(p + vec3<f32>(1.7, 9.2, 3.4), 3.0, 2.0, 0.5, 0.0), lib_fractal(p + vec3<f32>(8.3, 2.8, 4.1), 3.0, 2.0, 0.5, 0.0));
    return lib_fractal(p + (q - vec3<f32>(0.5)) * 2.0 * amount, 4.0, 2.0, 0.5, 0.0);
}
fn lib_scratches(uv: vec2<f32>, density: f32, angle: f32) -> f32 {
    let p = lib_rot2(uv, angle) * vec2<f32>(density * 0.05, density);
    let line = pow(1.0 - abs(lib_noise(p) * 2.0 - 1.0), 14.0);
    return line * step(0.55, lib_noise(p * 0.4 + vec2<f32>(5.0, 3.0)));
}
fn lib_grid(uv: vec2<f32>, cells: f32, width: f32) -> f32 {
    let f = fract(uv * cells);
    let d = min(f, vec2<f32>(1.0) - f);
    return 1.0 - smoothstep(0.0, width + 0.0001, min(d.x, d.y));
}
fn lib_planks(uv: vec2<f32>, rows: f32, length_scale: f32, gap: f32) -> f32 {
    let row = floor(uv.y * rows);
    let x = uv.x * rows / max(length_scale, 0.01) + lib_hash21(vec2<f32>(row, 3.7)) * 10.0;
    let cell = floor(x);
    let f = vec2<f32>(fract(x), fract(uv.y * rows));
    let edge = min(min(f.x, 1.0 - f.x) * length_scale, min(f.y, 1.0 - f.y));
    let tone = 0.62 + 0.38 * lib_hash21(vec2<f32>(cell, row));
    return tone * smoothstep(0.0, gap + 0.0001, edge);
}
fn lib_weave(uv: vec2<f32>, threads: f32) -> f32 {
    let p = uv * threads;
    let c = floor(p);
    let f = fract(p);
    let odd = abs(c.x + c.y - 2.0 * floor((c.x + c.y) * 0.5));
    let h = select(sin(f.y * 3.14159265), sin(f.x * 3.14159265), odd > 0.5);
    return 0.25 + 0.75 * h;
}
fn lib_blend(a: vec3<f32>, b: vec3<f32>, mode: f32) -> vec3<f32> {
    let m = i32(floor(mode + 0.5));
    let one = vec3<f32>(1.0);
    if (m == 1) { return a * b; }
    if (m == 2) { return lib_screen(a, b); }
    if (m == 3) { return lib_overlay(clamp(a, vec3<f32>(0.0), one), b); }
    if (m == 4) { return a + b; }
    if (m == 5) { return a - b; }
    if (m == 6) { return abs(a - b); }
    if (m == 7) { return min(a, b); }
    if (m == 8) { return max(a, b); }
    if (m == 9) { return (one - 2.0 * b) * a * a + 2.0 * b * a; }
    if (m == 10) { return a / max(one - b, vec3<f32>(0.001)); }
    if (m == 11) { return one - (one - a) / max(b, vec3<f32>(0.001)); }
    if (m == 12) { return lib_overlay(clamp(b, vec3<f32>(0.0), one), a); }
    return b;
}
fn lib_hsv_adjust(c: vec3<f32>, dh: f32, s: f32, v: f32) -> vec3<f32> {
    var h = lib_rgb_to_hsv(c);
    h.x = fract(h.x + dh);
    h.y = clamp(h.y * s, 0.0, 1.0);
    h.z = h.z * v;
    return lib_hsv_to_rgb(h);
}
fn lib_rotate_axis(v: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
    let k = normalize(axis + vec3<f32>(0.00001));
    let c = cos(angle);
    let s = sin(angle);
    return v * c + cross(k, v) * s + k * dot(k, v) * (1.0 - c);
}
fn lib_twirl(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let d = uv - vec2<f32>(0.5);
    let r = clamp(length(d) * 2.0, 0.0, 1.0);
    return lib_rot2(d, amount * (1.0 - r)) + vec2<f32>(0.5);
}
fn lib_smoother(x: vec3<f32>, e0: f32, e1: f32) -> vec3<f32> {
    let t = clamp((x - vec3<f32>(e0)) / vec3<f32>(e1 - e0 + 0.00001), vec3<f32>(0.0), vec3<f32>(1.0));
    return t * t * t * (t * (t * 6.0 - vec3<f32>(15.0)) + vec3<f32>(10.0));
}
fn lib_bias(x: vec3<f32>, b: f32) -> vec3<f32> {
    return pow(clamp(x, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(log(max(b, 0.001)) / log(0.5)));
}
fn lib_gain(x: vec3<f32>, g: f32) -> vec3<f32> {
    let lo = lib_bias(2.0 * x, 1.0 - g) * 0.5;
    let hi = vec3<f32>(1.0) - lib_bias(vec3<f32>(2.0) - 2.0 * x, 1.0 - g) * 0.5;
    return select(hi, lo, x < vec3<f32>(0.5));
}
// Tangent-space normal-map colour from the red channel of a texture (height map).
fn lib_tex_normal(t: texture_2d<f32>, s: sampler, uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let d = vec2<f32>(1.0) / vec2<f32>(textureDimensions(t));
    let l = textureSampleLevel(t, s, uv - vec2<f32>(d.x, 0.0), 0.0).r;
    let r = textureSampleLevel(t, s, uv + vec2<f32>(d.x, 0.0), 0.0).r;
    let u = textureSampleLevel(t, s, uv - vec2<f32>(0.0, d.y), 0.0).r;
    let b = textureSampleLevel(t, s, uv + vec2<f32>(0.0, d.y), 0.0).r;
    return normalize(vec3<f32>((l - r) * strength, (b - u) * strength, 1.0)) * 0.5 + vec3<f32>(0.5);
}
fn lib_triplanar(t: texture_2d<f32>, s: sampler, p: vec3<f32>, n: vec3<f32>, sharpness: f32) -> vec3<f32> {
    var w = pow(abs(n), vec3<f32>(sharpness));
    w = w / max(w.x + w.y + w.z, 0.0001);
    return textureSample(t, s, p.zy).rgb * w.x + textureSample(t, s, p.xz).rgb * w.y + textureSample(t, s, p.xy).rgb * w.z;
}
";

fn builtin(name: &str) -> Option<&'static str> {
    match name {
        "builtin:lib/color" => Some(LIB_COLOR),
        "builtin:lib/noise" => Some(LIB_NOISE),
        "builtin:lib/math" => Some(LIB_MATH),
        "builtin:lib/nodes" => Some(LIB_NODES),
        _ => None,
    }
}

/// First line of the generated declaration files (`kvas_mesh_api.wgsl`); `#include` ignores such files.
pub const STUB_MARKER: &str = "// @kvas-stub";

/// Replaces every `#include "path"` line with the included file (each file once, up to 8 levels deep).
pub fn expand(assets: &Assets, source: &str, base_dir: &str) -> Result<String, String> {
    let mut seen = Vec::new();
    expand_inner(assets, source, base_dir, &mut seen, 0)
}

fn expand_inner(assets: &Assets, source: &str, base_dir: &str, seen: &mut Vec<String>, depth: usize) -> Result<String, String> {
    if depth > 8 {
        return Err("#include nested too deeply".into());
    }
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("#include") else {
            out.push_str(line);
            out.push('\n');
            continue;
        };
        let name = rest.split("//").next().unwrap_or("").trim().trim_matches('"').trim_matches('<').trim_matches('>').to_string();
        if name.is_empty() {
            return Err("#include needs a path".into());
        }
        let resolved = if builtin(&name).is_some() || name.contains('/') && assets.read(&name).is_ok() || base_dir.is_empty() { name.clone() } else { format!("{base_dir}/{name}") };
        if seen.contains(&resolved) {
            continue;
        }
        seen.push(resolved.clone());
        let text = match builtin(&resolved) {
            Some(b) => b.to_string(),
            None => assets.read_string(&resolved).map_err(|e| format!("#include \"{name}\": {e}"))?,
        };
        if text.trim_start().starts_with(STUB_MARKER) {
            // Editor-only declaration files add nothing: the engine already provides these names.
            continue;
        }
        let dir = resolved.rsplit_once('/').map_or("", |(d, _)| d).to_string();
        out.push_str(&expand_inner(assets, &text, &dir, seen, depth + 1)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_includes_expand_once() {
        let assets = Assets::Dir(std::path::PathBuf::new());
        let src = "#include \"builtin:lib/color\"\n#include \"builtin:lib/color\"\nfn effect() {}\n";
        let out = expand(&assets, src, "").unwrap();
        assert_eq!(out.matches("fn lib_luma").count(), 1);
        assert!(out.contains("fn effect"));
        assert!(expand(&assets, "#include \"nope.wgsl\"", "").is_err());
    }
}
