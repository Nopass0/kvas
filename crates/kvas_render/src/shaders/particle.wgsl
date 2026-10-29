// Built-in particle shader (the prelude is prepended). Quad corner (-1..1) and normalised life
// (0..1) travel in the vertex normal; colours come from MeshRenderer.user (start rgba, end rgba)
// and the glow from `emissive`. Unlit and soft-edged; an optional texture supplies the shape.
// Every quad is turned to the camera that draws it: uv.x is the half size in metres, uv.y the
// mode (-1 billboard around the particle centre, -2 streak: the length is already in the
// position, only the width is turned). Positive uv.y means a plain textured quad.

struct PIn {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) corner_life: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(v: VertexIn) -> PIn {
    var out: PIn;
    var world = (object.model * vec4<f32>(v.position, 1.0)).xyz;
    out.uv = v.uv;
    if (v.uv.y < 0.0) {
        let to_eye = normalize(globals.camera_pos.xyz - world);
        var right = cross(vec3<f32>(0.0, 1.0, 0.0), to_eye);
        if (dot(right, right) < 1e-6) {
            right = vec3<f32>(1.0, 0.0, 0.0);
        }
        right = normalize(right);
        world += right * (v.normal.x * v.uv.x);
        if (v.uv.y > -1.5) {
            world += cross(to_eye, right) * (v.normal.y * v.uv.x);
        }
        out.uv = vec2<f32>(v.normal.x * 0.5 + 0.5, v.normal.y * 0.5 + 0.5);
    }
    out.clip = globals.view_proj * vec4<f32>(world, 1.0);
    out.world_pos = world;
    out.corner_life = v.normal;
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
    var alpha = tint.a * tex.a * soft;
    // Soft particles: fade where the quad cuts into a surface (object.params.x = distance).
    if (object.params.x > 0.0) {
        alpha *= smoothstep(0.0, object.params.x, scene_depth_behind(in.clip, in.world_pos));
    }
    if (alpha < 0.003) {
        discard;
    }
    let col = tint.rgb * tex.rgb * max(object.params.y, 0.0);
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), clamp(alpha, 0.0, 1.0));
}
