// Lens colour fringing that grows towards the corners. params.x = strength.
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let d = uv - vec2<f32>(0.5);
    let o = d * dot(d, d) * 0.06 * params.x;
    return vec3<f32>(scene_color(uv + o).r, scene_color(uv).g, scene_color(uv - o).b);
}
