// A ocean water surface whose vertices move with the waves (use a subdivided plane).
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const DEEP: vec3<f32> = vec3<f32>(0.0, 0.12, 0.3);
const SHALLOW: vec3<f32> = vec3<f32>(0.1, 0.5, 0.65);
const AMP: f32 = 0.16;
const SPEED: f32 = 1.0;
@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var o = standard_vertex(v);
    o.world_pos.y += (sin(o.world_pos.x * 1.2 + globals.time.x * SPEED) + cos(o.world_pos.z * 0.9 + globals.time.x * SPEED * 0.8)) * AMP;
    o.clip = globals.view_proj * vec4<f32>(o.world_pos, 1.0);
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let f = pow(1.0 - max(dot(n, v), 0.0), 3.0);
    let col = mix(mix(SHALLOW, DEEP, 0.4), env_color(reflect(-v, n)), f);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 0.8 * object.color.a);
}
