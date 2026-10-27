// Built-in texture-paint shader (the prelude is prepended). Four surface textures are blended by a
// painted mask: albedo slot = mask (R,G,B = weights of layers 2,3,4), normal slot = a 2x2 atlas of
// the four layers. Layers are sampled in world space, so the pattern does not stretch with the mesh.
// object.params.z = texture repeats per metre, object.user0.x = border roughness (noise breakup).

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

fn layer_color(index: u32, p: vec2<f32>) -> vec3<f32> {
    let tile = vec2<f32>(f32(index % 2u), f32(index / 2u));
    // Stay a hair inside each quadrant so mip filtering does not bleed the neighbours in.
    let f = fract(p) * 0.96 + vec2<f32>(0.02);
    let c = textureSample(normal_tex, albedo_smp, (tile + f) * 0.5).rgb;
    return pow(c, vec3<f32>(2.2));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let m = textureSample(albedo_tex, albedo_smp, in.uv);
    var w = vec4<f32>(max(1.0 - m.r - m.g - m.b, 0.0), m.r, m.g, m.b);

    // Organic borders: world-space noise nudges the weights, then a soft contrast keeps blends clean.
    let breakup = object.user0.x;
    if (breakup > 0.0) {
        let noise = fbm2(in.world_pos.xz * 0.9, 3);
        w = max(w + (noise - 0.5) * breakup * vec4<f32>(0.6, -0.2, 0.2, -0.2) * 2.0, vec4<f32>(0.0));
    }
    let sharp = pow(w, vec4<f32>(1.4));
    w = sharp / max(sharp.x + sharp.y + sharp.z + sharp.w, 1e-4);

    let rep = max(object.params.z, 0.001);
    let p = in.world_pos.xz * rep;
    var col = layer_color(0u, p) * w.x + layer_color(1u, p) * w.y + layer_color(2u, p) * w.z + layer_color(3u, p) * w.w;
    col *= object.color.rgb;
    let lit = lighting(in.world_pos, n, col, 0.9);
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), 1.0);
}
