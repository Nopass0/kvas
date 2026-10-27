// Soft outline glow that follows the silhouette. Object colour = base, user 1 = rim width (default 2).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let width = select(object.user0.x, 2.0, object.user0.x <= 0.0);
    let rim = pow(1.0 - max(dot(n, v), 0.0), width);
    var col = lighting(in.world_pos, n, object.color.rgb, 0.6);
    col += vec3<f32>(0.4, 0.7, 1.0) * rim * 1.5;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
