// The shape swells and shrinks along its normals.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const AMP: f32 = 0.3;
const SPEED: f32 = 3.0;
const FREQ: f32 = 2.0;
@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var o = standard_vertex(v);
    o.world_pos += o.normal * sin(globals.time.x * SPEED) * AMP;
    o.clip = globals.view_proj * vec4<f32>(o.world_pos, 1.0);
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let l = normalize(globals.sun_dir.xyz);
    let shadow = shadow_factor(in.world_pos, n);
    let lit = object.color.rgb * (globals.ambient.rgb * 0.8 + globals.sun_color.rgb * max(dot(n, l), 0.0) * shadow);
    let col = lit;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
