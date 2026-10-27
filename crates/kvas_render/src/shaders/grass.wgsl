// Built-in grass shader (the prelude is prepended). Wind, distance thinning and the stylised
// look are all computed on the GPU, so the chunk meshes never change.
//   object.user0 = (wind strength, wind speed, fade start, fade end)
//   object.user1 = (tip colour rgb, patchiness)    object.color = base colour

fn gust_front(p: vec2<f32>) -> f32 {
    let t = globals.time.x;
    return value_noise(p * 0.12 + vec2<f32>(t * 0.45, t * 0.18));
}

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    let world0 = (object.model * vec4<f32>(v.position, 1.0)).xyz;
    let root_y = object.model[3].y;
    let dist = distance(world0.xz, globals.camera_pos.xz);
    // Far away, a growing share of blades shrinks to nothing; the rest keep their size.
    let fade = smoothstep(object.user0.z, object.user0.w, dist);
    let keep = clamp((v.uv.x - fade) * 8.0 + 1.0, 0.0, 1.0);

    let strength = object.user0.x;
    let speed = object.user0.y;
    let t = v.uv.y;
    let bend = t * t * keep;
    let seed = v.uv.x * 6.2831;
    let time = globals.time.x;
    let sway = sin(time * speed + world0.x * 0.55 + world0.z * 0.35 + seed) * 0.5 + sin(time * speed * 2.1 + seed * 1.7) * 0.15;
    let gust = (gust_front(world0.xz) - 0.35) * 2.2;
    let push = (sway * 0.35 + gust) * strength * bend;

    var w = world0;
    w.y = root_y + (world0.y - root_y) * keep;
    w.x += push * 0.9;
    w.z += push * 0.4 + sway * 0.12 * strength * bend;
    w.y -= abs(push) * 0.35;

    var out: VsOut;
    out.clip = globals.view_proj * vec4<f32>(w, 1.0);
    out.world_pos = w;
    out.normal = normalize((object.normal_matrix * vec4<f32>(v.normal, 0.0)).xyz);
    out.uv = v.uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let g = in.uv.y;
    let base = object.color.rgb;
    let tip = pow(object.user1.xyz, vec3<f32>(2.2));
    let patchiness = object.user1.w;
    let strength = object.user0.x;

    // Three-stop gradient: dark root, saturated body, bright tip.
    let mid = mix(base, tip, 0.5) * vec3<f32>(1.05, 1.0, 0.8);
    var albedo = mix(base, mid, smoothstep(0.0, 0.45, g));
    albedo = mix(albedo, tip, smoothstep(0.4, 1.0, g));
    // Per-blade brightness, plus meadow-scale patches of dry and lush grass.
    albedo *= 0.8 + 0.4 * fract(in.uv.x * 7.3);
    let dry = smoothstep(0.55, 0.85, value_noise(in.world_pos.xz * 0.28));
    let lush = smoothstep(0.6, 0.9, value_noise(in.world_pos.xz * 0.9 + vec2<f32>(7.0, 3.0)));
    albedo = mix(albedo, albedo * vec3<f32>(1.4, 1.2, 0.5), dry * patchiness);
    albedo = mix(albedo, albedo * vec3<f32>(0.7, 0.95, 0.8), lush * patchiness * 0.6);
    // Gusts sweeping over the field lighten the blades they bend.
    albedo *= 1.0 + smoothstep(0.5, 0.95, gust_front(in.world_pos.xz)) * 0.25 * strength;

    // Soft, painterly light: wrapped diffuse from the sun, sky ambient darkened towards the roots.
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let sun = normalize(globals.sun_dir.xyz);
    let wrap = clamp(dot(up, sun) * 0.5 + 0.5, 0.0, 1.0);
    let shadow = shadow_factor(in.world_pos, up);
    let direct = globals.sun_color.xyz * (0.3 + 0.7 * wrap) * mix(0.55, 1.0, shadow);
    let ambient = globals.ambient.xyz * 1.5 * mix(0.35, 1.0, smoothstep(0.0, 0.8, g));
    var rgb = albedo * (direct + ambient);

    // Light shining through thin blades when looking towards the sun.
    let view = normalize(globals.camera_pos.xyz - in.world_pos);
    rgb += albedo * globals.sun_color.xyz * pow(max(dot(-view, sun), 0.0), 3.0) * g * 0.5;

    // Aerial perspective: distant grass melts into the horizon colour.
    let dist = distance(in.world_pos.xz, globals.camera_pos.xz);
    let horizon = env_color(normalize(vec3<f32>(-view.x, 0.12, -view.z)));
    rgb = mix(rgb, horizon, smoothstep(object.user0.z, object.user0.w * 1.3, dist) * 0.55);
    return vec4<f32>(to_display(rgb), 1.0);
}
