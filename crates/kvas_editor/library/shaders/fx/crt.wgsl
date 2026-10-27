// Old monitor: scan lines, slight curvature and glow. params.x = strength (0..1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(params.x, 0.0, 1.0);
    let c = uv * 2.0 - vec2<f32>(1.0);
    let bent = c * (1.0 + dot(c, c) * 0.06 * k);
    let uv2 = bent * 0.5 + vec2<f32>(0.5);
    if (uv2.x < 0.0 || uv2.x > 1.0 || uv2.y < 0.0 || uv2.y > 1.0) {
        return vec3<f32>(0.0);
    }
    var col = scene_color(uv2);
    col *= 1.0 - k * 0.25 * (0.5 + 0.5 * sin(uv2.y * 700.0));
    col *= 1.0 + k * 0.05 * sin(time * 60.0);
    return col * lib_vignette(uv2, 0.3 * k);
}
