// kvas:decal
// Built-in projected decal (the prelude is prepended). The renderer draws the back faces of the
// decal's box without a depth test; each pixel finds the opaque surface behind it (scene depth),
// keeps it if it lies inside the box, and paints it. Works on any geometry, no UVs needed.
//
//   object.color   tint and alpha           object.params  x roughness, y -, z -, w -
//   object.params2 x metallic, z has normal map, w -
//   object.user0   x blend mode, y shape, z angle fade, w normal strength
//   object.user1   x seed, y emission, z has normal map, w has ORM map
// Blend modes: 0 normal, 1 multiply, 2 screen, 3 lighten, 4 darken, 5 overlay, 6 add,
// 7 soft light, 8 relief only, 9 colour dodge, 10 colour burn, 11 tint.
// Shapes: 0 image, 1 blob, 2 scorch, 3 bullet hole, 4 cracks, 5 puddle, 6 grunge, 7 paint splat,
// 8 moss, 9 rust, 10 leaks, 11 ring.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

fn rot2(p: vec2<f32>, a: f32) -> vec2<f32> {
    let s = sin(a);
    let c = cos(a);
    return vec2<f32>(c * p.x - s * p.y, s * p.x + c * p.y);
}

fn cell_edge(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    var d1 = 8.0;
    var d2 = 8.0;
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let g = vec2<f32>(f32(x), f32(y));
            let o = vec2<f32>(hash21(i + g), hash21(i + g + vec2<f32>(5.3, 1.7)));
            let d = length(g + o - f);
            if (d < d1) { d2 = d1; d1 = d; } else if (d < d2) { d2 = d; }
        }
    }
    return d2 - d1;
}

