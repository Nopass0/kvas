// Cel shading: light is cut into bands with a crisp rim. Material parameters (Inspector > user):
//   user 1 = number of bands (default 3), user 2 = rim strength
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let bands = max(object.user0.x, 2.0);
    let l = normalize(globals.sun_dir.xyz);
    let diffuse = floor(max(dot(n, l), 0.0) * bands + 0.5) / bands;
    let shadow = shadow_factor(in.world_pos, n);
    let rim = smoothstep(0.55, 0.7, 1.0 - max(dot(n, v), 0.0)) * object.user0.y;
    var col = object.color.rgb * (globals.ambient.rgb * 0.8 + globals.sun_color.rgb * diffuse * shadow) + vec3<f32>(rim);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
