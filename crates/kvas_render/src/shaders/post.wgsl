// Full-screen post effects: bloom, lens flare, blur, motion blur, chromatic aberration,
// sharpen, vignette, film grain and tint. One pass, reads the finished scene colour.

struct Post {
    a: vec4<f32>,   // bloom, bloom threshold, bloom radius, lens flare
    b: vec4<f32>,   // blur, motion blur, vignette, chromatic aberration
    c: vec4<f32>,   // film grain, sharpen, time, aspect (w / h)
    d: vec4<f32>,   // tint rgb
    e: vec4<f32>,   // motion uv shift xy, texel size xy
    f: vec4<f32>,   // depth of field on, focus distance, f-number, focal length (mm)
    g: vec4<f32>,   // near, far, exposure compensation (stops), letterbox ratio
}

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
@group(0) @binding(2) var<uniform> u: Post;
@group(0) @binding(3) var depth_tex: texture_depth_multisampled_2d;

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

fn tap(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(src, smp, uv, 0.0).rgb;
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(123.34, 456.21));
    let r = q + dot(q, q + 45.32);
    return fract(r.x * r.y);
}

// Bright part of the picture only (what bloom and flares are made of).
fn bright(uv: vec2<f32>) -> vec3<f32> {
    let c = tap(uv);
    let k = max(luma(c) - u.a.y, 0.0) / max(1.0 - u.a.y, 0.15);
    return c * k;
}

const GOLDEN: f32 = 2.39996323;

fn view_depth(p: vec2<i32>) -> f32 {
    let size = vec2<i32>(textureDimensions(depth_tex));
    let d = textureLoad(depth_tex, clamp(p, vec2<i32>(0), size - vec2<i32>(1)), 0);
    let near = u.g.x;
    let far = u.g.y;
    return near * far / max(far - d * (far - near), 1e-4);
}

