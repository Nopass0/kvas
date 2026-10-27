// Red edges when the player is hurt: set params.x from a script (0 = none, 1 = full).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let edge = 1.0 - lib_vignette(uv, 1.0);
    let pulse = 0.8 + 0.2 * sin(time * 12.0);
    return mix(color, vec3<f32>(0.9, 0.05, 0.05), clamp(edge * params.x * pulse, 0.0, 0.85));
}
