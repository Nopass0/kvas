// Gradient by the distance from the object centre: ice.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const A: vec3<f32> = vec3<f32>(0.5, 0.75, 1.0);
const B: vec3<f32> = vec3<f32>(0.95, 1.0, 1.0);
const RADIUS: f32 = 1.5;
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
    let d = clamp(length(in.world_pos - object.model[3].xyz) / RADIUS, 0.0, 1.0);
    let col = mix(A, B, d) * (0.45 + 0.55 * lit);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
