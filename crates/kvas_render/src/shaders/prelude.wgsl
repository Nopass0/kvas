// Engine prelude, prepended to every mesh shader (built-in and custom).
// A custom shader only has to define `vs_main` and `fs_main`; it can reuse
// `standard_vertex`, `lighting` / `lighting_ex` and `to_display`, or ignore them.

struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    light_vp: mat4x4<f32>,   // sun shadow map projection
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,      // direction towards the sun (or moon at night)
    sun_color: vec4<f32>,    // rgb premultiplied by intensity
    ambient: vec4<f32>,
    sky: vec4<f32>,
    time: vec4<f32>,         // x = seconds since start, y = frame delta, z = exposure
    counts: vec4<f32>,       // x = local lights, y = toon bands, z = saturation, w = contrast
    shadow: vec4<f32>,       // x = enabled, y = depth bias, z = world size of a shadow texel, w = 1 / map size
    sky_a: vec4<f32>,        // x = hour of day, y = cloud cover, z = star brightness, w = procedural sky enabled
    sky_b: vec4<f32>,        // x = cloud speed, y = debug view mode, z = PSX look
    sky_c: vec4<f32>,        // x = haze (turbidity), y = moon phase (0.5 full), z = aurora, w = milky way
    environment: vec4<f32>, // x enabled, y rotation radians, z intensity
    viewport: vec4<f32>,    // physical width/height, reciprocal width/height
    point_pos: array<vec4<f32>, 8>,    // xyz position, w range
    point_color: array<vec4<f32>, 8>,  // rgb premultiplied by intensity, w = cos(inner cone) for spots
    point_dir: array<vec4<f32>, 8>,    // xyz spot direction, w = cos(outer cone); w < -1.5 means point light
};

struct Object {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    color: vec4<f32>,        // linear rgba
    params: vec4<f32>,       // x roughness, y emissive, z uv tiling, w triplanar scale (0 = off)
    params2: vec4<f32>,      // x metallic, y reflectivity, z PBR maps present, w normal strength
    user0: vec4<f32>,        // MeshRenderer.user[0..4]: free parameters for custom shaders
    user1: vec4<f32>,        // MeshRenderer.user[4..8]
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var shadow_map: texture_depth_2d;
@group(0) @binding(2) var shadow_smp: sampler_comparison;
@group(0) @binding(3) var environment_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> object: Object;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_smp: sampler;
@group(2) @binding(2) var volume_tex: texture_3d<f32>;
@group(2) @binding(3) var normal_tex: texture_2d<f32>;   // tangent-space normal map (OpenGL, green up)
@group(2) @binding(4) var orm_tex: texture_2d<f32>;      // R occlusion, G roughness, B metallic
// Depth of the opaque scene; filled for transparent objects only (water, glass), empty otherwise.
@group(3) @binding(0) var scene_depth_tex: texture_depth_multisampled_2d;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

// Distance from a transparent surface point to the opaque surface behind it, in metres
// (1000 where nothing is behind, or for opaque objects). Water uses it for shore and contact foam.
fn scene_depth_behind(clip: vec4<f32>, world_pos: vec3<f32>) -> f32 {
    let dims = vec2<i32>(textureDimensions(scene_depth_tex));
    let px = vec2<i32>(clip.xy);
    if (dims.x < 2 || px.x < 0 || px.y < 0 || px.x >= dims.x || px.y >= dims.y) { return 1000.0; }
    let d = textureLoad(scene_depth_tex, px, 0);
    if (d >= 1.0) { return 1000.0; }
    let uv = clip.xy / vec2<f32>(dims);
    let ndc = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    let w = globals.inv_view_proj * ndc;
    return distance(w.xyz / w.w, world_pos);
}

// World position of the opaque surface seen through this pixel (the point itself where nothing is behind).
fn scene_point_behind(clip: vec4<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let dims = vec2<i32>(textureDimensions(scene_depth_tex));
    let px = vec2<i32>(clip.xy);
    if (dims.x < 2 || px.x < 0 || px.y < 0 || px.x >= dims.x || px.y >= dims.y) { return world_pos; }
    let d = textureLoad(scene_depth_tex, px, 0);
    if (d >= 1.0) { return world_pos; }
    let uv = clip.xy / vec2<f32>(dims);
    let w = globals.inv_view_proj * vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    return w.xyz / w.w;
}

fn standard_vertex(v: VertexIn) -> VsOut {
    var out: VsOut;
    let world = object.model * vec4<f32>(v.position, 1.0);
    out.clip = globals.view_proj * world;
    if (globals.sky_b.z > 0.5) {
        // PSX: vertices snap to a coarse screen grid, which makes geometry wobble as it moves.
        let ndc = out.clip.xy / out.clip.w;
        out.clip = vec4<f32>(round(ndc * vec2<f32>(160.0, 120.0)) / vec2<f32>(160.0, 120.0) * out.clip.w, out.clip.zw);
    }
    out.world_pos = world.xyz;
    out.normal = normalize((object.normal_matrix * vec4<f32>(v.normal, 0.0)).xyz);
    out.uv = v.uv;
    return out;
}

// ---------------------------------------------------------------- procedural sky

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash21(i), hash21(i + vec2<f32>(1.0, 0.0)), u.x), mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

