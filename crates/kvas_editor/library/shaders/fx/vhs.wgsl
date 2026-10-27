// Old tape look: colour bleeding, scan lines, noise and a rolling band. params.x = amount.
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = max(params.x, 0.0);
    let wobble = (lib_noise(vec2<f32>(uv.y * 40.0, time * 3.0)) - 0.5) * 0.004 * k;
    let r = scene_color(uv + vec2<f32>(wobble + 0.002 * k, 0.0)).r;
    let g = scene_color(uv + vec2<f32>(wobble, 0.0)).g;
    let b = scene_color(uv + vec2<f32>(wobble - 0.002 * k, 0.0)).b;
    var c = vec3<f32>(r, g, b);
    c *= 0.9 + 0.1 * sin(uv.y * 900.0);
    c += (lib_hash21(uv * 800.0 + time) - 0.5) * 0.08 * k;
    let band = smoothstep(0.0, 0.03, abs(fract(uv.y - time * 0.1) - 0.5) - 0.46);
    return mix(c, c * 1.25, band * 0.3 * k) * lib_vignette(uv, 0.3);
}
