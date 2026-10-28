// Tints the picture magenta (multiply).
// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"
#include "builtin:lib/noise"
const TINT: vec3<f32> = vec3<f32>(1.0, 0.46, 0.88);
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    return mix(color, color * TINT, k);
}
