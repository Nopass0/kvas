// Digital glitch: horizontal tears and colour splits. params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = select(params.x, 1.0, params.x <= 0.0);
    let slot = floor(uv.y * 24.0);
    let tick = floor(time * 6.0);
    let on = step(0.85, lib_hash21(vec2<f32>(slot, tick)));
    let shift = (lib_hash21(vec2<f32>(slot + 3.0, tick)) - 0.5) * 0.15 * on * k;
    let r = scene_color(uv + vec2<f32>(shift + 0.004 * on * k, 0.0)).r;
    let g = scene_color(uv + vec2<f32>(shift, 0.0)).g;
    let b = scene_color(uv + vec2<f32>(shift - 0.004 * on * k, 0.0)).b;
    return vec3<f32>(r, g, b);
}
