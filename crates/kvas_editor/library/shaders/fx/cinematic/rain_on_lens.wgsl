// Rain on the lens: drops slide down the screen, bend the picture behind them and leave wet trails.
// params.x = amount (0..1), params.y = drop size (default 1), params.z = speed (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/noise"
fn drop_layer(uv: vec2<f32>, t: f32, scale: f32, seed: f32) -> vec3<f32> {
    let aspect = vec2<f32>(screen_aspect(), 1.0);
    var p = uv * aspect * scale;
    let col = floor(p.x);
    let speed = 0.15 + 0.35 * lib_hash21(vec2<f32>(col, seed));
    p.y += t * speed + lib_hash21(vec2<f32>(col, seed + 1.0)) * 5.0;
    let cell = floor(p);
    let f = fract(p) - vec2<f32>(0.5);
    let r = lib_hash21(cell + vec2<f32>(seed, 3.1));
    if (r < 0.55) { return vec3<f32>(0.0); }
    let wobble = sin(p.y * 6.0 + col) * 0.12;
    let g = f - vec2<f32>(wobble, 0.0);
    let d = length(g * vec2<f32>(1.0, 1.4));
    let drop = smoothstep(0.28, 0.16, d);
    let trail = smoothstep(0.08, 0.0, abs(g.x)) * smoothstep(-0.5, 0.2, f.y) * 0.35;
    return vec3<f32>(g * drop, max(drop, trail));
}
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let amount = clamp(select(params.x, 1.0, params.x == 0.0), 0.0, 1.0);
    let size = select(params.y, 1.0, params.y == 0.0);
    let speed = select(params.z, 1.0, params.z == 0.0);
    let a = drop_layer(uv, time * speed, 9.0 / size, 1.0);
    let b = drop_layer(uv, time * speed * 0.8, 16.0 / size, 7.0);
    let offset = (a.xy * 0.06 + b.xy * 0.035) * amount;
    let wet = max(a.z, b.z) * amount;
    // Behind a drop the picture is magnified and bent.
    let bent = scene_color(uv - offset);
    let blur = (scene_color(uv + vec2<f32>(0.004, 0.0)) + scene_color(uv - vec2<f32>(0.004, 0.0)) + scene_color(uv + vec2<f32>(0.0, 0.004)) + scene_color(uv - vec2<f32>(0.0, 0.004))) * 0.25;
    let fog = mix(color, blur * 1.05, 0.25 * amount);
    return mix(fog, bent * 1.12, wet);
}
