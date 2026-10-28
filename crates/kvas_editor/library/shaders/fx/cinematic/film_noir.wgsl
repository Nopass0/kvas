// Film noir: deep black and white with a hard curve, grain, a soft halo on lights and a vignette.
// params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(select(params.x, 1.0, params.x == 0.0), 0.0, 1.0);
    var y = lib_luma(color);
    let halo = lib_luma(scene_color(uv + vec2<f32>(0.004, 0.0)) + scene_color(uv - vec2<f32>(0.004, 0.0)) + scene_color(uv + vec2<f32>(0.0, 0.006))) / 3.0;
    y = max(y, halo * 0.9);
    y = smoothstep(0.03, 0.9, y);
    y = y * y * (3.0 - 2.0 * y);
    let grain = (fract(sin(dot(uv * 1000.0 + vec2<f32>(time), vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5) * 0.08;
    let d = uv - vec2<f32>(0.5);
    let vig = 1.0 - dot(d, d) * 1.6;
    return mix(color, vec3<f32>((y + grain) * vig), k);
}
