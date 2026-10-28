// Night street grade: sodium-lamp orange highlights, cold blue-green shadows, lifted blacks.
// params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(select(params.x, 1.0, params.x == 0.0), 0.0, 1.0);
    let y = lib_luma(color);
    let shadows = vec3<f32>(0.06, 0.12, 0.16);
    let highs = vec3<f32>(1.15, 0.78, 0.42);
    var c = mix(shadows + color * 0.8, color * highs, smoothstep(0.05, 0.6, y));
    c = lib_saturation(c, 0.85);
    return mix(color, c, k);
}
