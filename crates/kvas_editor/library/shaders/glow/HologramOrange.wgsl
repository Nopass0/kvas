// A orange hologram with scan lines and flicker.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const COLOR: vec3<f32> = vec3<f32>(1.0, 0.5, 0.05);
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let scan = 0.55 + 0.45 * sin(in.world_pos.y * 60.0 - t * 4.0);
    let flicker = 0.9 + 0.1 * sin(t * 40.0);
    let f = pow(1.0 - max(dot(n, v), 0.0), 2.0);
    let col = COLOR * (0.5 + f * 1.5) * scan * flicker;
    return vec4<f32>(to_display_at(col, in.clip), (0.35 + 0.65 * f) * object.color.a);
}
