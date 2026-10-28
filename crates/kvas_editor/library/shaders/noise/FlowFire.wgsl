// Animated flowing noise in fire colours (plasma, lava lamp, magic).
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const A: vec3<f32> = vec3<f32>(0.3, 0.0, 0.0);
const B: vec3<f32> = vec3<f32>(1.0, 0.85, 0.2);
const SCALE: f32 = 1.5;
const SPEED: f32 = 0.4;
const GLOW: f32 = 1.3;
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let p = in.world_pos * SCALE + vec3<f32>(0.0, t * SPEED, t * SPEED * 0.4);
    let k = fbm3(p + fbm3(p * 1.7, 3) * 1.5, 4);
    let col = mix(A, B, smoothstep(0.2, 0.8, k)) * GLOW;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
