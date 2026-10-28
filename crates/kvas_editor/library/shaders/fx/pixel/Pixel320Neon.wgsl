// Pixel art at 320 columns with a 4-colour Neon palette and ordered dithering. Works on any 3D scene.
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"
const CELLS: f32 = 320.0;
const DITHER: f32 = 0.18;
const P0: vec3<f32> = vec3<f32>(0.02, 0.0, 0.1);
const P1: vec3<f32> = vec3<f32>(0.5, 0.0, 0.8);
const P2: vec3<f32> = vec3<f32>(0.0, 0.8, 1.0);
const P3: vec3<f32> = vec3<f32>(1.0, 0.9, 1.0);
fn ign(p: vec2<f32>) -> f32 { return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715)))); }

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let grid = vec2<f32>(CELLS, CELLS / screen_aspect());
    let cell = floor(uv * grid);
    let c = scene_color((cell + 0.5) / grid);
    let l = clamp(lib_luma(c) + (ign(cell) - 0.5) * DITHER, 0.0, 1.0);
    let i = floor(l * 3.999);
    var pal = P0;
    if (i > 0.5) { pal = P1; }
    if (i > 1.5) { pal = P2; }
    if (i > 2.5) { pal = P3; }
    return mix(color, pal, k);
}
