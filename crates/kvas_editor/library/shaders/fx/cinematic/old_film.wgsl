// Old film reel: sepia, flicker, scratches, dust, gate weave and burnt corners.
// params.x = strength (default 1).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = clamp(select(params.x, 1.0, params.x == 0.0), 0.0, 1.0);
    let frame = floor(time * 18.0);
    let weave = vec2<f32>(lib_hash21(vec2<f32>(frame, 1.0)) - 0.5, lib_hash21(vec2<f32>(frame, 2.0)) - 0.5) * 0.003;
    let y = lib_luma(scene_color(uv + weave));
    var c = vec3<f32>(y) * vec3<f32>(1.07, 0.92, 0.72);
    c *= 0.9 + 0.1 * lib_hash21(vec2<f32>(frame, 3.0));
    let sx = lib_hash21(vec2<f32>(frame, 4.0));
    c = mix(c, vec3<f32>(0.95), smoothstep(0.0015, 0.0, abs(uv.x - sx)) * step(0.6, lib_hash21(vec2<f32>(frame, 5.0))));
    let dust = step(0.9993, lib_hash21(floor(uv * vec2<f32>(400.0, 300.0)) + vec2<f32>(frame)));
    c = mix(c, vec3<f32>(0.05), dust);
    let d = uv - vec2<f32>(0.5);
    c *= 1.0 - smoothstep(0.25, 0.75, dot(d, d) * 2.5);
    return mix(color, c, k);
}
