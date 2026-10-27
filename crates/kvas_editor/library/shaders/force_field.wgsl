// Energy shield: glowing rim, travelling bands and a pulse. Use with a colour alpha below 1.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let fres = pow(1.0 - max(dot(n, v), 0.0), 3.0);
    let t = globals.time.x;
    let band = smoothstep(0.7, 1.0, sin(in.world_pos.y * 3.0 - t * 2.0) * 0.5 + 0.5);
    let hex = fbm2(in.world_pos.xz * 4.0 + in.world_pos.y * 4.0 + t * 0.2, 2);
    let pulse = 0.75 + 0.25 * sin(t * 3.0);
    let a = clamp((fres * 1.2 + band * 0.35 + hex * 0.12) * pulse, 0.0, 1.0);
    return vec4<f32>(to_display_at(object.color.rgb * (1.0 + 2.0 * fres + band), in.clip), a);
}
