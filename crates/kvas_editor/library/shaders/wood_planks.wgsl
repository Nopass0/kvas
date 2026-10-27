// Procedural wooden planks along the world X axis. user 1 = plank width (default 0.4).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let width = select(object.user0.x, 0.4, object.user0.x <= 0.0);
    let uv = vec2<f32>(in.world_pos.x + in.world_pos.z * 0.0, in.world_pos.z + in.world_pos.y);
    let row = floor(uv.y / width);
    let along = uv.x + hash21(vec2<f32>(row, 3.0)) * 10.0;
    let grain = fbm2(vec2<f32>(along * 1.5, uv.y * 14.0), 4);
    let gap = smoothstep(0.0, 0.03, fract(uv.y / width)) * smoothstep(1.0, 0.97, fract(uv.y / width));
    let tint = 0.8 + 0.4 * hash21(vec2<f32>(row, 7.0));
    let base = mix(vec3<f32>(0.32, 0.18, 0.08), vec3<f32>(0.55, 0.34, 0.16), grain) * tint * object.color.rgb * 1.6;
    let col = lighting(in.world_pos, n, base * (0.3 + 0.7 * gap), 0.7);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
