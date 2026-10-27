// Warm evening grade with a soft vignette. params.x = warmth, params.y = vignette strength.
#include "builtin:lib/color"

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    var c = lib_saturation(color, 1.0 + 0.12 * params.x);
    c *= vec3<f32>(1.0 + 0.05 * params.x, 1.0, 1.0 - 0.07 * params.x);
    c *= lib_vignette(uv, params.y);
    return c;
}
