// Dissolves with noise and a cyan burning edge. user 1 = amount 0..1 (animates by itself when 0).
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const EDGE: vec3<f32> = vec3<f32>(0.1, 0.9, 1.0);
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
    let amount = select(object.user0.x, 0.5 + 0.5 * sin(t * 0.7), object.user0.x == 0.0);
    let k = fbm3(in.world_pos * 3.0, 4);
    if (k < amount) { discard; }
    let edge = 1.0 - smoothstep(0.0, 0.06, k - amount);
    let col = lit + EDGE * edge * 4.0;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
