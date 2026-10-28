// Fear: a heartbeat pulse darkens and reddens the edges, the picture swims and splits colours.
// params.x = fear (0..1), params.y = heart rate in beats per minute (default 110).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let fear = clamp(params.x, 0.0, 1.0);
    let bpm = select(params.y, 110.0, params.y == 0.0);
    let beat = fract(time * bpm / 60.0);
    let pulse = exp(-beat * 9.0) + 0.6 * exp(-abs(beat - 0.22) * 14.0);
    let d = uv - vec2<f32>(0.5);
    let r = length(d * vec2<f32>(screen_aspect(), 1.0));
    let swim = vec2<f32>(sin(uv.y * 9.0 + time * 1.7), cos(uv.x * 7.0 + time * 1.3)) * 0.003 * fear;
    let split = d * 0.012 * fear * (0.5 + pulse);
    var c = vec3<f32>(scene_color(uv + swim + split).r, scene_color(uv + swim).g, scene_color(uv + swim - split).b);
    let edge = smoothstep(0.35, 0.95, r) * fear * (0.6 + 0.4 * pulse);
    c = mix(c, c * vec3<f32>(0.5, 0.05, 0.05), edge);
    c = mix(c, vec3<f32>(lib_luma(c)), fear * 0.35);
    return c;
}
