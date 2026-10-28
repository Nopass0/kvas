// A violet layer (snow, moss, dust, paint) settles on upward-facing surfaces. Cover amount via the constants.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const COVER: vec3<f32> = vec3<f32>(0.5, 0.2, 1.0);
const THRESHOLD: f32 = 0.35;
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
    let up = smoothstep(THRESHOLD, THRESHOLD + 0.2, n.y + (fbm3(in.world_pos * 4.0, 3) - 0.5) * 0.4);
    let col = mix(lit, COVER * (0.55 + 0.45 * max(dot(n, l), 0.0)), up);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
