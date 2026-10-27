// Glowing grid lines on a dark surface. user 1 = cell size (default 1), user 2 = line width (default 0.04).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let size = select(object.user0.x, 1.0, object.user0.x <= 0.0);
    let width = select(object.user0.y, 0.04, object.user0.y <= 0.0);
    let p = in.world_pos / size;
    let f = abs(fract(p) - vec3<f32>(0.5));
    let edge = vec3<f32>(0.5) - f;
    let line = max(max(smoothstep(width, 0.0, edge.x), smoothstep(width, 0.0, edge.y)), smoothstep(width, 0.0, edge.z));
    let pulse = 0.7 + 0.3 * sin(globals.time.x * 2.0 - length(in.world_pos.xz) * 0.5);
    let col = vec3<f32>(0.01, 0.015, 0.03) + object.color.rgb * line * 3.0 * pulse;
    return vec4<f32>(to_display_at(col, in.clip), 1.0);
}