fn value_noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(hash31(i), hash31(i + vec3<f32>(1.0, 0.0, 0.0)), u.x), mix(hash31(i + vec3<f32>(0.0, 1.0, 0.0)), hash31(i + vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y);
    let b = mix(mix(hash31(i + vec3<f32>(0.0, 0.0, 1.0)), hash31(i + vec3<f32>(1.0, 0.0, 1.0)), u.x), mix(hash31(i + vec3<f32>(0.0, 1.0, 1.0)), hash31(i + vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y);
    return mix(a, b, u.z);
}

fn fbm2(p: vec2<f32>, octaves: i32) -> f32 {
    var amp = 0.5;
    var sum = 0.0;
    var q = p;
    for (var i = 0; i < octaves; i = i + 1) {
        sum += amp * value_noise(q);
        q = q * 2.03 + vec2<f32>(17.1, 9.2);
        amp *= 0.5;
    }
    return sum;
}

fn fbm3(p: vec3<f32>, octaves: i32) -> f32 {
    var amp = 0.5;
    var sum = 0.0;
    var q = p;
    for (var i = 0; i < octaves; i = i + 1) {
        sum += amp * value_noise3(q);
        q = q * 2.07 + vec3<f32>(11.3, 5.7, 8.1);
        amp *= 0.5;
    }
    return sum;
}

// Same formula as `kvas_engine::sky::sun_direction`.
fn sky_sun_direction() -> vec3<f32> {
    let a = (globals.sky_a.x - 6.0) / 24.0 * 6.2831853;
    return normalize(vec3<f32>(cos(a), sin(a), 0.35));
}

// Fraction of the moon disc that is lit (0 new, 1 full); same as `kvas_engine::sky::moon_illumination`.
fn moon_illumination() -> f32 {
    return 0.5 - 0.5 * cos(6.2831853 * globals.sky_c.y);
}

// Rayleigh optical depth of the whole atmosphere at the zenith, per colour channel.
const TAU_R: vec3<f32> = vec3<f32>(0.0464, 0.108, 0.265);

fn mie_depth() -> f32 {
    return 0.012 + 0.16 * globals.sky_c.x;
}

// Relative path length through the atmosphere for a ray at height `y` (sine of the elevation).
fn air_mass(y: f32) -> f32 {
    let c = max(y, 0.0);
    return 1.0 / (c + 0.025 * exp(-11.0 * c) + 0.012);
}

// Colour of direct light after crossing the atmosphere: white overhead, red at the horizon.
fn sun_transmittance(light_y: f32) -> vec3<f32> {
    return exp(-(TAU_R + vec3<f32>(mie_depth() * 0.9)) * air_mass(light_y)) * smoothstep(-0.2, 0.0, light_y);
}

// Single-scattering sky: Rayleigh (blue sky, red sunsets) plus Mie (haze and the glow around the sun).
fn sky_scatter(dir: vec3<f32>, light: vec3<f32>, intensity: f32) -> vec3<f32> {
    let mu = dot(dir, light);
    let y = max(dir.y, 0.0);
    let am = air_mass(y);
    let tm = mie_depth();
    let k_r = vec3<f32>(1.0) - exp(-TAU_R * am * 0.6);
    let k_m = 1.0 - exp(-tm * am);
    let phase_r = 0.0596831 * (1.0 + mu * mu);
    let g = 0.76;
    let phase_m = (1.0 - g * g) / (12.566 * pow(1.0 + g * g - 2.0 * g * mu, 1.5));
    let horizon = mix(0.32, 1.0, smoothstep(0.0, 0.4, y));
    return intensity * sun_transmittance(light.y) * (k_r * phase_r + vec3<f32>(k_m * phase_m * 0.03)) * horizon;
}

fn rotate_axis(v: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return v * c + cross(axis, v) * s + axis * dot(axis, v) * (1.0 - c);
}

// One layer of round, twinkling, colour-tinted stars; `scale` = cells per unit direction.
fn star_layer(d: vec3<f32>, scale: f32, density: f32, seed: f32) -> vec3<f32> {
    let sp = d * scale;
    let cell = floor(sp) + vec3<f32>(seed);
    let r = hash31(cell);
    let jitter = vec3<f32>(hash31(cell + vec3<f32>(1.7, 9.2, 3.1)), hash31(cell + vec3<f32>(8.3, 2.8, 6.4)), hash31(cell + vec3<f32>(4.1, 7.5, 0.9)));
    let twinkle = 0.7 + 0.3 * sin(globals.time.x * 3.0 + r * 60.0);
    let mag = hash31(cell + vec3<f32>(3.1, 1.7, 5.3));
    let star = step(1.0 - density, r) * smoothstep(0.24, 0.0, length(fract(sp) - jitter)) * (0.25 + 0.75 * mag * mag) * twinkle;
    let tint = mix(vec3<f32>(1.0, 0.72, 0.5), vec3<f32>(0.62, 0.78, 1.0), hash31(cell + vec3<f32>(9.1, 4.4, 2.2)));
    return mix(vec3<f32>(1.0), tint, 0.7) * star;
}

// The moon as a shaded, cratered ball that waxes and wanes with `moon_phase`.
fn moon_disc(dir: vec3<f32>, moon: vec3<f32>) -> vec3<f32> {
    let md = dot(dir, moon);
    let radius = 0.0447;   // sin of the angular radius (about 2.5 degrees, larger than the real moon for looks)
    if (md < 0.99) {
        return vec3<f32>(0.0);
    }
    let t = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), moon));
    let b = cross(moon, t);
    let off = dir - moon * md;
    let u = dot(off, t) / radius;
    let v = dot(off, b) / radius;
    let rr = u * u + v * v;
    if (rr >= 1.0) {
        return vec3<f32>(0.0);
    }
    let w = sqrt(1.0 - rr);
    let a = (globals.sky_c.y - 0.5) * 6.2831853;
    let lit = smoothstep(-0.02, 0.1, u * sin(a) + w * cos(a));
    let maria = smoothstep(0.42, 0.62, fbm2(vec2<f32>(u, v) * 2.2 + vec2<f32>(4.0, 1.0), 4));
    let craters = fbm2(vec2<f32>(u, v) * 11.0, 3);
    let albedo = mix(0.95, 0.5, maria) * (0.78 + 0.35 * craters);
    return vec3<f32>(1.0, 0.97, 0.88) * albedo * (lit * 0.62 + 0.02) * smoothstep(1.0, 0.9, sqrt(rr));
}

