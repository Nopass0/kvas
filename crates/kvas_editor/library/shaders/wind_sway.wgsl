// Foliage that sways in the wind: the upper part of the mesh moves more. user 1 = strength,
// user 2 = speed, user 3 = size of the area the object covers (for the height weighting).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var local = v.position;
    let strength = select(object.user0.x, 0.12, object.user0.x <= 0.0);
    let speed = select(object.user0.y, 1.4, object.user0.y <= 0.0);
    let height = max(local.y, 0.0);
    let phase = object.model[3].x * 0.7 + object.model[3].z * 0.9;
    let t = globals.time.x * speed + phase;
    local.x += sin(t + local.y * 2.0) * strength * height;
    local.z += cos(t * 0.8 + local.y * 1.7) * strength * 0.6 * height;
    var out: VsOut;
    let world = object.model * vec4<f32>(local, 1.0);
    out.clip = globals.view_proj * world;
    out.world_pos = world.xyz;
    out.normal = normalize((object.normal_matrix * vec4<f32>(v.normal, 0.0)).xyz);
    out.uv = v.uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tex = textureSample(albedo_tex, albedo_smp, in.uv * object.params.z);
    if (tex.a < 0.5) { discard; }
    let base = object.color * tex;
    let lit = lighting(in.world_pos, normalize(in.normal), base.rgb, object.params.x);
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), base.a);
}
