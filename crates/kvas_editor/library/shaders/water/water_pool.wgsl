// Clear pool / lagoon water: you see the bottom through it, with caustic light patterns on the
// floor and a turquoise tint that grows with depth. Soft foam where objects break the surface.
// Keep "water" in the file name (or the colour alpha below 1) so the scene depth is available.
//   object.color  tint            object.user0  x clarity (m), y caustic strength, z foam width (m)
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let clarity = select(object.user0.x, 4.0, object.user0.x == 0.0);
    let caustics_k = select(object.user0.y, 1.0, object.user0.y == 0.0);
    let foam_w = select(object.user0.z, 0.15, object.user0.z == 0.0);
    let t = globals.time.x;
    let p = in.world_pos.xz;
    // Small ripples for the surface normal.
    let e = 0.05;
    let h = fbm2(p * 3.0 + vec2<f32>(t * 0.4, t * 0.3), 3);
    let hx = fbm2(p * 3.0 + vec2<f32>(e, 0.0) * 3.0 + vec2<f32>(t * 0.4, t * 0.3), 3);
    let hz = fbm2(p * 3.0 + vec2<f32>(0.0, e) * 3.0 + vec2<f32>(t * 0.4, t * 0.3), 3);
    let n = normalize(vec3<f32>(-(hx - h) / e * 0.08, 1.0, -(hz - h) / e * 0.08));
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let fresnel = 0.02 + 0.98 * pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 5.0);

    let depth = scene_depth_behind(in.clip, in.world_pos);
    let has_floor = depth < 999.0;
    let d = select(30.0, depth, has_floor);
    let floor_p = scene_point_behind(in.clip, in.world_pos);
    // Caustics: two sharp noise layers multiplied, projected on the floor.
    let c1 = pow(1.0 - abs(fbm2(floor_p.xz * 2.2 + vec2<f32>(t * 0.5, t * 0.2), 3) * 2.0 - 1.0), 8.0);
    let c2 = pow(1.0 - abs(fbm2(floor_p.xz * 3.1 - vec2<f32>(t * 0.3, t * 0.45), 3) * 2.0 - 1.0), 8.0);
    let caustic = (c1 + c2) * caustics_k * exp(-d * 0.6) * globals.sun_color.xyz;
    let tint = 1.0 - exp(-d / clarity * vec3<f32>(1.4, 0.55, 0.4));
    var r = reflect(-v, n);
    r.y = abs(r.y);
    let body = object.color.rgb * tint + caustic * 0.6;
    var col = mix(body, env_color(r), fresnel);
    col += globals.sun_color.xyz * pow(max(dot(r, normalize(globals.sun_dir.xyz)), 0.0), 300.0) * 3.0;
    let foam = select(0.0, 1.0 - smoothstep(0.0, foam_w, d), has_floor) * smoothstep(0.4, 0.7, fbm2(p * 6.0 + t * 0.5, 2) + 0.3);
    col = mix(col, vec3<f32>(0.95), foam);
    // Transparent where shallow, so the floor (and its caustics, added above) shows through.
    let alpha = clamp(max(tint.g * 1.2, fresnel) + foam, 0.05, 1.0);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), alpha);
}
