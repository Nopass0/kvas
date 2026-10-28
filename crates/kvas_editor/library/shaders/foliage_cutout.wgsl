// kvas:alpha_to_coverage
// For imported plants and trees: cuts leaves out of the model's texture (alpha below 0.5 is
// discarded), lights them softly from both sides and sways them in the wind, more at the top.
//   object.user0  x wind strength (default 1), y height of the model in metres (default 4)
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var out = standard_vertex(v);
    let wind = select(object.user0.x, 1.0, object.user0.x == 0.0);
    let height = select(object.user0.y, 4.0, object.user0.y == 0.0);
    let root = object.model[3].xyz;
    let k = clamp((out.world_pos.y - root.y) / height, 0.0, 1.5);
    let t = globals.time.x;
    let sway = sin(t * 1.3 + root.x * 0.4 + root.z * 0.3) * 0.12 * k * k * wind;
    let flutter = sin(t * 7.0 + dot(out.world_pos, vec3<f32>(3.1, 2.3, 1.7))) * 0.025 * k * wind;
    out.world_pos += vec3<f32>(sway + flutter, flutter * 0.5, sway * 0.6 - flutter * 0.4);
    out.clip = globals.view_proj * vec4<f32>(out.world_pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VsOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let tex = textureSample(albedo_tex, albedo_smp, in.uv);
    // Sharpened alpha: smooth edges through alpha-to-coverage instead of a hard cut.
    let coverage = clamp((tex.a - 0.5) / max(fwidth(tex.a), 1e-4) + 0.5, 0.0, 1.0);
    if (coverage <= 0.01) { discard; }
    let albedo = tex.rgb * object.color.rgb;
    var n = normalize(in.normal);
    if (!front) { n = -n; }
    let l = normalize(globals.sun_dir.xyz);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let shadow = shadow_factor(in.world_pos, n);
    let wrap = clamp(dot(n, l) * 0.5 + 0.5, 0.0, 1.0);
    var col = albedo * globals.sun_color.xyz * wrap * mix(0.35, 1.0, shadow) + albedo * env_color(n) * 0.5;
    col += albedo * globals.sun_color.xyz * pow(max(dot(-v, l), 0.0), 4.0) * 0.6 * shadow;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), coverage);
}
