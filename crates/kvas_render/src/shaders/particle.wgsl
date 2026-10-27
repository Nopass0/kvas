// Built-in particle shader (the prelude is prepended). Quad corner (-1..1) and normalised life
// (0..1) travel in the vertex normal; colours come from MeshRenderer.user (start rgba, end rgba)
// and the glow from `emissive`. Unlit and soft-edged; an optional texture supplies the shape.

struct PIn {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) corner_life: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(v: VertexIn) -> PIn {
    var out: PIn;
    let world = object.model * vec4<f32>(v.position, 1.0);
    out.clip = globals.view_proj * world;
    out.world_pos = world.xyz;
    out.corner_life = v.normal;
    out.uv = v.uv;
    return out;
}

@fragment
fn fs_main(in: PIn) -> @location(0) vec4<f32> {
    let r = length(in.corner_life.xy);
    if (r > 1.0) {
        discard;
    }
    let life = clamp(in.corner_life.z, 0.0, 1.0);
    let soft = 1.0 - smoothstep(0.25, 1.0, r);
    let tex = textureSample(albedo_tex, albedo_smp, in.uv);
    let tint = mix(object.user0, object.user1, life);
    let alpha = tint.a * tex.a * soft;
    if (alpha < 0.003) {
        discard;
    }
    let col = tint.rgb * tex.rgb * max(object.params.y, 0.0);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), clamp(alpha, 0.0, 1.0));
}
