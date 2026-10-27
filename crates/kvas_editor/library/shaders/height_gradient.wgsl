// Colours the object by world height: user 1 = height where the gradient starts, user 2 = where it ends.
// The object's colour is the bottom colour; the emissive tint is not used. Top colour is a warm sky tone.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let lo = object.user0.x;
    let hi = select(object.user0.y, lo + 10.0, object.user0.y <= lo);
    let t = clamp((in.world_pos.y - lo) / (hi - lo), 0.0, 1.0);
    let top = vec3<f32>(0.95, 0.75, 0.45);
    let base = mix(object.color.rgb, top, t * t);
    let lit = lighting(in.world_pos, normalize(in.normal), base, object.params.x);
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), object.color.a);
}
