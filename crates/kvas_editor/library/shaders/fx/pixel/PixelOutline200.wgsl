// Pixel art at 200 columns with dark outlines where the picture changes sharply.
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"
const CELLS: f32 = 200.0;

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let grid = vec2<f32>(CELLS, CELLS / screen_aspect());
    let cell = floor(uv * grid);
    let q = (cell + 0.5) / grid;
    let c = scene_color(q);
    let o = 1.0 / grid;
    let e = abs(lib_luma(scene_color(q + vec2<f32>(o.x, 0.0))) - lib_luma(scene_color(q - vec2<f32>(o.x, 0.0)))) + abs(lib_luma(scene_color(q + vec2<f32>(0.0, o.y))) - lib_luma(scene_color(q - vec2<f32>(0.0, o.y))));
    return mix(color, mix(floor(c * 5.0 + 0.5) / 5.0, vec3<f32>(0.02), step(0.22, e)), k);
}
