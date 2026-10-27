// Burns the object away. user 1 = progress 0..1 (animate it from a script or the Inspector),
// user 2 = glowing edge width. Colour of the edge is the emissive tint.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let noise = fbm2(in.world_pos.xz * 1.7 + in.world_pos.y * 0.9, 4);
    let edge = max(object.user0.y, 0.02);
    let t = object.user0.x * (1.0 + edge) - edge;
    if (noise < t) { discard; }
    let glow = 1.0 - smoothstep(0.0, edge, noise - t);
    let lit = lighting(in.world_pos, n, object.color.rgb, object.params.x);
    let col = lit + vec3<f32>(1.6, 0.55, 0.12) * glow * 3.0;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
