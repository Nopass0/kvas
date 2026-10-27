// Toon shading with a dark outline from the silhouette. user 1 = bands (default 3), user 2 = outline width (default 0.35).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let bands = max(select(object.user0.x, 3.0, object.user0.x <= 0.0), 2.0);
    let width = select(object.user0.y, 0.35, object.user0.y <= 0.0);
    let l = normalize(globals.sun_dir.xyz);
    let diffuse = floor(max(dot(n, l), 0.0) * bands + 0.5) / bands;
    let edge = smoothstep(width, width + 0.05, max(dot(n, v), 0.0));
    let col = object.color.rgb * (globals.ambient.rgb * 0.8 + globals.sun_color.rgb * diffuse) * edge;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
