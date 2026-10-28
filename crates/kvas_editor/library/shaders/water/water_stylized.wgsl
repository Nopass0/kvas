// Stylised (toon) water: flat colour bands by depth, crisp white foam lines around every object
// and along the shore, animated by noise. Reads the scene depth, so keep the colour alpha below 1
// (or keep "water" in the file name, which also makes the object draw after the opaque scene).
//   object.color  deep colour     object.user0  x foam width (m), y foam speed, z band depth (m)
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var out = standard_vertex(v);
    let t = globals.time.x;
    out.world_pos.y += sin(out.world_pos.x * 0.6 + t) * 0.04 + sin(out.world_pos.z * 0.8 - t * 1.3) * 0.03;
    out.clip = globals.view_proj * vec4<f32>(out.world_pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let width = select(object.user0.x, 0.35, object.user0.x == 0.0);
    let speed = select(object.user0.y, 1.0, object.user0.y == 0.0);
    let band = select(object.user0.z, 1.5, object.user0.z == 0.0);
    let t = globals.time.x * speed;
    let depth = min(scene_depth_behind(in.clip, in.world_pos), 30.0);
    // Three flat colour bands: shallow, middle, deep.
    let deep = object.color.rgb;
    let mid = deep * 1.6 + vec3<f32>(0.0, 0.08, 0.08);
    let shallow = deep * 2.2 + vec3<f32>(0.1, 0.3, 0.28);
    var col = select(select(deep, mid, depth < band * 2.0), shallow, depth < band);
    // Foam lines: one solid ring at the contact and a broken ring further out.
    let wobble = fbm2(in.world_pos.xz * 2.5 + vec2<f32>(t * 0.3, -t * 0.2), 2) * 0.25;
    let ring1 = step(depth + wobble * width, width * 0.5);
    let ring2 = step(abs(depth - width * (1.2 + 0.3 * sin(t * 1.5))) + wobble * width * 0.3, width * 0.12) * step(0.45, fbm2(in.world_pos.xz * 4.0 + t * 0.2, 2));
    // Little sparkle lines on the open water.
    let streak = step(0.78, fbm2(in.world_pos.xz * vec2<f32>(0.8, 3.0) + vec2<f32>(t * 0.5, 0.0), 3));
    let foam = max(max(ring1, ring2), streak * 0.6);
    let lit = col * (0.6 + 0.4 * max(dot(normalize(in.normal), normalize(globals.sun_dir.xyz)), 0.0)) + globals.ambient.xyz * col * 0.3;
    let final_col = mix(lit, vec3<f32>(0.95, 0.98, 1.0), foam);
    let alpha = clamp(max(object.color.a, smoothstep(0.0, band, depth) * 0.85) + foam, 0.0, 1.0);
    return vec4<f32>(to_display_at(apply_fog(final_col, in.world_pos), in.clip), alpha);
}
