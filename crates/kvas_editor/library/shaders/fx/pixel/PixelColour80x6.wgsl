// Pixel art at 80 columns keeping the scene colours: 6 levels per channel and dithering.
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"
const CELLS: f32 = 80.0;
const LEVELS: f32 = 6.0;
fn ign(p: vec2<f32>) -> f32 { return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715)))); }

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let grid = vec2<f32>(CELLS, CELLS / screen_aspect());
    let cell = floor(uv * grid);
    let c = scene_color((cell + 0.5) / grid);
    let d = (ign(cell) - 0.5) / LEVELS;
    return mix(color, floor((c + vec3<f32>(d)) * LEVELS + 0.5) / LEVELS, k);
}
