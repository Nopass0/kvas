// Splits red and blue by 4.0 pixels.
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let o = vec2<f32>(4.0, 0.0) * pixel_size();
    let c = vec3<f32>(scene_color(uv + o).r, color.g, scene_color(uv - o).b);
    return mix(color, c, k);
}