// Stars, milky way, moon and aurora: the night part of the sky in direction `dir`.
fn night_sky(dir: vec3<f32>, moon: vec3<f32>, night: f32) -> vec3<f32> {
    let axis = normalize(vec3<f32>(0.0, 0.8, -0.6));
    let d = rotate_axis(dir, axis, globals.sky_a.x / 24.0 * 6.2831853);
    let ill = moon_illumination();
    let moon_up = smoothstep(-0.1, 0.2, moon.y);
    let wash = 1.0 - 0.75 * ill * moon_up * smoothstep(0.2, 1.0, dot(dir, moon) * 0.5 + 0.5);
    let visible = night * smoothstep(0.0, 0.1, dir.y) * wash;

    let pole = normalize(vec3<f32>(0.35, 0.8, 0.5));
    let lat = dot(d, pole);
    let band = exp(-lat * lat * 26.0);
    let core = pow(max(dot(d, normalize(vec3<f32>(0.6, -0.1, 0.8))), 0.0), 3.0);
    let dust = smoothstep(0.45, 0.78, fbm3(d * 9.0 + vec3<f32>(7.7), 4));
    let cloudy = 0.3 + 0.7 * fbm3(d * 5.0, 4);
    let milky = band * cloudy * (1.0 - 0.75 * dust) * (0.45 + 1.3 * core);
    var col = mix(vec3<f32>(0.5, 0.62, 1.0), vec3<f32>(1.0, 0.82, 0.62), core) * milky * 0.2 * globals.sky_c.w;

    let boost = 1.0 + 1.5 * band * globals.sky_c.w;
    col += (star_layer(d, 90.0, 0.2 * boost, 0.0) * 1.0 + star_layer(d, 230.0, 0.1 * boost, 3.0) * 1.4 + star_layer(d, 520.0, 0.06 * boost, 7.0) * 1.2) * globals.sky_a.z;
    col *= visible;

    // Moon: disc plus a soft halo.
    let md = dot(dir, moon);
    col += moon_disc(dir, moon) * moon_up;
    col += vec3<f32>(0.55, 0.62, 0.8) * pow(max(md, 0.0), 160.0) * 0.18 * ill * moon_up;
    col += vec3<f32>(0.5, 0.58, 0.8) * pow(max(md, 0.0), 12.0) * 0.012 * ill * moon_up;

    // Aurora: a few glowing ribbons whose rays flicker.
    let strength = globals.sky_c.z;
    if (strength > 0.01 && dir.y > 0.04) {
        var aur = vec3<f32>(0.0);
        let time = globals.time.x;
        for (var i = 0; i < 10; i = i + 1) {
            let layer = f32(i);
            let q = dir.xz * ((1.0 + layer * 0.16) / dir.y);
            let path = sin(q.x * 0.22 + time * 0.05) * 2.2 + sin(q.x * 0.61 - time * 0.03) * 0.8 - 5.5;
            let dist = (q.y - path) / 1.9;
            let ribbon = exp(-dist * dist);
            let rays = 0.45 + 0.55 * value_noise(vec2<f32>(q.x * 7.0 + layer * 3.0, time * 0.5 + layer));
            let tint = mix(vec3<f32>(0.1, 1.0, 0.42), vec3<f32>(0.62, 0.22, 0.95), smoothstep(2.0, 9.0, layer));
            aur += tint * ribbon * rays * exp(-layer * 0.18);
        }
        col += aur * 0.07 * strength * night * smoothstep(0.04, 0.3, dir.y);
    }
    return col;
}

