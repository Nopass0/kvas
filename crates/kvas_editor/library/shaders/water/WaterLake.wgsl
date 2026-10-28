// Realistic animated lake water: moving wave normals, depth tint, Fresnel sky reflection and a sun glint.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const DEEP: vec3<f32> = vec3<f32>(0.02, 0.18, 0.25);
const SHALLOW: vec3<f32> = vec3<f32>(0.15, 0.55, 0.55);
const AMP: f32 = 0.02;
const SPEED: f32 = 0.7;
const SCALE: f32 = 1.0;
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let p = in.world_pos.xz * SCALE;
    let w1 = vec2<f32>(sin(p.x * 1.3 + t * SPEED) + sin(p.y * 0.9 - t * SPEED * 0.8), cos(p.y * 1.1 + t * SPEED * 0.9) + cos(p.x * 0.7 - t * SPEED));
    let w2 = vec2<f32>(sin(p.x * 3.1 - t * SPEED * 1.7), cos(p.y * 2.7 + t * SPEED * 1.4)) * 0.4;
    let nn = normalize(n + vec3<f32>(w1.x + w2.x, 0.0, w1.y + w2.y) * AMP * 8.0);
    let f = pow(1.0 - max(dot(nn, v), 0.0), 4.0);
    let l = normalize(globals.sun_dir.xyz);
    let spec = pow(max(dot(reflect(-l, nn), v), 0.0), 120.0);
    let body = mix(SHALLOW, DEEP, clamp(length(in.world_pos - globals.camera_pos.xyz) / 40.0, 0.0, 1.0));
    let col = mix(body, env_color(reflect(-v, nn)), f * 0.8) + globals.sun_color.rgb * spec;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), clamp(0.55 + f * 0.4, 0.0, 1.0) * object.color.a);
}
