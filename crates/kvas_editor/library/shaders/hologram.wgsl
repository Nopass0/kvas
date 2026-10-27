// Blue hologram: scan lines, Fresnel glow and a flicker. Needs a colour alpha below 1 on the
// object (so it is drawn as transparent). user 1 = scan line density.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let fres = pow(1.0 - max(dot(n, v), 0.0), 2.5);
    let t = globals.time.x;
    let density = select(object.user0.x, 40.0, object.user0.x <= 0.0);
    let scan = 0.55 + 0.45 * sin(in.world_pos.y * density - t * 4.0);
    let flicker = 0.9 + 0.1 * sin(t * 37.0) * sin(t * 11.0);
    let col = object.color.rgb * (0.4 + 1.6 * fres) * scan * flicker;
    return vec4<f32>(to_display_at(col * 1.5, in.clip), clamp(0.25 + fres * 0.7, 0.0, 1.0) * scan);
}
