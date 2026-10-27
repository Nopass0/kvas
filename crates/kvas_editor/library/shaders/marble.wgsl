// White marble with grey veins. user 1 = vein scale (default 1.5).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let scale = select(object.user0.x, 1.5, object.user0.x <= 0.0);
    let p = in.world_pos * scale;
    let veins = abs(sin(p.x * 3.0 + fbm3(p * 2.0, 5) * 6.0));
    let vein = 1.0 - smoothstep(0.0, 0.12, veins);
    let base = mix(object.color.rgb, vec3<f32>(0.25, 0.27, 0.3), vein * 0.8);
    var col = lighting_pbr(in.world_pos, n, base, 0.18, 0.0, 1.0);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
