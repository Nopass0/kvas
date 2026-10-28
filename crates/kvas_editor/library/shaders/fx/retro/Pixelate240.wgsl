// Blocky pixels, 240 columns.
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let p = floor(uv * 240.0) / 240.0;
    return mix(color, scene_color(p), k);
}
