// Crisper picture. params.x = amount (default 0.5).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = select(params.x, 0.5, params.x <= 0.0);
    let px = pixel_size();
    let blur = (scene_color(uv + vec2<f32>(px.x, 0.0)) + scene_color(uv - vec2<f32>(px.x, 0.0)) + scene_color(uv + vec2<f32>(0.0, px.y)) + scene_color(uv - vec2<f32>(0.0, px.y))) * 0.25;
    return max(color + (color - blur) * k, vec3<f32>(0.0));
}
