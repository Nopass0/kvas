// Shows the albedo texture as big crisp 128x128 pixels (nearest sampling) with banded lighting.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const TEX: f32 = 128.0;
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
    let uv = floor(in.uv * TEX) / TEX + 0.5 / TEX;
    let tex = textureSample(albedo_tex, albedo_smp, uv).rgb;
    let col = floor(tex * lit * 5.0 + 0.5) / 5.0;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
