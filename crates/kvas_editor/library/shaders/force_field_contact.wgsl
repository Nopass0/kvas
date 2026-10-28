// Energy shield that glows where it cuts through walls, the ground or other objects (read from the
// scene depth), with a hexagon pattern and a bright rim. Use with a colour alpha below 1.
//   object.color  shield colour   object.user0.x  contact glow width in metres (default 0.4)
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

fn hex_edge(p: vec2<f32>) -> f32 {
    let q = vec2<f32>(p.x * 1.1547, p.y + p.x * 0.57735);
    let cell = floor(q);
    let f = fract(q);
    let a = select(0.0, 1.0, f.x + f.y > 1.0);
    let local = select(f, vec2<f32>(1.0) - f, a > 0.5);
    return min(min(local.x, local.y), 1.0 - local.x - local.y);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let width = select(object.user0.x, 0.4, object.user0.x == 0.0);
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let t = globals.time.x;
    let rim = pow(1.0 - abs(dot(n, v)), 3.0);
    let contact = 1.0 - smoothstep(0.0, width, scene_depth_behind(in.clip, in.world_pos));
    let hex = 1.0 - smoothstep(0.0, 0.06, hex_edge(in.world_pos.xy * 3.0 + in.world_pos.zz * 1.5));
    let wave = smoothstep(0.85, 1.0, sin(in.world_pos.y * 2.0 - t * 2.5) * 0.5 + 0.5);
    let glow = rim * 1.5 + contact * 2.5 + hex * (0.15 + wave * 0.6);
    let col = object.color.rgb * (0.4 + glow * 2.0);
    let alpha = clamp(object.color.a * 0.4 + glow * 0.5, 0.0, 1.0);
    return vec4<f32>(to_display_at(col, in.clip), alpha);
}