// Coverage (x) and relief height (y) of a procedural shape at uv (0..1).
fn decal_shape(uv: vec2<f32>, shape: i32, seed: f32) -> vec2<f32> {
    let p = (uv - vec2<f32>(0.5)) * 2.0;
    let r = length(p);
    let s = vec2<f32>(seed * 7.13, seed * 3.71);
    let edge_noise = fbm2(p * 2.5 + s, 4) - 0.5;
    switch shape {
        case 1: { // blob / blood
            let blob = smoothstep(0.75, 0.6, r + edge_noise * 0.55);
            var drops = 0.0;
            for (var i = 0; i < 6; i = i + 1) {
                let a = hash21(s + vec2<f32>(f32(i), 1.0)) * 6.283;
                let c = vec2<f32>(cos(a), sin(a)) * (0.65 + 0.25 * hash21(s + vec2<f32>(f32(i), 2.0)));
                drops = max(drops, smoothstep(0.09, 0.05, length(p - c)));
            }
            let m = max(blob, drops);
            return vec2<f32>(m, m * 0.3);
        }
        case 2: { // scorch
            let m = smoothstep(1.0, 0.1, r + edge_noise * 0.6) * (0.7 + 0.3 * fbm2(p * 6.0 + s, 3));
            return vec2<f32>(m, -m * 0.2);
        }
        case 3: { // bullet hole
            let hole = smoothstep(0.16, 0.12, r);
            let rim = smoothstep(0.35, 0.15, r) * (0.6 + 0.4 * fbm2(p * 10.0 + s, 2));
            let rays = smoothstep(0.6, 1.0, fbm2(vec2<f32>(atan2(p.y, p.x) * 3.0, r * 2.0) + s, 3)) * smoothstep(0.9, 0.2, r);
            return vec2<f32>(max(max(hole, rim * 0.85), rays * 0.6), rim * 0.5 - hole * 1.5);
        }
        case 4: { // cracks
            let warp = vec2<f32>(fbm2(p * 2.0 + s, 3), fbm2(p * 2.0 + s + vec2<f32>(4.0), 3)) * 0.6;
            let e = cell_edge((p + warp) * 2.2 + s);
            let crack = smoothstep(0.06, 0.0, e) * smoothstep(1.0, 0.4, r);
            return vec2<f32>(crack, -crack);
        }
        case 5: { // puddle
            let m = smoothstep(0.85, 0.7, r + edge_noise * 0.7);
            return vec2<f32>(m, 0.0);
        }
        case 6: { // grunge / dirt
            let g = smoothstep(0.35, 0.75, fbm2(p * 2.0 + s, 5)) * smoothstep(1.0, 0.5, r);
            return vec2<f32>(g, g * 0.1);
        }
        case 7: { // paint splat
            let ang = atan2(p.y, p.x);
            let spikes = pow(abs(sin(ang * (5.0 + floor(hash21(s) * 4.0)) + seed)), 6.0) * 0.35;
            let m = smoothstep(0.62 + spikes, 0.55 + spikes, r + edge_noise * 0.3);
            return vec2<f32>(m, m * 0.15);
        }
        case 8: { // moss
            let g = smoothstep(0.4, 0.62, fbm2(p * 3.0 + s, 5)) * smoothstep(1.0, 0.55, r);
            let clumps = fbm2(p * 18.0 + s, 3);
            return vec2<f32>(g, g * clumps * 0.8);
        }
        case 9: { // rust
            let g = smoothstep(0.42, 0.7, fbm2(p * 2.5 + s, 6)) * smoothstep(1.0, 0.6, r);
            return vec2<f32>(g, g * fbm2(p * 25.0, 2) * 0.5);
        }
        case 10: { // leaks streaking down (-v)
            let x = uv.x * 12.0 + s.x;
            let lane = hash21(vec2<f32>(floor(x), seed));
            let len = 0.3 + 0.7 * lane;
            let streak = smoothstep(0.5, 0.0, abs(fract(x) - 0.5) * (1.6 + lane)) * smoothstep(len, 0.0, uv.y) * step(0.35, lane);
            return vec2<f32>(streak * (0.6 + 0.4 * fbm2(vec2<f32>(x, uv.y * 8.0), 2)), 0.0);
        }
        case 11: { // ring / glyph
            let ring = smoothstep(0.06, 0.0, abs(r - 0.7)) + smoothstep(0.03, 0.0, abs(r - 0.5));
            let spokes = smoothstep(0.04, 0.0, abs(sin(atan2(p.y, p.x) * 3.0)) * r) * step(r, 0.7) * step(0.2, r);
            let m = clamp(ring + spokes, 0.0, 1.0);
            return vec2<f32>(m, m * 0.2);
        }
        default: { return vec2<f32>(1.0, 0.0); }
    }
}

// Rain ripples on standing water: rings growing from random drops; returns the slope.
fn ripples(p: vec2<f32>, t: f32) -> vec2<f32> {
    var g = vec2<f32>(0.0);
    for (var k = 0; k < 3; k = k + 1) {
        let q = p * 3.0 + vec2<f32>(f32(k) * 7.31, f32(k) * 3.17);
        let cell = floor(q);
        let jitter = vec2<f32>(hash21(cell + vec2<f32>(1.7, f32(k))), hash21(cell + vec2<f32>(9.2, f32(k)))) - vec2<f32>(0.5);
        let f = fract(q) - vec2<f32>(0.5) - jitter * 0.5;
        let phase = fract(t * (0.7 + 0.3 * hash21(cell)) + hash21(cell + vec2<f32>(f32(k), 4.4)));
        let d = length(f);
        let ring = exp(-pow((d - phase * 0.48) * 22.0, 2.0)) * (1.0 - phase);
        g += normalize(f + vec2<f32>(1e-5)) * ring * sin((d - phase * 0.48) * 60.0);
    }
    return g;
}