// Two cloud decks: puffy, sun-lit cumulus low down and thin streaky cirrus high up.
fn sky_clouds(col_in: vec3<f32>, dir: vec3<f32>, sun: vec3<f32>) -> vec3<f32> {
    var col = col_in;
    let cover_amount = globals.sky_a.y;
    let drift = globals.time.x * globals.sky_b.x;
    let wind = vec2<f32>(drift, drift * 0.4);
    let moon = -sun;
    let sun_col = sun_transmittance(sun.y) * 1.7 + sun_transmittance(moon.y) * vec3<f32>(0.5, 0.6, 1.0) * (0.04 + 0.2 * moon_illumination());
    let sd = dot(dir, sun);
    let up = max(dir.y, 0.0);

    // Cirrus: stretched noise, mostly a glow that picks up sunset colours.
    let cu = dir.xz / (dir.y + 0.25);
    let cp = vec2<f32>(cu.x * 0.45 + wind.x * 0.6, cu.y * 2.6 + wind.y);
    let streak = smoothstep(0.5, 0.95, fbm2(cp * 1.6, 4)) * mix(0.35, 0.95, cover_amount);
    col += sun_col * streak * (0.3 + 0.7 * pow(max(sd, 0.0), 3.0)) * 0.55 * smoothstep(0.02, 0.3, up);

    // Cumulus.
    let uv = dir.xz / (dir.y + 0.1) * 0.6 + wind;
    let warp = vec2<f32>(fbm2(uv * 0.7 + vec2<f32>(3.0, wind.x), 3), fbm2(uv * 0.7 + vec2<f32>(wind.y, 7.0), 3)) - vec2<f32>(0.5);
    let base = fbm2(uv + warp * 0.9, 6);
    let detail = fbm2(uv * 3.3 + warp * 2.0 + vec2<f32>(wind.y, 2.0), 4);
    let shape = base + 0.32 * (detail - 0.5);
    let threshold = mix(0.74, 0.38, cover_amount);
    let thick = smoothstep(threshold, threshold + 0.24, shape);
    if (thick > 0.001 && dir.y > 0.005) {
        // Self shadowing: look a little towards the sun; denser cloud there means this spot is shaded.
        let to_sun = normalize(sun.xz + vec2<f32>(0.0001, 0.0)) * 0.09;
        let shape_sun = fbm2(uv + to_sun, 3) + 0.32 * (detail - 0.5);
        let shade = clamp(0.62 + (shape - shape_sun) * 5.0, 0.0, 1.0);
        let sky_amb = sky_scatter(vec3<f32>(0.0, 1.0, 0.0), sun, 40.0) * 1.3 + vec3<f32>(0.03, 0.04, 0.07);
        let core = 1.0 - 0.55 * thick;
        let silver = pow(max(sd, 0.0), 9.0) * (1.0 - thick) * 3.0 + pow(1.0 - thick, 2.0) * pow(max(sd, 0.0), 4.0) * 2.2;
        var cloud = sky_amb * vec3<f32>(0.88, 0.94, 1.06) * (0.34 + 0.66 * core) + sun_col * shade * (0.55 + 0.45 * core + silver);
        // Distant clouds fade into the haze at the horizon.
        let haze = exp(-up * 5.5) * 0.85;
        cloud = mix(cloud, col, haze);
        col = mix(col, cloud, thick * smoothstep(0.005, 0.12, up));
    }
    return col;
}

