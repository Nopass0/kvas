// A magenta energy shield: moving hex-like bands and a bright rim.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const COLOR: vec3<f32> = vec3<f32>(1.0, 0.1, 0.8);
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let f = pow(1.0 - max(dot(n, v), 0.0), 2.0);
    let bands = 0.5 + 0.5 * sin(in.world_pos.y * 12.0 + in.world_pos.x * 6.0 - t * 2.0);
    let col = COLOR * (0.3 + f * 2.5 + bands * 0.8);
    return vec4<f32>(to_display_at(col, in.clip), clamp(0.15 + f * 0.8 + bands * 0.2, 0.0, 1.0) * object.color.a);
}
