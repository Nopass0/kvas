// PlayStation-style picture: low resolution, 15-bit colour with an ordered dither.
// params.x = vertical resolution (default 240), params.y = colour levels (default 32).
#include "shaders/kvas_camera_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
fn bayer(p: vec2<f32>) -> f32 {
    let x = i32(p.x) % 4;
    let y = i32(p.y) % 4;
    var m = array<f32, 16>(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return m[y * 4 + x] / 16.0 - 0.5;
}
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let lines = select(params.x, 240.0, params.x == 0.0);
    let levels = select(params.y, 32.0, params.y == 0.0);
    let res = vec2<f32>(lines * screen_aspect(), lines);
    let px = floor(uv * res);
    let c = scene_color((px + vec2<f32>(0.5)) / res);
    let d = bayer(px) / levels;
    let q = floor(c * levels + vec3<f32>(d * levels + 0.5)) / levels;
    return max(q, vec3<f32>(0.0));
}
