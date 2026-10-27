// Volumetric clouds (the prelude is prepended). The entity draws an inverted bounding box; this
// shader ray-marches the voxel density volume (volume_tex) inside it.
//   object.user0.xyz = half extent of the volume, .w = density scale
//   object.user1.xyz = wind drift (m/s),          .w = refraction strength
// Light: single scattering with a two-lobe phase function (silver lining), 4-sample sun marching
// for self-shadowing, sky-coloured ambient, and rays bent along the density gradient.

const STEPS: i32 = 56;
const LIGHT_STEPS: i32 = 4;

fn box_half() -> vec3<f32> {
    let m = object.model;
    return object.user0.xyz * vec3<f32>(length(m[0].xyz), length(m[1].xyz), length(m[2].xyz));
}

// Density (0..1) at a world position; edges of the box fade out so the cut is invisible.
fn cloud_density(p: vec3<f32>, centre: vec3<f32>, half: vec3<f32>) -> f32 {
    var uvw = (p - (centre - half)) / (half * 2.0);
    let edge = smoothstep(0.0, 0.3, uvw.x) * smoothstep(1.0, 0.7, uvw.x) * smoothstep(0.0, 0.3, uvw.z) * smoothstep(1.0, 0.7, uvw.z) * smoothstep(0.0, 0.08, uvw.y) * smoothstep(1.0, 0.92, uvw.y);
    let drift = object.user1.xyz * globals.time.x / (half * 2.0);
    uvw = vec3<f32>(fract(uvw.x - drift.x), uvw.y, fract(uvw.z - drift.z));
    return textureSampleLevel(volume_tex, albedo_smp, uvw, 0.0).r * edge * object.user0.w;
}

fn henyey_greenstein(mu: f32, g: f32) -> f32 {
    let g2 = g * g;
    return (1.0 - g2) / (12.566 * pow(1.0 + g2 - 2.0 * g * mu, 1.5));
}

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let centre = object.model[3].xyz;
    let half = box_half();
    let ro = globals.camera_pos.xyz;
    var rd = normalize(in.world_pos - ro);

    // Entry / exit of the ray through the box (works from outside and from inside).
    let inv = 1.0 / rd;
    let t1 = (centre - half - ro) * inv;
    let t2 = (centre + half - ro) * inv;
    let tmin = max(max(min(t1.x, t2.x), min(t1.y, t2.y)), min(t1.z, t2.z));
    let tmax = min(min(max(t1.x, t2.x), max(t1.y, t2.y)), max(t1.z, t2.z));
    let start = max(tmin, 0.0);
    if (tmax <= start) {
        discard;
    }
    let length_total = min(tmax - start, 900.0);
    let dt = length_total / f32(STEPS);
    // Interleaved gradient noise hides the stepping.
    let jitter = fract(52.9829189 * fract(dot(in.clip.xy, vec2<f32>(0.06711056, 0.00583715))));

    let sun = normalize(globals.sun_dir.xyz);
    let mu = dot(rd, sun);
    let phase = 0.65 * henyey_greenstein(mu, 0.6) + 0.35 * henyey_greenstein(mu, -0.2);
    let sun_light = globals.sun_color.xyz;
    let light_step = max(half.y * 0.5, 12.0) / f32(LIGHT_STEPS);
    let sigma = 0.045;
    let bend = object.user1.w;

    var transmittance = 1.0;
    var radiance = vec3<f32>(0.0);
    for (var i = 0; i < STEPS; i = i + 1) {
        if (transmittance < 0.02) {
            break;
        }
        let p = ro + rd * (start + dt * (f32(i) + jitter));
        let d = cloud_density(p, centre, half);
        if (d < 0.004) {
            continue;
        }
        if (bend > 0.0) {
            // Light bends towards denser air: a cheap, soft stand-in for refraction.
            let e = max(half.y * 0.08, 1.0);
            let grad = vec3<f32>(cloud_density(p + vec3<f32>(e, 0.0, 0.0), centre, half) - d, cloud_density(p + vec3<f32>(0.0, e, 0.0), centre, half) - d, cloud_density(p + vec3<f32>(0.0, 0.0, e), centre, half) - d);
            rd = normalize(rd + grad * bend * 1.5);
        }
        var optical = 0.0;
        for (var j = 1; j <= LIGHT_STEPS; j = j + 1) {
            optical += cloud_density(p + sun * light_step * f32(j), centre, half);
        }
        let attenuation = exp(-optical * light_step * sigma) + 0.25 * exp(-optical * light_step * sigma * 0.25);
        let height = clamp((p.y - (centre.y - half.y)) / (half.y * 2.0), 0.0, 1.0);
        let sky = mix(max(globals.ambient.xyz * 2.4, vec3<f32>(0.2, 0.2, 0.24)), vec3<f32>(0.55, 0.65, 0.85) * (0.25 + 0.75 * clamp(globals.sun_dir.y * 2.0 + 0.4, 0.0, 1.0)), height);
        let scatter = sun_light * phase * attenuation * 9.0 + sky * (0.5 + 0.5 * height);
        let extinction = exp(-d * dt * sigma);
        radiance += transmittance * (1.0 - extinction) * scatter;
        transmittance *= extinction;
    }

    let alpha = clamp(1.0 - transmittance, 0.0, 1.0);
    if (alpha < 0.003) {
        discard;
    }
    return vec4<f32>(to_display(radiance / max(alpha, 0.05)), alpha * object.color.a / 0.9);
}
