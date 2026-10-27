// Thermal camera: warm things are bright. params.x = strength (0..1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let heat = clamp(lib_luma(color) * 1.5, 0.0, 1.0);
    let ramp = vec3<f32>(smoothstep(0.4, 0.8, heat), smoothstep(0.1, 0.5, heat) * (1.0 - smoothstep(0.8, 1.0, heat) * 0.5), smoothstep(0.0, 0.3, heat) * (1.0 - smoothstep(0.3, 0.7, heat)));
    return mix(color, ramp, clamp(params.x, 0.0, 1.0));
}
