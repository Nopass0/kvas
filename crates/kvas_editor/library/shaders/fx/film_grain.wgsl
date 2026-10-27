// Fine film grain. params.x = amount (default 0.06).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let amount = select(params.x, 0.06, params.x <= 0.0);
    let g = lib_hash21(uv * 1500.0 + vec2<f32>(time * 37.0, time * 11.0)) - 0.5;
    return max(color + vec3<f32>(g * amount), vec3<f32>(0.0));
}
