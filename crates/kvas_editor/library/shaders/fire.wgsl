// Animated flame surface: use on a cone, sphere or quad. Hot white core, orange edges, upward flow.
// Use with an object colour alpha below 1.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let t = globals.time.x;
    let p = in.world_pos;
    let flow = fbm2(vec2<f32>(p.x * 2.0 + p.z * 1.3, p.y * 2.2 - t * 2.4), 4);
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - p);
    let edge = pow(max(dot(n, v), 0.0), 1.4);
    let heat = clamp(flow * 1.6 * edge, 0.0, 1.0);
    let col = mix(vec3<f32>(0.9, 0.2, 0.02), vec3<f32>(1.0, 0.9, 0.5), heat * heat) * (1.0 + 2.0 * heat);
    return vec4<f32>(to_display_at(col, in.clip), clamp(heat * 1.3, 0.0, 1.0));
}
