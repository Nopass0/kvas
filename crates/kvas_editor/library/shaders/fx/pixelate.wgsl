// Chunky pixels with a limited palette. params.x = pixel size in screen pixels, params.y = colour levels.
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let size = max(params.x, 1.0) * pixel_size();
    let snapped = (floor(uv / size) + vec2<f32>(0.5)) * size;
    let c = scene_color(snapped);
    let levels = max(params.y, 2.0);
    return floor(c * levels + vec3<f32>(0.5)) / levels;
}
