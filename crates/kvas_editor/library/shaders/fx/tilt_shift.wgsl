// Miniature look: blur above and below a sharp band. params.x = blur size (default 3).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
#include "builtin:lib/color"
#include "builtin:lib/noise"
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let size = select(params.x, 3.0, params.x <= 0.0);
    let amount = smoothstep(0.15, 0.45, abs(uv.y - 0.5));
    let px = pixel_size() * size * amount;
    var sum = color;
    sum += scene_color(uv + vec2<f32>(px.x, 0.0)) + scene_color(uv - vec2<f32>(px.x, 0.0));
    sum += scene_color(uv + vec2<f32>(0.0, px.y)) + scene_color(uv - vec2<f32>(0.0, px.y));
    sum += scene_color(uv + px) + scene_color(uv - px);
    sum += scene_color(uv + vec2<f32>(px.x, -px.y)) + scene_color(uv + vec2<f32>(-px.x, px.y));
    return sum / 9.0;
}
