// Snow settles on everything facing up. user 1 = amount 0..1 (animate it for a snowfall),
// user 2 = how sharply the snow edge follows the surface normal.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let tex = textureSample(albedo_tex, albedo_smp, in.uv * object.params.z);
    var base = object.color.rgb * tex.rgb;
    let amount = clamp(object.user0.x, 0.0, 1.0);
    let sharp = select(object.user0.y, 4.0, object.user0.y <= 0.0);
    let breakup = fbm2(in.world_pos.xz * 2.5, 3) * 0.35;
    let cover = smoothstep(1.0 - amount - 0.1, 1.0 - amount + 0.1, clamp(n.y, 0.0, 1.0) + breakup - 0.15) * clamp(pow(max(n.y, 0.0), 1.0 / sharp) * 1.2, 0.0, 1.0);
    base = mix(base, vec3<f32>(0.92, 0.95, 1.0), cover);
    let lit = lighting(in.world_pos, n, base, mix(object.params.x, 0.5, cover));
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), 1.0);
}
