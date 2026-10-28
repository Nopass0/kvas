// A see-through red ghost: transparent in the middle, glowing at the edges.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const COLOR: vec3<f32> = vec3<f32>(1.0, 0.12, 0.1);
const POWER: f32 = 2.0;
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let f = pow(1.0 - max(dot(n, v), 0.0), POWER);
    let col = COLOR * (0.4 + 1.6 * f);
    return vec4<f32>(to_display_at(col, in.clip), clamp(f * 1.2 + 0.1, 0.0, 1.0) * object.color.a);
}
