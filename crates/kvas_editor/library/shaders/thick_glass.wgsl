// Thick tinted glass or crystal: the colour deepens with how much glass the view passes through
// (from the scene depth), edges catch the light, and objects behind are seen softened.
// Use with a colour alpha below 1.
//   object.color  glass tint      object.user0.x  thickness at which the tint is full (m, default 0.8)
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let full = select(object.user0.x, 0.8, object.user0.x == 0.0);
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let fresnel = 0.04 + 0.96 * pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 5.0);
    let thickness = min(scene_depth_behind(in.clip, in.world_pos), full * 4.0);
    let absorb = clamp(thickness / full, 0.0, 1.0);
    var r = reflect(-v, n);
    let refl = env_color(r);
    let l = normalize(globals.sun_dir.xyz);
    let spec = pow(max(dot(r, l), 0.0), 400.0) * 4.0;
    let col = mix(object.color.rgb * (0.3 + 0.7 * absorb), refl, fresnel) + globals.sun_color.xyz * spec;
    let alpha = clamp(object.color.a * 0.3 + absorb * 0.5 + fresnel, 0.0, 1.0);
    return vec4<f32>(to_display_at(col, in.clip), alpha);
}
