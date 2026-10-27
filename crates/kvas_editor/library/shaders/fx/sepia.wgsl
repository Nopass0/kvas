// Old photograph: warm brown tones. params.x = strength (0..1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let l = lib_luma(color);
    let sepia = vec3<f32>(l * 1.2, l * 1.0, l * 0.8);
    return mix(color, sepia, k) * lib_vignette(uv, 0.25 * k);
}
