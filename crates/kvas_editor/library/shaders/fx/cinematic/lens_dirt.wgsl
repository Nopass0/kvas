// Dirty lens: bright lights bloom through smudges and dust spots on the glass.
// params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/noise"
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = select(params.x, 1.0, params.x == 0.0);
    var bright = vec3<f32>(0.0);
    for (var i = 0; i < 12; i = i + 1) {
        let a = f32(i) * 0.5236;
        let o = vec2<f32>(cos(a), sin(a)) * 0.03;
        let c = scene_color(uv + o);
        bright += c * smoothstep(0.7, 1.4, lib_luma(c));
    }
    bright /= 12.0;
    let smudge = lib_fbm(uv * vec2<f32>(3.0 * screen_aspect(), 3.0)) * 0.8 + 0.2 * step(0.97, lib_hash21(floor(uv * 60.0)));
    return color + bright * smudge * 2.0 * k;
}
