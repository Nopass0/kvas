// Draws dark outlines where the picture changes sharply. params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = select(params.x, 1.0, params.x <= 0.0);
    let px = pixel_size();
    let a = lib_luma(scene_color(uv + vec2<f32>(px.x, 0.0)));
    let b = lib_luma(scene_color(uv - vec2<f32>(px.x, 0.0)));
    let c = lib_luma(scene_color(uv + vec2<f32>(0.0, px.y)));
    let d = lib_luma(scene_color(uv - vec2<f32>(0.0, px.y)));
    let edge = clamp((abs(a - b) + abs(c - d)) * 4.0 * k, 0.0, 1.0);
    return color * (1.0 - edge);
}