// Screen-space reflection: walks the reflected ray through the opaque scene depth and returns the
// colour it meets (rgb) and how sure the hit is (a, 0 = nothing found on screen).
fn ssr(start: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    let dims = vec2<f32>(textureDimensions(scene_depth_tex));
    let cam = globals.camera_pos.xyz;
    var pos = start;
    var step = 0.08;
    for (var i = 0; i < 40; i = i + 1) {
        pos += dir * step;
        step *= 1.12;
        let c = globals.view_proj * vec4<f32>(pos, 1.0);
        if (c.w <= 0.0) { break; }
        let ndc = c.xyz / c.w;
        let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
        if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) { break; }
        let px = vec2<i32>(uv * dims);
        let d = textureLoad(scene_depth_tex, px, 0);
        let w = globals.inv_view_proj * vec4<f32>(ndc.xy, d, 1.0);
        let scene_p = w.xyz / w.w;
        let behind = distance(cam, pos) - distance(cam, scene_p);
        if (d < 1.0 && behind > 0.0 && behind < step * 2.5 + 0.1) {
            let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
            return vec4<f32>(textureLoad(scene_color_tex, px, 0).rgb, smoothstep(0.0, 0.08, edge) * (1.0 - f32(i) / 40.0));
        }
    }
    return vec4<f32>(0.0);
}

