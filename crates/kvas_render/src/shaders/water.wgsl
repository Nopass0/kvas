// Built-in water shader (the prelude is prepended). Animated ripple normals from layered noise,
// Fresnel blend between a deep-water tint and the live sky reflection, sun glints and crest foam.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

fn ripple_height(p: vec2<f32>, t: f32) -> f32 {
    var h = 0.0;
    h += fbm2(p * 0.9 + vec2<f32>(t * 0.35, t * 0.18), 3) * 1.0;
    h += fbm2(p * 2.3 + vec2<f32>(-t * 0.5, t * 0.31), 2) * 0.45;
    h += fbm2(p * 5.1 + vec2<f32>(t * 0.9, -t * 0.7), 2) * 0.18;
    return h;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let t = globals.time.x;
    let p = in.world_pos.xz;
    let e = 0.08;
    let h0 = ripple_height(p, t);
    let hx = ripple_height(p + vec2<f32>(e, 0.0), t);
    let hz = ripple_height(p + vec2<f32>(0.0, e), t);
    let slope = vec2<f32>(hx - h0, hz - h0) / e;
    let geo = normalize(in.normal);
    let n = normalize(geo + vec3<f32>(-slope.x, 0.0, -slope.y) * 0.22);

    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let ndv = clamp(dot(n, v), 0.0, 1.0);
    let fresnel = 0.02 + 0.98 * pow(1.0 - ndv, 5.0);

    // Reflection of the live sky (also carries clouds, stars and the moon at night).
    var r = reflect(-v, n);
    r.y = abs(r.y);
    let reflection = env_color(r);

    // Tint: deeper and darker looking straight down, lighter and greener at grazing angles.
    let deep = object.color.rgb;
    let shallow = deep * 1.8 + vec3<f32>(0.02, 0.07, 0.06);
    let body = mix(deep, shallow, pow(1.0 - ndv, 2.0));
    let sun_up = clamp(sky_sun_direction().y * 3.0 + 0.2, 0.15, 1.0);
    var col = mix(body * (0.35 + 0.65 * sun_up), reflection, clamp(fresnel * object.params2.y + 0.04, 0.0, 1.0));

    // Sun glints.
    let l = normalize(globals.sun_dir.xyz);
    let spec = pow(max(dot(r, l), 0.0), 220.0);
    col += globals.sun_color.xyz * spec * 2.5;

    // Light scattering through wave crests facing the sun (the glassy green-turquoise look).
    let lift = clamp(h0 * 1.2, 0.0, 1.0);
    let sss = pow(clamp(dot(-v, l) * 0.5 + 0.5, 0.0, 1.0), 3.0) * lift * sun_up;
    col += vec3<f32>(0.05, 0.38, 0.32) * sss * 1.2;

    // Tiny sparkles where the surface catches the sun.
    let cell = floor(p * 26.0 + vec2<f32>(t * 1.5, -t));
    let spark = step(0.985, hash21(cell)) * pow(max(dot(r, l), 0.0), 40.0);
    col += globals.sun_color.xyz * spark * 6.0;

    // Foam on steep crests and shifting speckles.
    let crest = clamp((1.0 - geo.y) * 7.0, 0.0, 1.0);
    let speck = smoothstep(0.62, 0.8, fbm2(p * 3.0 + vec2<f32>(t * 0.4, -t * 0.3), 2));
    let foam = crest * speck;
    col = mix(col, vec3<f32>(0.9, 0.95, 1.0) * (0.4 + 0.6 * sun_up), foam * 0.6);

    let alpha = clamp(object.color.a + fresnel * 0.4 + foam * 0.3, 0.0, 1.0);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), alpha);
}
