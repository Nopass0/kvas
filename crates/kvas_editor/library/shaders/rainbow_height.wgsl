// Colour cycles with world height, handy for debugging terrain. user 1 = metres per full cycle (default 10).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let period = select(object.user0.x, 10.0, object.user0.x <= 0.0);
    let h = fract(in.world_pos.y / period);
    let hue = vec3<f32>(0.5) + 0.5 * cos(6.2831853 * (vec3<f32>(h) + vec3<f32>(0.0, 0.33, 0.67)));
    let col = lighting(in.world_pos, n, hue * object.color.rgb, 0.8);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
