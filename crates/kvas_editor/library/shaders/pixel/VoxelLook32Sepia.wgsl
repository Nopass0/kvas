// Pixel-art look for a 3D object: light per 0.031 m world cell, 4 bands of a Sepia palette, ordered dither.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const RES: f32 = 32.0;
const MIX: f32 = 0.5;
const P0: vec3<f32> = vec3<f32>(0.1, 0.05, 0.02);
const P1: vec3<f32> = vec3<f32>(0.4, 0.25, 0.12);
const P2: vec3<f32> = vec3<f32>(0.75, 0.55, 0.3);
const P3: vec3<f32> = vec3<f32>(1.0, 0.9, 0.65);
fn ign(p: vec2<f32>) -> f32 { return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715)))); }

@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let cell = floor(in.world_pos * RES);
    let sun = normalize(globals.sun_dir.xyz);
    let shadow = shadow_factor((cell + 0.5) / RES, n);
    let l = clamp(max(dot(n, sun), 0.0) * shadow * 0.8 + 0.2 + (ign(floor(in.clip.xy)) - 0.5) * 0.15, 0.0, 1.0);
    let i = floor(l * 3.999);
    var pal = P0;
    if (i > 0.5) { pal = P1; }
    if (i > 1.5) { pal = P2; }
    if (i > 2.5) { pal = P3; }
    let col = mix(pal, pal * object.color.rgb * 1.6, MIX);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
