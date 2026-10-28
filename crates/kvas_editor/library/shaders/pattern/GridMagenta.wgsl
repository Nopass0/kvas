// A glowing magenta world grid on a dark surface.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const COLOR: vec3<f32> = vec3<f32>(1.0, 0.1, 0.8);
const SCALE: f32 = 1.0;
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let g = abs(fract(in.world_pos * SCALE - 0.5) - 0.5) / fwidth(in.world_pos * SCALE);
    let line = 1.0 - min(min(g.x, g.y), g.z);
    let col = vec3<f32>(0.02) + COLOR * clamp(line, 0.0, 1.0) * 2.0;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
