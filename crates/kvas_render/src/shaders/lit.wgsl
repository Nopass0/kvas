// Built-in shader (the prelude is prepended automatically).

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

// Debug views (Scene view toolbar): 1 albedo, 2 normals, 3 depth with contour lines, 4 UV checker.
fn debug_color(mode: u32, in: VsOut, base: vec4<f32>) -> vec3<f32> {
    let n = normalize(in.normal);
    if (mode == 1u) {
        return base.rgb;
    }
    if (mode == 2u) {
        return n * 0.5 + vec3<f32>(0.5);
    }
    if (mode == 3u) {
        let d = distance(in.world_pos, globals.camera_pos.xyz);
        let t = clamp(d / 60.0, 0.0, 1.0);
        let ramp = mix(vec3<f32>(1.0, 0.35, 0.1), vec3<f32>(0.1, 0.35, 1.0), t) * (1.0 - 0.7 * t);
        let line = smoothstep(0.46, 0.5, abs(fract(d / 4.0) - 0.5));
        return mix(ramp, vec3<f32>(1.0), line * 0.6);
    }
    if (mode == 7u) {
        // Material channels: red = roughness, green = metallic, blue = emissive.
        return vec3<f32>(object.params.x, object.params2.x, clamp(object.params.y, 0.0, 1.0));
    }
    if (mode == 8u) {
        // Creases and silhouettes from screen-space derivatives, over a dark clay.
        let crease = length(fwidth(n)) * 6.0 + length(fwidth(in.world_pos)) * 0.0;
        let grid = smoothstep(vec2<f32>(0.46), vec2<f32>(0.5), abs(fract(in.uv * 8.0) - vec2<f32>(0.5)));
        let line = clamp(crease + max(grid.x, grid.y) * 0.25, 0.0, 1.0);
        let shade = 0.18 + 0.22 * max(dot(n, normalize(vec3<f32>(0.4, 0.8, 0.3))), 0.0);
        return mix(vec3<f32>(shade), vec3<f32>(0.75, 0.9, 1.0), line);
    }
    let cell = floor(in.uv * 8.0);
    let checker = abs((cell.x + cell.y) % 2.0);
    return mix(vec3<f32>(in.uv, 0.2), vec3<f32>(0.9), checker * 0.35);
}

// Albedo / normal / ORM samples for the fragment, planar (UV) or triplanar (world space, for rocks).
struct Surface {
    albedo: vec4<f32>,
    normal_ts: vec3<f32>,   // tangent-space normal (planar mode) or world-space perturbation (triplanar)
    orm: vec3<f32>,
}

