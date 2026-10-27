// Wobbling, doubled vision. params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = select(params.x, 1.0, params.x <= 0.0);
    let w = vec2<f32>(sin(time * 1.7 + uv.y * 6.0), cos(time * 1.3 + uv.x * 5.0)) * 0.012 * k;
    let a = scene_color(uv + w);
    let b = scene_color(uv - w * 1.5);
    return mix(a, b, 0.5);
}
