// Frosted ice: bluish, glossy, with a cloudy interior. Works with a colour alpha around 0.6-0.8.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let fres = 0.05 + 0.95 * pow(1.0 - max(dot(n, v), 0.0), 4.0);
    let frost = fbm3(in.world_pos * 3.0, 4);
    let base = mix(vec3<f32>(0.55, 0.78, 0.95), vec3<f32>(0.9, 0.97, 1.0), frost) * object.color.rgb;
    var col = lighting(in.world_pos, n, base, 0.15);
    col = mix(col, env_color(reflect(-v, n)), fres);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), clamp(object.color.a + fres * 0.3, 0.0, 1.0));
}