fn overlay(b: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    return select(vec3<f32>(1.0) - 2.0 * (vec3<f32>(1.0) - b) * (vec3<f32>(1.0) - d), 2.0 * b * d, b < vec3<f32>(0.5));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Derivatives need every pixel of a quad, so nothing is discarded until the very end.
    let dims = vec2<i32>(textureDimensions(scene_depth_tex));
    let px = vec2<i32>(in.clip.xy);
    let d = textureLoad(scene_depth_tex, clamp(px, vec2<i32>(0), max(dims - vec2<i32>(1), vec2<i32>(0))), 0);
    var keep = dims.x >= 2 && d < 1.0;
    let p = scene_point_behind(in.clip, in.world_pos);

    // Into the box: x, z across the decal, y along the projection.
    let origin = object.model[3].xyz;
    let ax = object.model[0].xyz;
    let ay = object.model[1].xyz;
    let az = object.model[2].xyz;
    let q = p - origin;
    let local = vec3<f32>(dot(q, ax) / dot(ax, ax), dot(q, ay) / dot(ay, ay), dot(q, az) / dot(az, az));
    keep = keep && all(abs(local) <= vec3<f32>(0.5));
    let uv = vec2<f32>(local.x + 0.5, 0.5 - local.z);

    // Surface normal from the depth (works on any mesh, smooth or not).
    var n = normalize(cross(dpdy(p), dpdx(p)));
    let v = normalize(globals.camera_pos.xyz - p);
    if (dot(n, v) < 0.0) { n = -n; }
    let up = normalize(ay);
    let facing = dot(n, up);
    let angle_fade = object.user0.z;
    let fade = smoothstep(angle_fade - 0.15, angle_fade + 0.1, facing) * (1.0 - smoothstep(0.38, 0.5, abs(local.y)));

    // Shape and colour.
    let shape = i32(object.user0.y + 0.5);
    var albedo = object.color.rgb;
    var alpha = object.color.a;
    var height = 0.0;
    let duv = vec2<f32>(length(dpdx(uv)), length(dpdy(uv)));
    let grad = min(duv, vec2<f32>(0.02));
    if (shape == 0) {
        let tex = textureSampleGrad(albedo_tex, albedo_smp, uv, vec2<f32>(grad.x, 0.0), vec2<f32>(0.0, grad.y));
        albedo *= tex.rgb;
        alpha *= tex.a;
    } else {
        let sh = decal_shape(uv, shape, object.user1.x);
        alpha *= sh.x;
        height = sh.y;
        if (shape == 3) { albedo = mix(albedo, albedo * 0.2, smoothstep(0.2, 0.1, length(uv - vec2<f32>(0.5)) * 2.0)); }
        if (shape == 8 || shape == 9) { albedo *= 0.7 + 0.6 * fbm2(uv * 30.0, 2); }
    }
    alpha *= fade;

    // Relief: the normal map, or the slope of the procedural height.
    let t = normalize(ax - n * dot(ax, n));
    let b = cross(n, t);
    let strength = object.user0.w;
    var n2 = n;
    if (object.user1.z > 0.5) {
        let m = textureSampleGrad(normal_tex, albedo_smp, uv, vec2<f32>(grad.x, 0.0), vec2<f32>(0.0, grad.y)).xyz * 2.0 - vec3<f32>(1.0);
        n2 = normalize(t * m.x * strength - b * m.y * strength + n * max(m.z, 0.05));
    } else if (shape != 0) {
        let hx = dpdx(height);
        let hy = dpdy(height);
        let px = dpdx(p);
        let py = dpdy(p);
        let grad_w = (cross(py, n) * hx + cross(n, px) * hy) / max(abs(dot(px, cross(py, n))), 1e-6);
        n2 = normalize(n - grad_w * strength * 0.15);
    }
    var rough = object.params.x;
    var metal = object.params2.x;
    var ao = 1.0;
    if (object.user1.w > 0.5) {
        let orm = textureSampleGrad(orm_tex, albedo_smp, uv, vec2<f32>(grad.x, 0.0), vec2<f32>(0.0, grad.y)).rgb;
        ao = orm.r;
        rough = orm.g;
        metal = orm.b;
    }

    let base = scene_color_behind(in.clip);
    // How the relief changes the light on the existing surface (used by every mode).
    let flat_light = lighting_pbr(p, n, vec3<f32>(0.5), 0.8, 0.0, 1.0);
    let bumped_light = lighting_pbr(p, n2, vec3<f32>(0.5), 0.8, 0.0, ao);
    let relief = clamp(dot(bumped_light, vec3<f32>(0.33)) / max(dot(flat_light, vec3<f32>(0.33)), 1e-3), 0.3, 2.5);
    let lit = to_display_at(apply_fog(lighting_pbr(p, n2, albedo, rough, metal, ao) + albedo * object.user1.y, p), in.clip);
    let tint = albedo;
    var out = lit;
    switch i32(object.user0.x + 0.5) {
        case 1: { out = base * tint * relief; }
        case 2: { out = (vec3<f32>(1.0) - (vec3<f32>(1.0) - base) * (vec3<f32>(1.0) - tint)) * relief; }
        case 3: { out = max(base, lit); }
        case 4: { out = min(base, lit); }
        case 5: { out = overlay(base, tint) * relief; }
        case 6: { out = base + lit; }
        case 7: { out = mix(base * (base + 2.0 * tint * (vec3<f32>(1.0) - base)), base, 0.0) * relief; }
        case 8: { out = base * relief; }
        case 9: { out = base / max(vec3<f32>(1.0) - tint, vec3<f32>(0.02)); }
        case 10: { out = vec3<f32>(1.0) - (vec3<f32>(1.0) - base) / max(tint, vec3<f32>(0.02)); }
        case 11: {
            let l = dot(base, vec3<f32>(0.2126, 0.7152, 0.0722));
            out = tint / max(dot(tint, vec3<f32>(0.2126, 0.7152, 0.0722)), 1e-3) * l * relief;
        }
        default: {}
    }
    // Standing water: dark wet ground with real reflections of what is on screen (street lamps,
    // signs) and rings from falling drops.
    if (shape == 5) {
        let rip = ripples(p.xz, globals.time.x) * strength;
        let wn = normalize(n + (t * rip.x + b * rip.y) * 0.18);
        let r = reflect(-v, wn);
        let hit = ssr(p + n * 0.02, r);
        let sky = to_display_at(env_color(vec3<f32>(r.x, abs(r.y), r.z)), in.clip);
        let refl = mix(sky * 0.6, hit.rgb, hit.a);
        let fres = 0.04 + 0.96 * pow(1.0 - clamp(dot(wn, v), 0.0, 1.0), 5.0);
        let wet = base * albedo * 1.6;
        out = mix(wet, refl, clamp(fres + 0.35, 0.0, 1.0));
    }
    // Glow is added whatever the mode.
    out += to_display_at(albedo * object.user1.y, in.clip) * select(1.0, 0.0, i32(object.user0.x + 0.5) == 0);
    if (!keep || alpha < 0.003) { discard; }
    return vec4<f32>(clamp(out, vec3<f32>(0.0), vec3<f32>(64.0)), clamp(alpha, 0.0, 1.0));
}
