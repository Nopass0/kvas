// Hurt: red blotches creep in from the edges, the picture loses colour and blurs.
// params.x = damage (0..1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/noise"
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let dmg = clamp(params.x, 0.0, 1.0);
    let d = uv - vec2<f32>(0.5);
    let r = length(d * vec2<f32>(screen_aspect(), 1.0));
    let n = lib_fbm(uv * 4.0 + vec2<f32>(time * 0.05));
    let blot = smoothstep(0.95 - dmg * 0.6, 1.05 - dmg * 0.5, r + n * 0.35);
    let blur = (scene_color(uv + vec2<f32>(0.005, 0.0)) + scene_color(uv - vec2<f32>(0.005, 0.0)) + scene_color(uv + vec2<f32>(0.0, 0.005)) + scene_color(uv - vec2<f32>(0.0, 0.005))) * 0.25;
    var c = mix(color, blur, dmg * 0.6);
    c = mix(c, vec3<f32>(lib_luma(c)), dmg * 0.5);
    return mix(c, vec3<f32>(0.35, 0.0, 0.0) * (0.6 + 0.4 * n), blot * 0.85);
}
