// Moss grows on upward-facing surfaces. user 1 = coverage (default 0.5), user 2 = noise scale (default 3).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let cover = select(object.user0.x, 0.5, object.user0.x <= 0.0);
    let scale = select(object.user0.y, 3.0, object.user0.y <= 0.0);
    let noise = fbm3(in.world_pos * scale, 4);
    let amount = smoothstep(1.0 - cover - 0.1, 1.0 - cover + 0.1, n.y * 0.6 + noise * 0.6);
    let moss = mix(vec3<f32>(0.1, 0.3, 0.05), vec3<f32>(0.25, 0.5, 0.1), noise);
    let base = mix(object.color.rgb, moss, amount);
    let col = lighting(in.world_pos, n, base, mix(0.8, 1.0, amount));
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
