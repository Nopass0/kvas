// Dream / memory: soft glow, washed colours, warm light leaking from the edge, slow breathing blur.
// params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(select(params.x, 1.0, params.x == 0.0), 0.0, 1.0);
    let r = 0.006 + 0.003 * sin(time * 0.8);
    var glow = vec3<f32>(0.0);
    for (var i = 0; i < 8; i = i + 1) {
        let a = f32(i) * 0.785;
        glow += scene_color(uv + vec2<f32>(cos(a), sin(a)) * r * vec2<f32>(1.0 / screen_aspect(), 1.0) * 3.0);
    }
    glow /= 8.0;
    var c = max(color, glow * 1.1);
    c = lib_saturation(c, 0.7) + vec3<f32>(0.04, 0.02, 0.0);
    let leak = smoothstep(0.6, 1.0, uv.x + sin(time * 0.3) * 0.1) * vec3<f32>(0.25, 0.1, 0.02);
    return mix(color, c + leak, k);
}
