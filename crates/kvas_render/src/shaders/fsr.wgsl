// Spatial upscaler in the spirit of AMD FidelityFX Super Resolution 1.0: an edge-preserving
// bicubic reconstruction (EASU-like, with anti-ringing clamp) followed by contrast-adaptive
// sharpening (RCAS-like). Reads the low-resolution frame, writes the full-resolution one.

struct Params {
    src_size: vec2<f32>,
    out_size: vec2<f32>,
    sharpness: f32,
    pad0: f32,
    pad1: f32,
    pad2: f32,
}

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var<uniform> u: Params;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    var o: VsOut;
    o.pos = vec4<f32>(p[i], 0.0, 1.0);
    o.uv = vec2<f32>(p[i].x * 0.5 + 0.5, 0.5 - p[i].y * 0.5);
    return o;
}

fn load(p: vec2<i32>) -> vec3<f32> {
    let hi = vec2<i32>(u.src_size) - vec2<i32>(1);
    return textureLoad(src, clamp(p, vec2<i32>(0), hi), 0).rgb;
}

// Catmull-Rom weight.
fn w(t: f32) -> f32 {
    let a = abs(t);
    if (a < 1.0) {
        return 1.5 * a * a * a - 2.5 * a * a + 1.0;
    }
    if (a < 2.0) {
        return -0.5 * a * a * a + 2.5 * a * a - 4.0 * a + 2.0;
    }
    return 0.0;
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

fn reconstruct(uv: vec2<f32>) -> vec3<f32> {
    let pos = uv * u.src_size - vec2<f32>(0.5);
    let base = vec2<i32>(floor(pos));
    let f = pos - floor(pos);
    var acc = vec3<f32>(0.0);
    var wsum = 0.0;
    var lo = vec3<f32>(1e9);
    var hi = vec3<f32>(-1e9);
    for (var y = -1; y <= 2; y = y + 1) {
        for (var x = -1; x <= 2; x = x + 1) {
            let c = load(base + vec2<i32>(x, y));
            let wt = w(f32(x) - f.x) * w(f32(y) - f.y);
            acc += c * wt;
            wsum += wt;
            if (x >= 0 && x <= 1 && y >= 0 && y <= 1) {
                lo = min(lo, c);
                hi = max(hi, c);
            }
        }
    }
    // Anti-ringing: the result may not leave the range of the four nearest source pixels.
    return clamp(acc / wsum, lo, hi);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let px = 1.0 / u.out_size;
    let c = reconstruct(in.uv);
    if (u.sharpness <= 0.0) {
        return vec4<f32>(c, 1.0);
    }
    let n = reconstruct(in.uv + vec2<f32>(0.0, -px.y));
    let s = reconstruct(in.uv + vec2<f32>(0.0, px.y));
    let e = reconstruct(in.uv + vec2<f32>(px.x, 0.0));
    let wv = reconstruct(in.uv + vec2<f32>(-px.x, 0.0));
    // Contrast-adaptive: flat areas get the full effect, high-contrast edges get less (no halos).
    let mn = min(min(luma(n), luma(s)), min(luma(e), luma(wv)));
    let mx = max(max(luma(n), luma(s)), max(luma(e), luma(wv)));
    let adapt = clamp(1.0 - (mx - mn) * 2.0, 0.15, 1.0);
    let avg = (n + s + e + wv) * 0.25;
    let sharp = c + (c - avg) * u.sharpness * 1.6 * adapt;
    return vec4<f32>(clamp(sharp, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
