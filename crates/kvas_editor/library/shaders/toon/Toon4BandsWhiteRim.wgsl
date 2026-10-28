// Cel shading with 4 light bands and a white rim.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const BANDS: f32 = 4.0;
const RIM: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let l = normalize(globals.sun_dir.xyz);
    let band = floor(max(dot(n, l), 0.0) * BANDS + 0.5) / BANDS;
    let shadow = shadow_factor(in.world_pos, n);
    let rim = smoothstep(0.55, 0.7, 1.0 - max(dot(n, v), 0.0));
    let col = object.color.rgb * (globals.ambient.rgb * 0.8 + globals.sun_color.rgb * band * shadow) + RIM * rim * 0.6;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
