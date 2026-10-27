// Green night-vision goggles with noise. params.x = brightness boost (default 2).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let boost = select(params.x, 2.0, params.x <= 0.0);
    let n = lib_hash21(uv * 900.0 + time) * 0.15;
    let l = lib_luma(color) * boost + n;
    return vec3<f32>(0.1, 1.0, 0.25) * l * lib_vignette(uv, 0.7);
}