// Linear radiance of the procedural sky in direction `dir`. `with_detail` adds the expensive parts
// (stars, moon, aurora, clouds); reflections and ambient use the cheap variant.
fn sky_radiance(dir: vec3<f32>, with_detail: bool) -> vec3<f32> {
    let sun = sky_sun_direction();
    let moon = -sun;
    let ill = moon_illumination();
    let day = smoothstep(-0.12, 0.2, sun.y);
    let night = 1.0 - smoothstep(-0.3, 0.0, sun.y);
    let sdir = normalize(vec3<f32>(dir.x, max(dir.y, 0.0) + 0.0001, dir.z));

    var col = sky_scatter(sdir, sun, 40.0) + sky_scatter(sdir, moon, 0.9 * (0.06 + 0.94 * ill)) * vec3<f32>(0.55, 0.8, 1.25);
    col += vec3<f32>(0.003, 0.006, 0.014) * night;

    // Twilight: warm glow at the horizon under the sun, a purple belt on the opposite side.
    let sd = dot(dir, sun);
    let twilight = smoothstep(-0.25, 0.0, sun.y) * (1.0 - smoothstep(0.0, 0.22, sun.y));
    let low = exp(-max(dir.y, 0.0) * 6.0);
    col += vec3<f32>(0.5, 0.17, 0.1) * twilight * pow(max(sd, 0.0), 2.0) * low * 1.1;
    col += vec3<f32>(0.2, 0.1, 0.3) * twilight * low * (0.5 - 0.5 * sd) * 0.3;

    // Sun disc and bloom.
    let tr = sun_transmittance(sun.y);
    col += tr * 70.0 * smoothstep(0.99992, 0.99997, sd) * smoothstep(-0.03, 0.02, dir.y);
    col += tr * pow(max(sd, 0.0), 1800.0) * 2.5;

    if (with_detail) {
        col += night_sky(dir, moon, night);
        col = sky_clouds(col, dir, sun);
    }
    if (dir.y < 0.0) {
        // Below the horizon: fog-coloured ground that gets darker with depth.
        let ground = (col + vec3<f32>(0.01)) * 0.22;
        col = mix(col, ground, smoothstep(0.0, -0.12, dir.y));
    }
    return col;
}

