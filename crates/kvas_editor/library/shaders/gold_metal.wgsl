// Polished metal (gold by default, change the object colour for copper, steel...).
// user 1 = roughness (default 0.2).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let rough = select(object.user0.x, 0.2, object.user0.x <= 0.0);
    let base = object.color.rgb * vec3<f32>(1.0, 0.78, 0.35);
    var col = lighting_pbr(in.world_pos, n, base, rough, 1.0, 1.0);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    col += env_color(reflect(-v, n)) * base * (1.0 - rough) * 0.6;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
