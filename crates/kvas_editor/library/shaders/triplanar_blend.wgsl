// Applies the albedo texture in world space on all three axes (no UV stretching on rocks, terrain).
// Texture repeats per metre = the object's "tiling" value.
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let w0 = pow(abs(n), vec3<f32>(6.0));
    let w = w0 / (w0.x + w0.y + w0.z);
    let s = max(object.params.z, 0.01);
    let x = textureSample(albedo_tex, albedo_smp, in.world_pos.zy * s).rgb;
    let y = textureSample(albedo_tex, albedo_smp, in.world_pos.xz * s).rgb;
    let z = textureSample(albedo_tex, albedo_smp, in.world_pos.xy * s).rgb;
    let base = (x * w.x + y * w.y + z * w.z) * object.color.rgb;
    let lit = lighting(in.world_pos, n, base, object.params.x);
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), object.color.a);
}
