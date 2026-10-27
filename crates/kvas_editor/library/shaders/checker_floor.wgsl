// Two-colour checker on the world XZ plane, lit normally. user 1 = square size (default 1).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let size = select(object.user0.x, 1.0, object.user0.x <= 0.0);
    let c = floor(in.world_pos.xz / size);
    let check = abs((c.x + c.y) % 2.0);
    let base = mix(object.color.rgb, object.color.rgb * 0.35, check);
    let col = lighting(in.world_pos, n, base, 0.8);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
