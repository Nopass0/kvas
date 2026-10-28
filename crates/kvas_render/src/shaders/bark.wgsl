// Built-in bark shader for procedural trees (the prelude is prepended): vertical furrows, knots,
// moss on the upward faces, and the same wind sway as the leaves so branches move with them.
//   object.color  bark colour      object.user0.x  wind
//   uv.x          around the branch (0..1), uv.y along it (in bark-sized steps)

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var out = standard_vertex(v);
    let root = object.model[3].xyz;
    let up = max(out.world_pos.y - root.y, 0.0);
    let t = globals.time.x;
    let gust = 0.6 + 0.4 * sin(t * 0.7 + root.x * 0.11 + root.z * 0.07);
    let sway = sin(t * 1.25 + root.x * 0.3 + root.z * 0.2) * 0.018 * up * object.user0.x * gust;
    out.world_pos += vec3<f32>(sway, 0.0, sway * 0.55);
    out.clip = globals.view_proj * vec4<f32>(out.world_pos, 1.0);
    return out;
}

fn bark_height(uv: vec2<f32>) -> f32 {
    let warp = fbm2(vec2<f32>(uv.x * 4.0, uv.y * 0.35), 3);
    let furrow = abs(fract(uv.x * 7.0 + warp * 0.9) - 0.5) * 2.0;
    let plates = fbm2(vec2<f32>(uv.x * 14.0, uv.y * 1.6), 3);
    return smoothstep(0.15, 0.75, furrow) * 0.7 + plates * 0.3;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(in.uv.x, in.uv.y * 0.5);
    let h = bark_height(uv);
    let e = 0.01;
    let dh = vec2<f32>(bark_height(uv + vec2<f32>(e, 0.0)) - h, bark_height(uv + vec2<f32>(0.0, e)) - h) / e;
    let n0 = normalize(in.normal);
    // Perturb the normal across the branch (furrows) and along it.
    let along = normalize(cross(n0, vec3<f32>(0.0, 1.0, 0.0)) + vec3<f32>(0.0001));
    let n = normalize(n0 - along * dh.x * 0.04 - vec3<f32>(0.0, 1.0, 0.0) * dh.y * 0.01);

    var albedo = object.color.rgb * (0.45 + 0.75 * h);
    // Knots and lichen.
    let knot = smoothstep(0.82, 0.92, fbm2(uv * vec2<f32>(3.0, 0.8) + vec2<f32>(5.0, 1.0), 2));
    albedo = mix(albedo, object.color.rgb * 0.35, knot);
    let moss = smoothstep(0.35, 0.8, n0.y) * smoothstep(0.4, 0.7, fbm2(in.world_pos.xz * 3.0 + in.world_pos.yy, 3));
    albedo = mix(albedo, vec3<f32>(0.16, 0.24, 0.07), moss * 0.8);
    let ao = 0.55 + 0.45 * h;
    let lit = lighting_pbr(in.world_pos, n, albedo, 0.92, 0.0, ao);
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), 1.0);
}
