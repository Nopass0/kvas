// Heavy VHS tape: tracking wobble, colour bleed, head-switch noise at the bottom, tape dropouts.
// params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/noise"
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = select(params.x, 1.0, params.x == 0.0);
    let line = floor(uv.y * 240.0);
    let wobble = (lib_noise(vec2<f32>(line * 0.05, time * 2.0)) - 0.5) * 0.004 * k;
    let tear = step(0.985, lib_hash21(vec2<f32>(floor(time * 8.0), 1.0))) * smoothstep(0.02, 0.0, abs(uv.y - fract(time * 0.37))) * 0.03 * k;
    let head = smoothstep(0.94, 1.0, uv.y) * (lib_hash21(vec2<f32>(line, floor(time * 30.0))) - 0.5) * 0.04 * k;
    let u = uv + vec2<f32>(wobble + tear + head, 0.0);
    // Chroma is blurred sideways and shifted, luma stays sharp.
    let y = lib_luma(scene_color(u));
    let c = (scene_color(u + vec2<f32>(0.006, 0.0)) + scene_color(u + vec2<f32>(0.012, 0.0)) + scene_color(u)) / 3.0;
    let chroma = c - vec3<f32>(lib_luma(c));
    var col = vec3<f32>(y) + chroma * 1.15;
    // Dropouts: thin white streaks.
    let drop = step(0.9975, lib_hash21(vec2<f32>(line, floor(time * 24.0)))) * k;
    col = mix(col, vec3<f32>(0.9), drop * smoothstep(0.0, 0.3, lib_hash21(vec2<f32>(uv.x * 40.0, line))));
    col *= 0.94 + 0.06 * sin(uv.y * 900.0);
    col += vec3<f32>((lib_hash21(uv * 400.0 + vec2<f32>(time)) - 0.5) * 0.05 * k);
    return mix(color, col, clamp(k, 0.0, 1.0));
}
