// Flowing lava: dark crust with glowing cracks. user 1 = scale (default 2), user 2 = flow speed (default 0.15).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let scale = select(object.user0.x, 2.0, object.user0.x <= 0.0);
    let flow = select(object.user0.y, 0.15, object.user0.y <= 0.0);
    let t = globals.time.x * flow;
    let p = in.world_pos.xz * scale;
    let warp = vec2<f32>(fbm2(p + vec2<f32>(t, 0.0), 4), fbm2(p + vec2<f32>(5.2, t), 4));
    let heat = fbm2(p + warp * 2.0 + vec2<f32>(0.0, t), 5);
    let cracks = smoothstep(0.35, 0.65, heat);
    let crust = vec3<f32>(0.08, 0.05, 0.04);
    let glow = mix(vec3<f32>(1.0, 0.25, 0.02), vec3<f32>(1.0, 0.85, 0.3), cracks * cracks) * 3.0;
    let col = mix(crust, glow, cracks);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
