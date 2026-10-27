// Letterbox bars like a film. params.x = bar height as a fraction of the screen (default 0.1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let bar = select(params.x, 0.1, params.x <= 0.0);
    let inside = step(bar, uv.y) * step(uv.y, 1.0 - bar);
    return color * inside;
}
