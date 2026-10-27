// Limited colour steps like a comic. params.x = number of steps (default 5).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let steps = max(select(params.x, 5.0, params.x <= 0.0), 2.0);
    return floor(color * steps + vec3<f32>(0.5)) / steps;
}
