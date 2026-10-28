// A 3D checkerboard in fire colours.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const A: vec3<f32> = vec3<f32>(0.3, 0.0, 0.0);
const B: vec3<f32> = vec3<f32>(1.0, 0.85, 0.2);
const SCALE: f32 = 2.0;
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let l = normalize(globals.sun_dir.xyz);
    let shadow = shadow_factor(in.world_pos, n);
    let lit = object.color.rgb * (globals.ambient.rgb * 0.8 + globals.sun_color.rgb * max(dot(n, l), 0.0) * shadow);
    let c = floor(in.world_pos * SCALE);
    let s = abs(fract((c.x + c.y + c.z) * 0.5) * 2.0);
    let col = mix(A, B, step(0.5, s)) * (0.5 + 0.5 * lit);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
