// Clear glass with a coloured tint, strong Fresnel reflection of the sky and sharp sun glints.
// Use with a colour alpha around 0.15-0.35.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let fres = 0.04 + 0.96 * pow(1.0 - max(dot(n, v), 0.0), 5.0);
    let r = reflect(-v, n);
    let reflection = env_color(r);
    let spec = pow(max(dot(r, normalize(globals.sun_dir.xyz)), 0.0), 180.0) * globals.sun_color.rgb * 4.0;
    let col = mix(object.color.rgb * 0.6, reflection, fres) + spec;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), clamp(object.color.a + fres * 0.6, 0.0, 1.0));
}
