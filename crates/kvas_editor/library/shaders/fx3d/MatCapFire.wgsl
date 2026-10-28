// A fake lit sphere look (MatCap) in fire colours: shading depends only on the view angle.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const A: vec3<f32> = vec3<f32>(0.3, 0.0, 0.0);
const B: vec3<f32> = vec3<f32>(1.0, 0.85, 0.2);
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let vn = normalize((globals.view_proj * vec4<f32>(n, 0.0)).xyz);
    let k = clamp(vn.y * 0.5 + 0.5, 0.0, 1.0);
    let col = mix(A, B, k * k) * (0.6 + 0.4 * max(dot(n, v), 0.0));
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