// What a mirror-like surface sees in direction `dir`.
fn env_color(dir: vec3<f32>) -> vec3<f32> {
    if globals.environment.x > 0.5 { return hdri_color(dir); }
    if (globals.sky_a.w > 0.5) {
        return sky_radiance(dir, false);
    }
    let sky_lin = pow(globals.sky.xyz, vec3<f32>(2.2));
    return mix(globals.ambient.xyz * 1.2, sky_lin, clamp(dir.y * 0.5 + 0.5, 0.0, 1.0));
}

// Linear HDR radiance, bilinear horizontally wrapped equirectangular coordinates.
fn hdri_texel(p: vec2<i32>) -> vec3<f32> {
    let size = vec2<i32>(textureDimensions(environment_tex));
    let x = ((p.x % size.x) + size.x) % size.x;
    return textureLoad(environment_tex, vec2<i32>(x, clamp(p.y, 0, size.y - 1)), 0).rgb;
}
fn hdri_color(direction: vec3<f32>) -> vec3<f32> {
    let d = normalize(direction);
    let uv = vec2<f32>(fract(atan2(d.z, d.x) / 6.2831853 + 0.5 + globals.environment.y / 6.2831853), acos(clamp(d.y, -1.0, 1.0)) / 3.14159265);
    let p = uv * vec2<f32>(textureDimensions(environment_tex)) - 0.5;
    let cell = vec2<i32>(floor(p)); let f = fract(p);
    return mix(mix(hdri_texel(cell), hdri_texel(cell + vec2<i32>(1, 0)), f.x), mix(hdri_texel(cell + vec2<i32>(0, 1)), hdri_texel(cell + vec2<i32>(1, 1)), f.x), f.y) * globals.environment.z;
}

// ---------------------------------------------------------------- lighting

// Cel shading: snaps a 0..1 light amount to `toon_bands` soft steps (off when 0).
fn quantize(x: f32) -> f32 {
    let bands = globals.counts.y;
    if (bands < 1.0) {
        return x;
    }
    let s = x * bands;
    return (floor(s) + smoothstep(0.0, 0.1, fract(s))) / bands;
}

fn blinn(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, shininess: f32) -> vec2<f32> {
    let diffuse = max(dot(n, l), 0.0);
    let h = normalize(l + v);
    var spec = select(0.0, pow(max(dot(n, h), 0.0), shininess), diffuse > 0.0);
    if (globals.counts.y >= 1.0) {
        spec = smoothstep(0.45, 0.55, spec) * 0.8;
    }
    return vec2<f32>(quantize(diffuse), spec);
}

// 0 = fully in shadow, 1 = lit. PCF over a 5x5 kernel whose spread follows the scene's shadow softness,
// blended by the shadow strength.
fn shadow_factor(p: vec3<f32>, n: vec3<f32>) -> f32 {
    if (globals.shadow.x < 0.5) {
        return 1.0;
    }
    let lp = globals.light_vp * vec4<f32>(p + n * globals.shadow.z * 1.5, 1.0);
    let uv = vec2<f32>(lp.x * 0.5 + 0.5, 0.5 - lp.y * 0.5);
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || lp.z > 1.0) {
        return 1.0;
    }
    let depth = lp.z - globals.shadow.y;
    let spread = globals.sky_b.w * 0.6;
    var sum = 0.0;
    for (var dy = -2; dy <= 2; dy = dy + 1) {
        for (var dx = -2; dx <= 2; dx = dx + 1) {
            sum += textureSampleCompareLevel(shadow_map, shadow_smp, uv + vec2<f32>(f32(dx), f32(dy)) * globals.shadow.w * spread, depth);
        }
    }
    return mix(1.0, sum / 25.0, globals.time.w);
}

