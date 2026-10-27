// Full-screen procedural sky (the prelude is prepended). Drawn first, behind everything.

struct SkyOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn sky_vs(@builtin(vertex_index) i: u32) -> SkyOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    let pos = uv * 2.0 - vec2<f32>(1.0);
    var out: SkyOut;
    out.clip = vec4<f32>(pos, 1.0, 1.0);
    out.ndc = pos;
    return out;
}

@fragment
fn sky_fs(in: SkyOut) -> @location(0) vec4<f32> {
    let far = globals.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let dir = normalize(far.xyz / far.w - globals.camera_pos.xyz);
    if globals.environment.x > 0.5 { return vec4<f32>(to_display(hdri_color(dir)), 1.0); }
    return vec4<f32>(to_display(sky_radiance(dir, true)), 1.0);
}
