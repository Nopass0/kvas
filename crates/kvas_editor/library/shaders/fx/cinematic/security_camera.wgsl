// Security monitor: fisheye, green monochrome, scanlines, rolling bar, noise and a REC dot.
// params.x = strength (default 1), params.y = fisheye (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(select(params.x, 1.0, params.x == 0.0), 0.0, 1.0);
    let fish = select(params.y, 1.0, params.y == 0.0);
    let d = uv - vec2<f32>(0.5);
    let warped = vec2<f32>(0.5) + d * (1.0 - fish * 0.22 * dot(d, d) * 4.0);
    if (any(warped < vec2<f32>(0.0)) || any(warped > vec2<f32>(1.0))) { return mix(color, vec3<f32>(0.0), k); }
    let y = lib_luma(scene_color(warped));
    var g = vec3<f32>(0.25, 1.0, 0.45) * pow(max(y, 0.0), 0.8);
    g *= 0.85 + 0.15 * sin(warped.y * 700.0);
    g *= 1.0 - 0.15 * smoothstep(0.0, 0.05, abs(fract(warped.y - time * 0.2) - 0.5) - 0.42);
    g += vec3<f32>((fract(sin(dot(warped * 800.0 + vec2<f32>(time), vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5) * 0.12);
    let rec = smoothstep(0.012, 0.008, length((uv - vec2<f32>(0.06, 0.07)) * vec2<f32>(screen_aspect(), 1.0))) * step(0.5, fract(time));
    g = mix(g, vec3<f32>(1.0, 0.1, 0.1), rec);
    return mix(color, g * (1.0 - dot(d, d) * 1.2), k);
}
