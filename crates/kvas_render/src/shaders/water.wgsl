// Built-in water shader (the prelude is prepended). Gerstner-like swell in the vertex stage,
// layered ripple normals, Fresnel blend between the absorbed under-water colour and the live sky
// reflection, sun glints, and foam: on crests, along shores and in rings around every object that
// touches the surface (read from the scene depth behind the water).
//
// object.color.rgb  deep-water tint      object.color.a  base opacity
// object.params2.y  reflectivity          object.user0    x wave height, y wave length, z foam amount, w clarity (metres)
// object.user1      x flow speed, y shore foam width (metres), z ripple strength

fn wave_params() -> vec4<f32> {
    // Defaults when the material leaves the user values at zero.
    let height = object.user0.x; // 0: no extra swell (VoxelWater already moves its mesh)
    let length = select(object.user0.y, 6.0, object.user0.y == 0.0);
    let foam = select(object.user0.z, 1.0, object.user0.z == 0.0);
    let clarity = select(object.user0.w, 2.5, object.user0.w == 0.0);
    return vec4<f32>(height, length, foam, clarity);
}

fn flow_params() -> vec3<f32> {
    let speed = select(object.user1.x, 1.0, object.user1.x == 0.0);
    let shore = select(object.user1.y, 0.6, object.user1.y == 0.0);
    let ripple = select(object.user1.z, 1.0, object.user1.z == 0.0);
    return vec3<f32>(speed, shore, ripple);
}

// Sum of a few directional waves: height and its slope.
fn swell(p: vec2<f32>, t: f32) -> vec3<f32> {
    let wp = wave_params();
    var h = 0.0;
    var d = vec2<f32>(0.0);
    let dirs = array<vec2<f32>, 4>(vec2<f32>(1.0, 0.25), vec2<f32>(-0.4, 1.0), vec2<f32>(0.7, -0.8), vec2<f32>(-1.0, -0.3));
    for (var i = 0; i < 4; i = i + 1) {
        let k = 6.2831853 / (wp.y * (1.0 - f32(i) * 0.17));
        let dir = normalize(dirs[i]);
        let phase = dot(dir, p) * k + t * sqrt(9.81 * k) * 0.6;
        let a = wp.x / (1.0 + f32(i) * 0.8);
        h += a * sin(phase);
        d += dir * (a * k * cos(phase));
    }
    return vec3<f32>(h, d);
}

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var out = standard_vertex(v);
    let s = swell(out.world_pos.xz, globals.time.x * flow_params().x);
    out.world_pos.y += s.x;
    out.normal = normalize(out.normal + vec3<f32>(-s.y, 0.0, -s.z));
    out.clip = globals.view_proj * vec4<f32>(out.world_pos, 1.0);
    return out;
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
    let wp = wave_params();
    let fp = flow_params();
    let t = globals.time.x * fp.x;
    let p = in.world_pos.xz;
    let e = 0.08;
    let h0 = ripple_height(p, t);
    let hx = ripple_height(p + vec2<f32>(e, 0.0), t);
    let hz = ripple_height(p + vec2<f32>(0.0, e), t);
    let slope = vec2<f32>(hx - h0, hz - h0) / e;
    let geo = normalize(in.normal);
    let n = normalize(geo + vec3<f32>(-slope.x, 0.0, -slope.y) * 0.22 * fp.z);

    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let ndv = clamp(dot(n, v), 0.0, 1.0);
    let fresnel = 0.02 + 0.98 * pow(1.0 - ndv, 5.0);

    // How much water lies between the surface and the ground (or an object) behind it.
    let behind = scene_depth_behind(in.clip, in.world_pos);
    let has_depth = behind < 999.0;
    let thickness = select(30.0, behind, has_depth);

    // Reflection of the live sky (also carries clouds, stars and the moon at night).
    var r = reflect(-v, n);
    r.y = abs(r.y);
    let reflection = env_color(r);

    // Body colour: light is absorbed with depth, red first, so shallow water turns turquoise.
    let deep = object.color.rgb;
    let shallow = deep * 1.6 + vec3<f32>(0.05, 0.22, 0.2);
    let absorb = 1.0 - exp(-thickness / max(wp.w, 0.05) * vec3<f32>(1.6, 0.75, 0.55));
    let body = mix(shallow, deep, clamp(absorb, vec3<f32>(0.0), vec3<f32>(1.0)));
    let sun_up = clamp(sky_sun_direction().y * 3.0 + 0.2, 0.15, 1.0);
    var col = mix(body * (0.35 + 0.65 * sun_up), reflection, clamp(fresnel * max(object.params2.y, 0.6) + 0.04, 0.0, 1.0));

    // Caustic shimmer seen on the bottom through shallow water.
    let bottom = scene_point_behind(in.clip, in.world_pos);
    let caustic = pow(1.0 - abs(fbm2(bottom.xz * 1.7 + vec2<f32>(t * 0.4, t * 0.25), 3) * 2.0 - 1.0), 6.0);
    col += vec3<f32>(0.5, 0.8, 0.75) * caustic * exp(-thickness * 1.2) * 0.35 * sun_up;

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

    // Foam: on steep crests, and where the water touches the shore or an object. Contact foam
    // is broken into moving streaks and pulses outwards in rings.
    let crest = clamp((1.0 - geo.y) * 7.0, 0.0, 1.0);
    let speck = smoothstep(0.62, 0.8, fbm2(p * 3.0 + vec2<f32>(t * 0.4, -t * 0.3), 2));
    let shore = select(0.0, 1.0 - smoothstep(0.0, fp.y, thickness), has_depth);
    let rings = 0.5 + 0.5 * sin(thickness / max(fp.y, 0.05) * 9.0 - globals.time.x * 3.0);
    let breakup = smoothstep(0.35, 0.75, fbm2(p * 4.5 + vec2<f32>(t * 0.6, t * 0.45), 3) + shore * 0.45);
    let contact = clamp(shore * (0.55 + 0.45 * rings) * breakup + pow(shore, 6.0), 0.0, 1.0);
    let foam = clamp(crest * speck * 0.6 + contact, 0.0, 1.0) * wp.z;
    let foam_col = vec3<f32>(0.92, 0.96, 1.0) * (0.45 + 0.55 * sun_up) * (0.6 + 0.4 * max(dot(n, l), 0.0));
    col = mix(col, foam_col, clamp(foam, 0.0, 1.0));

    // Shallow water fades out softly where it meets the ground (no hard edge).
    let edge = select(1.0, smoothstep(0.0, 0.12, thickness), has_depth);
    let depth_alpha = select(1.0, clamp(absorb.g * 1.15, 0.0, 1.0), has_depth);
    let alpha = clamp(max(object.color.a, depth_alpha) + fresnel * 0.4 + foam * 0.6, 0.0, 1.0) * edge;
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), alpha);
}
