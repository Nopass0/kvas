// Procedural brick wall on world X/Y (or Z/Y). user 1 = brick width (default 0.5).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let w = select(object.user0.x, 0.5, object.user0.x <= 0.0);
    let u = select(in.world_pos.x, in.world_pos.z, abs(n.x) > abs(n.z));
    let p = vec2<f32>(u / w, in.world_pos.y / (w * 0.5));
    let row = floor(p.y);
    let q = vec2<f32>(p.x + select(0.0, 0.5, (row % 2.0) > 0.5), p.y);
    let cell = floor(q);
    let f = fract(q);
    let mortar = 1.0 - smoothstep(0.0, 0.08, min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y)) - 0.04);
    let shade = 0.75 + 0.5 * hash21(cell);
    let brick = object.color.rgb * shade * (0.9 + 0.2 * value_noise(q * 12.0));
    let base = mix(brick, vec3<f32>(0.62, 0.6, 0.55), mortar);
    let col = lighting(in.world_pos, n, base, 0.9);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