// Circle of confusion in pixels for a point at distance z (thin-lens model, 24 mm sensor height).
fn coc_pixels(z: f32) -> f32 {
    let f = u.f.w * 0.001;
    let s = max(u.f.y, f + 0.05);
    let n = max(u.f.z, 0.7);
    let coc = f * f / (n * (s - f)) * abs(z - s) / max(z, 0.01);
    return min(coc / 0.024 / u.e.w, 26.0);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let d = uv - vec2<f32>(0.5);
    let aspect = vec2<f32>(1.0, u.c.w);

    // Chromatic aberration: red and blue drift apart towards the corners.
    let ca = u.b.w * dot(d, d) * 0.06;
    var col = vec3<f32>(tap(uv + d * ca).r, tap(uv).g, tap(uv - d * ca).b);

    // Depth of field: gather a disc whose radius follows how far this pixel is from the focus plane.
    if (u.f.x > 0.5) {
        let pix = vec2<i32>(in.pos.xy);
        let c0 = coc_pixels(view_depth(pix));
        if (c0 > 0.6) {
            var acc = vec3<f32>(0.0);
            var wsum = 0.0;
            for (var i = 0; i < 32; i = i + 1) {
                let fi = f32(i) + 0.5;
                let r = sqrt(fi / 32.0) * c0;
                let a = fi * GOLDEN;
                let off = vec2<f32>(cos(a), sin(a)) * r;
                let sample_c = coc_pixels(view_depth(pix + vec2<i32>(round(off))));
                // Sharp objects in front must not bleed into the blur behind them (and vice versa).
                let w = select(0.2, 1.0, sample_c >= r * 0.75);
                acc += tap(uv + off * u.e.zw) * w;
                wsum += w;
            }
            col = mix(col, acc / max(wsum, 1e-4), smoothstep(0.6, 2.0, c0));
        }
    }

    // Full-screen blur: a golden-angle disc.
    if (u.b.x > 0.0) {
        var acc = vec3<f32>(0.0);
        let r = u.b.x * 0.03;
        for (var i = 0; i < 24; i = i + 1) {
            let fi = f32(i) + 0.5;
            let rad = sqrt(fi / 24.0) * r;
            let a = fi * GOLDEN;
            acc += tap(uv + vec2<f32>(cos(a), sin(a)) * rad * aspect);
        }
        col = mix(col, acc / 24.0, clamp(u.b.x * 4.0, 0.0, 1.0));
    }

    // Motion blur: streak along how far the picture moved since the last frame.
    if (u.b.y > 0.0) {
        let m = u.e.xy * u.b.y;
        if (dot(m, m) > 1e-9) {
            var acc = vec3<f32>(0.0);
            for (var i = 0; i < 10; i = i + 1) {
                let t = f32(i) / 9.0 - 0.5;
                acc += tap(uv + m * t);
            }
            col = mix(col, acc / 10.0, 0.9);
        }
    }

    // Sharpen: add back the difference to a small blur.
    if (u.c.y > 0.0) {
        let t = u.e.zw;
        let blur = (tap(uv + vec2<f32>(t.x, 0.0)) + tap(uv - vec2<f32>(t.x, 0.0)) + tap(uv + vec2<f32>(0.0, t.y)) + tap(uv - vec2<f32>(0.0, t.y))) * 0.25;
        col = col + (col - blur) * u.c.y * 1.5;
    }

    // Bloom: soft halo from bright pixels, sampled on a spiral.
    if (u.a.x > 0.0) {
        var acc = vec3<f32>(0.0);
        let r = u.a.z * 0.035;
        for (var i = 0; i < 32; i = i + 1) {
            let fi = f32(i) + 0.5;
            let rad = fi / 32.0;
            let a = fi * GOLDEN;
            let w = 1.0 - rad * 0.7;
            acc += bright(uv + vec2<f32>(cos(a), sin(a)) * rad * r * aspect) * w;
        }
        col += acc / 32.0 * u.a.x * 2.2;
    }

    // Lens flare: ghost images mirrored through the centre, plus a thin halo ring.
    if (u.a.w > 0.0) {
        var acc = vec3<f32>(0.0);
        for (var i = 1; i < 5; i = i + 1) {
            let k = f32(i) * 0.32;
            let g = vec2<f32>(0.5) - d * k;
            let tint = mix(vec3<f32>(1.0, 0.75, 0.5), vec3<f32>(0.5, 0.75, 1.0), f32(i) / 4.0);
            let fade = pow(1.0 - clamp(length(g - vec2<f32>(0.5)) * 1.6, 0.0, 1.0), 2.0);
            acc += bright(g) * tint * fade;
        }
        let ring_uv = vec2<f32>(0.5) - normalize(d + vec2<f32>(1e-5)) * 0.35;
        acc += bright(ring_uv) * vec3<f32>(0.6, 0.8, 1.0) * 0.5;
        col += acc * u.a.w * 0.9;
    }

    // Vignette.
    let v = smoothstep(0.35, 1.05, length(d * aspect) * 1.5);
    col *= 1.0 - u.b.z * v;

    // Film grain.
    let g = hash(uv * vec2<f32>(1920.0, 1080.0) + vec2<f32>(u.c.z * 37.0, u.c.z * 91.0)) - 0.5;
    col += vec3<f32>(g) * u.c.x * 0.25;

    col *= u.d.xyz;
    col *= exp2(u.g.z);

    // Cinematic bars for a wider (or narrower) picture ratio.
    if (u.g.w > 0.0) {
        let shown = u.c.w / u.g.w;
        if (shown < 1.0 && abs(uv.y - 0.5) > shown * 0.5) {
            col = vec3<f32>(0.0);
        } else if (shown > 1.0 && abs(uv.x - 0.5) > 0.5 / shown) {
            col = vec3<f32>(0.0);
        }
    }
    return vec4<f32>(max(col, vec3<f32>(0.0)), 1.0);
}
