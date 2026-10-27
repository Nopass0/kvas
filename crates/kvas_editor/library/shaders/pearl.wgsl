// Iridescent pearl: colour shifts with the viewing angle.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let angle = 1.0 - max(dot(n, v), 0.0);
    let hue = vec3<f32>(0.5) + 0.5 * cos(6.2831853 * (vec3<f32>(angle) * 1.2 + vec3<f32>(0.0, 0.33, 0.67)));
    let base = mix(object.color.rgb, hue, 0.55);
    var col = lighting_pbr(in.world_pos, n, base, 0.25, 0.3, 1.0);
    col += env_color(reflect(-v, n)) * (0.2 + angle * 0.5);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