// Direct light only: sun with shadows, point and spot lights. Returns lit colour.
fn lighting_direct(p: vec3<f32>, n: vec3<f32>, base: vec3<f32>, roughness: f32) -> vec3<f32> {
    let v = normalize(globals.camera_pos.xyz - p);
    let shininess = mix(96.0, 4.0, clamp(roughness, 0.0, 1.0));
    let spec_strength = (1.0 - clamp(roughness, 0.0, 1.0)) * 0.5;
    var light = mix(globals.ambient.xyz * 0.6, globals.ambient.xyz + globals.sky.xyz * 0.15, n.y * 0.5 + 0.5);
    var spec = vec3<f32>(0.0);

    let shadow = shadow_factor(p, n);
    let sun = blinn(n, normalize(globals.sun_dir.xyz), v, shininess);
    light += globals.sun_color.xyz * sun.x * shadow;
    spec += globals.sun_color.xyz * sun.y * spec_strength * shadow;

    let count = i32(globals.counts.x);
    for (var i = 0; i < count; i = i + 1) {
        let to_light = globals.point_pos[i].xyz - p;
        let dist = length(to_light);
        let range = max(globals.point_pos[i].w, 0.001);
        var atten = pow(clamp(1.0 - dist / range, 0.0, 1.0), 2.0);
        let l = to_light / max(dist, 0.0001);
        if (globals.point_dir[i].w > -1.5) {
            // spot light: smooth falloff between the inner and outer cone
            atten *= smoothstep(globals.point_dir[i].w, globals.point_color[i].w, dot(-l, globals.point_dir[i].xyz));
        }
        let r = blinn(n, l, v, shininess);
        light += globals.point_color[i].xyz * r.x * atten;
        spec += globals.point_color[i].xyz * r.y * atten * spec_strength;
    }
    return base * light + spec;
}

// Lit colour plus mirror reflections of the sky (Fresnel weighted). `.w` is the reflection amount,
// which the built-in shader adds to alpha so glass becomes visible at grazing angles.
fn lighting_ex(p: vec3<f32>, n: vec3<f32>, base: vec3<f32>, roughness: f32, metallic: f32, reflectivity: f32) -> vec4<f32> {
    let direct = lighting_direct(p, n, base, roughness);
    let v = normalize(globals.camera_pos.xyz - p);
    let fres = 0.04 + 0.96 * pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 5.0);
    let k = clamp(reflectivity + metallic, 0.0, 1.0);
    let amount = k * mix(fres, 1.0, metallic * 0.8);
    var env = env_color(reflect(-v, n));
    env = mix(env, mix(globals.ambient.xyz, env, 0.35), clamp(roughness * 1.2, 0.0, 1.0));
    let body = direct * (1.0 - metallic * 0.85);
    return vec4<f32>(mix(body, env * mix(vec3<f32>(1.0), base, metallic), amount), amount);
}

// ---------------------------------------------------------------- physically based shading

const PI: f32 = 3.14159265;

// Cook-Torrance (GGX) term for one light. `radiance` is in the engine's light units.
fn pbr_term(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, albedo: vec3<f32>, rough: f32, metal: f32, f0: vec3<f32>, radiance: vec3<f32>) -> vec3<f32> {
    let h = normalize(l + v);
    let nl = max(dot(n, l), 0.0);
    let nv = max(dot(n, v), 1e-3);
    let nh = max(dot(n, h), 0.0);
    let vh = max(dot(v, h), 0.0);
    let a = max(rough * rough, 0.003);
    let a2 = a * a;
    let d = nh * nh * (a2 - 1.0) + 1.0;
    let ndf = a2 / (PI * d * d + 1e-6);
    let k = (rough + 1.0) * (rough + 1.0) / 8.0;
    let g = (nv / (nv * (1.0 - k) + k)) * (nl / (nl * (1.0 - k) + k));
    let fresnel = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - vh, 5.0);
    let spec = ndf * g * fresnel / (4.0 * nv * nl + 1e-4);
    let kd = (vec3<f32>(1.0) - fresnel) * (1.0 - metal);
    return (kd * albedo / PI + spec) * radiance * nl;
}