fn sample_planar(uv: vec2<f32>) -> Surface {
    var s: Surface;
    if (globals.sky_b.z > 0.5) {
        // PSX: nearest-neighbour texels, no filtering or mips.
        let dim = vec2<f32>(textureDimensions(albedo_tex));
        let t = vec2<i32>(floor(fract(uv) * dim));
        s.albedo = textureLoad(albedo_tex, t, 0);
    } else {
        s.albedo = textureSample(albedo_tex, albedo_smp, uv);
    }
    s.normal_ts = textureSample(normal_tex, albedo_smp, uv).xyz * 2.0 - 1.0;
    s.orm = textureSample(orm_tex, albedo_smp, uv).xyz;
    return s;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let mode = u32(globals.sky_b.y + 0.5);
    let pbr = object.params2.z > 0.5;
    let triplanar = object.params.w > 0.0;
    var n = normalize(in.normal);
    var surf: Surface;
    if (triplanar) {
        // Triplanar projection keeps rocks free of UV stretching and seams.
        let w0 = pow(abs(n), vec3<f32>(6.0));
        let w = w0 / (w0.x + w0.y + w0.z);
        let sc = object.params.w;
        let px = in.world_pos.zy * sc;
        let py = in.world_pos.xz * sc;
        let pz = in.world_pos.xy * sc;
        let ax = sample_planar(px);
        let ay = sample_planar(py);
        let az = sample_planar(pz);
        surf.albedo = ax.albedo * w.x + ay.albedo * w.y + az.albedo * w.z;
        surf.orm = ax.orm * w.x + ay.orm * w.y + az.orm * w.z;
        // Whiteout-style blend of the three tangent normals into a world normal.
        let sgn = vec3<f32>(select(-1.0, 1.0, n.x >= 0.0), select(-1.0, 1.0, n.y >= 0.0), select(-1.0, 1.0, n.z >= 0.0));
        let tx = vec3<f32>(ax.normal_ts.xy + n.zy, abs(n.x) * ax.normal_ts.z);
        let ty = vec3<f32>(ay.normal_ts.xy + n.xz, abs(n.y) * ay.normal_ts.z);
        let tz = vec3<f32>(az.normal_ts.xy + n.xy, abs(n.z) * az.normal_ts.z);
        let wx = vec3<f32>(tx.z * sgn.x, tx.y, tx.x);
        let wy = vec3<f32>(ty.x, ty.z * sgn.y, ty.y);
        let wz = vec3<f32>(tz.x, tz.y, tz.z * sgn.z);
        let blended = normalize(wx * w.x + wy * w.y + wz * w.z);
        n = normalize(mix(n, blended, clamp(object.params2.w, 0.0, 2.0) * 0.5 + 0.0));
        if (object.params2.w <= 0.0) {
            n = normalize(in.normal);
        }
    } else {
        surf = sample_planar(in.uv * object.params.z);
        if (pbr && object.params2.w > 0.0) {
            // Tangent frame from screen-space derivatives (no tangents in the vertex format).
            let uv = in.uv * object.params.z;
            let dp1 = dpdx(in.world_pos);
            let dp2 = dpdy(in.world_pos);
            let du1 = dpdx(uv);
            let du2 = dpdy(uv);
            let dp2perp = cross(dp2, n);
            let dp1perp = cross(n, dp1);
            let t = dp2perp * du1.x + dp1perp * du2.x;
            let b = dp2perp * du1.y + dp1perp * du2.y;
            let inv = inverseSqrt(max(max(dot(t, t), dot(b, b)), 1e-12));
            let m = surf.normal_ts * vec3<f32>(object.params2.w, object.params2.w, 1.0);
            n = normalize(t * inv * m.x + b * inv * m.y + n * m.z);
        }
    }
    // Cut-out textures (leaves, fences): transparent texels must not write colour or depth.
    if (surf.albedo.a < 0.5) {
        discard;
    }
    var base = object.color * surf.albedo;
    // 5 = lighting only (neutral white), 6 = clay: the lit result without textures or colours.
    if (mode == 5u) {
        base = vec4<f32>(0.85, 0.85, 0.85, base.a);
    } else if (mode == 6u) {
        base = vec4<f32>(0.72, 0.66, 0.6, base.a);
    }
    if (mode > 0u && mode != 5u && mode != 6u) {
        return vec4<f32>(select(debug_color(mode, in, base), to_display(base.rgb), mode == 1u), 1.0);
    }
    if (pbr || triplanar) {
        let rough = clamp(object.params.x * surf.orm.y, 0.04, 1.0);
        let lit = lighting_pbr(in.world_pos, n, base.rgb, rough, clamp(object.params2.x * surf.orm.z, 0.0, 1.0), surf.orm.x);
        return vec4<f32>(to_display_at(apply_fog(lit + base.rgb * object.params.y, in.world_pos), in.clip), base.a);
    }
    let lit = lighting_ex(in.world_pos, n, base.rgb, object.params.x, object.params2.x, object.params2.y);
    let rgb = apply_fog(lit.xyz + base.rgb * object.params.y, in.world_pos);
    return vec4<f32>(to_display_at(rgb, in.clip), clamp(base.a + lit.w, 0.0, 1.0));
}
