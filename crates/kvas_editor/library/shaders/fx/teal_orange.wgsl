// Hollywood grade: teal shadows, orange highlights. params.x = strength (0..1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let l = lib_luma(color);
    let teal = vec3<f32>(0.0, 0.5, 0.6);
    let orange = vec3<f32>(1.0, 0.55, 0.2);
    let tint = mix(teal, orange, smoothstep(0.1, 0.8, l));
    return mix(color, color * (0.5 + tint), k * 0.7);
}
