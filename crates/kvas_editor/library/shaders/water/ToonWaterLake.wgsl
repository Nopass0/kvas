// Stylized lake water: flat colour bands, drifting foam lines and a hard highlight.
// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.
#include "shaders/kvas_mesh_api.wgsl"
const DEEP: vec3<f32> = vec3<f32>(0.02, 0.18, 0.25);
const SHALLOW: vec3<f32> = vec3<f32>(0.15, 0.55, 0.55);
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
    let wave = sin(p.x * 2.0 + t * SPEED) * 0.5 + sin(p.y * 2.6 - t * SPEED * 0.7) * 0.5 + fbm3(vec3<f32>(p * 0.8, t * 0.2), 3);
    let band = floor(clamp(wave * 0.4 + 0.5, 0.0, 1.0) * 3.0) / 3.0;
    let foam = step(0.92, fract(wave * 2.0 + t * 0.1)) * 0.5;
    let col = mix(DEEP, SHALLOW, band) + foam + vec3<f32>(step(0.985, fbm3(vec3<f32>(p * 3.0, t * 0.5), 2)));
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);
}
