// A light 4-tap blur.
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let o = pixel_size() * 1.5;
    let b = (scene_color(uv + vec2<f32>(o.x, 0.0)) + scene_color(uv - vec2<f32>(o.x, 0.0)) + scene_color(uv + vec2<f32>(0.0, o.y)) + scene_color(uv - vec2<f32>(0.0, o.y)) + color) * 0.2;
    return mix(color, b, k);
}
