// Black and white with a contrast curve. params.x = amount of desaturation (1 = fully grey).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let grey = vec3<f32>(lib_luma(color));
    return lib_contrast(mix(color, grey, clamp(params.x, 0.0, 1.0)), 1.15);
}