// Full PBR: sun with shadows, point / spot lights, hemisphere ambient and sky reflections.
fn lighting_pbr(p: vec3<f32>, n: vec3<f32>, albedo: vec3<f32>, rough_in: f32, metal: f32, ao: f32) -> vec3<f32> {
    let rough = clamp(rough_in, 0.04, 1.0);
    let v = normalize(globals.camera_pos.xyz - p);
    let f0 = mix(vec3<f32>(0.04), albedo, metal);
    var col = pbr_term(n, normalize(globals.sun_dir.xyz), v, albedo, rough, metal, f0, globals.sun_color.xyz * PI * shadow_factor(p, n));
    let count = i32(globals.counts.x);
    for (var i = 0; i < count; i = i + 1) {
        let to_light = globals.point_pos[i].xyz - p;
        let dist = length(to_light);
        let range = max(globals.point_pos[i].w, 0.001);
        var atten = pow(clamp(1.0 - dist / range, 0.0, 1.0), 2.0);
        let l = to_light / max(dist, 0.0001);
        if (globals.point_dir[i].w > -1.5) {
            atten *= smoothstep(globals.point_dir[i].w, globals.point_color[i].w, dot(-l, globals.point_dir[i].xyz));
        }
        col += pbr_term(n, l, v, albedo, rough, metal, f0, globals.point_color[i].xyz * PI * atten);
    }
    let nv = max(dot(n, v), 1e-3);
    let hemi = mix(globals.ambient.xyz * 0.6, globals.ambient.xyz + globals.sky.xyz * 0.15, n.y * 0.5 + 0.5);
    var env = env_color(reflect(-v, n));
    env = mix(env, mix(globals.ambient.xyz, env, 0.3), rough);
    let f_env = f0 + (max(vec3<f32>(1.0 - rough), f0) - f0) * pow(1.0 - nv, 5.0);
    col += albedo * (1.0 - metal) * hemi * ao + env * f_env * ao;
    return col;
}

// Simple Blinn-Phong with hemisphere ambient, sun shadows and local lights. `base` is linear colour.
fn lighting(p: vec3<f32>, n: vec3<f32>, base: vec3<f32>, roughness: f32) -> vec3<f32> {
    return lighting_ex(p, n, base, roughness, 0.0, 0.0).xyz;
}

// Linear light -> screen colour, with the scene's exposure, saturation and contrast.
// ACES filmic curve (Narkowicz fit): soft highlight roll-off and a touch of contrast.
fn aces_film(x: vec3<f32>) -> vec3<f32> {
    return clamp((x * (2.51 * x + vec3<f32>(0.03))) / (x * (2.43 * x + vec3<f32>(0.59)) + vec3<f32>(0.14)), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Distance haze towards the colour of the sky behind the point (aerial perspective).
fn apply_fog(col: vec3<f32>, world: vec3<f32>) -> vec3<f32> {
    let density = globals.sun_dir.w;
    if (density <= 0.0) {
        return col;
    }
    let to_p = world - globals.camera_pos.xyz;
    let dist = length(to_p);
    let dir = to_p / max(dist, 1e-4);
    // Thinner with height, so hills and rooftops clear the haze.
    let thin = exp(-max(world.y, 0.0) * 0.045);
    let amount = 1.0 - exp(-dist * density * thin);
    let haze = env_color(vec3<f32>(dir.x, max(dir.y, 0.02), dir.z));
    return mix(col, haze, clamp(amount, 0.0, 0.92));
}

fn to_display_at(linear: vec3<f32>, pos: vec4<f32>) -> vec3<f32> {
    var lin = max(linear * globals.time.z, vec3<f32>(0.0));
    lin = mix(lin, aces_film(lin * 0.9) * 1.04, globals.sky.w);
    var c = pow(lin, vec3<f32>(1.0 / 2.2));
    let luma = dot(c, vec3<f32>(0.299, 0.587, 0.114));
    c = mix(vec3<f32>(luma), c, globals.counts.z);
    c = (c - vec3<f32>(0.5)) * globals.counts.w + vec3<f32>(0.5);
    c = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    if (globals.sky_b.z > 0.5) {
        // PSX: 15-bit colour with a 4x4 ordered dither.
        let q = vec2<u32>(vec2<f32>(pos.xy)) & vec2<u32>(3u);
        let bayer = (f32((q.x * 5u + q.y * 3u) & 15u) / 16.0 - 0.5) / 32.0;
        c = floor((c + vec3<f32>(bayer)) * 31.0 + vec3<f32>(0.5)) / 31.0;
    }
    return c;
}

fn to_display(linear: vec3<f32>) -> vec3<f32> {
    return to_display_at(linear, vec4<f32>(0.0));
}
