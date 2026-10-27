// Glowing neon tube: bright core colour that pulses. user 1 = pulse speed (default 2), user 2 = glow strength.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let speed = select(object.user0.x, 2.0, object.user0.x <= 0.0);
    let power = select(object.user0.y, 3.0, object.user0.y <= 0.0);
    let pulse = 0.75 + 0.25 * sin(globals.time.x * speed);
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let core = pow(max(dot(n, v), 0.0), 0.6);
    let col = object.color.rgb * power * pulse * (0.4 + core);
    return vec4<f32>(to_display_at(col, in.clip), 1.0);
}
