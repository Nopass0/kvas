// A flickering blue flame colour, hotter towards the middle.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const COLOR: vec3<f32> = vec3<f32>(0.15, 0.4, 1.0);
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let p = in.world_pos * 2.0 + vec3<f32>(0.0, -t * 1.6, 0.0);
    let k = fbm3(p, 4);
    let heat = clamp(k * 1.6 - (in.world_pos.y - object.model[3].y) * 0.3, 0.0, 1.0);
    let col = mix(COLOR * 0.4, vec3<f32>(1.0, 0.95, 0.7), heat * heat) * (1.0 + heat * 2.0);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
