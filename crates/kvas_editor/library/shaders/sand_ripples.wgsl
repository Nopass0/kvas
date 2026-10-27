// Desert sand with wind ripples and a sun sparkle. user 1 = ripple scale (default 3).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let scale = select(object.user0.x, 3.0, object.user0.x <= 0.0);
    let p = in.world_pos.xz * scale;
    let ripple = sin(p.x * 2.0 + fbm2(p * 0.5, 3) * 4.0) * 0.5 + 0.5;
    var n = normalize(in.normal);
    n = normalize(n + vec3<f32>((ripple - 0.5) * 0.25, 0.0, 0.0));
    let base = mix(object.color.rgb * 0.85, object.color.rgb * 1.1, ripple);
    var col = lighting(in.world_pos, n, base, 0.95);
    let sparkle = step(0.985, hash21(floor(in.world_pos.xz * 60.0))) * max(dot(n, normalize(globals.sun_dir.xyz)), 0.0);
    col += globals.sun_color.rgb * sparkle * 0.6;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
