// Underwater: wavy distortion, blue-green tint, depth falloff to the edges. params.x = strength.
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = max(params.x, 0.0);
    let off = vec2<f32>(sin(uv.y * 22.0 + time * 2.0), cos(uv.x * 18.0 + time * 1.6)) * 0.004 * k;
    var c = scene_color(uv + off);
    c = mix(c, c * vec3<f32>(0.45, 0.85, 1.0), clamp(k, 0.0, 1.0) * 0.85);
    let caustic = lib_fbm(uv * 10.0 + time * 0.3);
    c += vec3<f32>(0.04, 0.08, 0.1) * caustic * k;
    let d = length(uv - vec2<f32>(0.5));
    return c * (1.0 - 0.5 * smoothstep(0.3, 0.9, d) * k);
}
