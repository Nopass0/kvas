// Rim-glow shader that reuses the engine's standard vertex stage and lighting.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    return vec4<f32>(to_display(lit + rim * pulse * vec3<f32>(0.5, 0.3, 0.8)), 1.0);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let view_dir = normalize(globals.camera_pos.xyz - in.world_pos);
    let rim = pow(1.0 - max(dot(n, view_dir), 0.0), 5.0);
    let pulse = 0.6 + 0.4 * sin(globals.time.x * 03.0);
    let lit = lighting(in.world_pos, n, object.color.rgb, 0.3);
    return vec4<f32>(to_display(lit + rim * pulse * vec3<f32>(0.5, 0.3, 0.8)), 1.0);
}
