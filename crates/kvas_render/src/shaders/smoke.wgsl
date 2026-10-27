// Built-in shader for voxel smoke (the prelude is prepended). Voxel density arrives in uv.x.
// Soft silhouette (edges fade where the surface turns away), a light-to-dark ramp by density and a
// little forward scattering of the sun so smoke stays readable at dusk and in shadow.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return standard_vertex(v);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let density = clamp(in.uv.x, 0.0, 1.0);
    let n = normalize(in.normal);
    let view = normalize(globals.camera_pos.xyz - in.world_pos);
    let facing = clamp(dot(n, view), 0.0, 1.0);
    let base = object.color.rgb * mix(1.15, 0.45, density);
    var lit = lighting(in.world_pos, n, base, 1.0);
    // Sun shining through the thin parts.
    let through = pow(clamp(dot(-view, normalize(globals.sun_dir.xyz)) * 0.5 + 0.5, 0.0, 1.0), 4.0);
    lit += globals.sun_color.rgb * base * through * (1.0 - density) * 0.6;
    lit += base * 0.18;
    let soft = 0.35 + 0.65 * pow(facing, 0.6);
    let alpha = clamp(density * 1.25, 0.0, 1.0) * soft * object.color.a;
    return vec4<f32>(to_display(apply_fog(lit + base * object.params.y, in.world_pos)), alpha);
}
