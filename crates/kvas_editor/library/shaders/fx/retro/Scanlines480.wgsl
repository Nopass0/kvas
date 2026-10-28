// CRT scan lines (480 lines).
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let s = 1.0 - 0.25 * (0.5 + 0.5 * sin(uv.y * 480.0 * 3.14159));
    return mix(color, color * s, k);
}
