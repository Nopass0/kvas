// Soft dreamy bloom. params.x = glow strength (default 0.6), params.y = radius in pixels (default 4).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = select(params.x, 0.6, params.x <= 0.0);
    let r = select(params.y, 4.0, params.y <= 0.0);
    let px = pixel_size() * r;
    var sum = vec3<f32>(0.0);
    for (var i = 0; i < 8; i = i + 1) {
        let a = f32(i) * 0.785398;
        sum += scene_color(uv + vec2<f32>(cos(a), sin(a)) * px);
        sum += scene_color(uv + vec2<f32>(cos(a), sin(a)) * px * 2.0);
    }
    let blur = sum / 16.0;
    return color + max(blur - vec3<f32>(0.5), vec3<f32>(0.0)) * k * 2.0 + blur * k * 0.25;
}
