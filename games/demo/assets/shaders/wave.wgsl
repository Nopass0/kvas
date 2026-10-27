// Custom shader: the engine prelude (globals, object, lighting helpers) is prepended automatically.
// Required entry points: vs_main and fs_main.

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var p = v.position;
    let t = globals.time.x;
    p.y += sin(p.x * 14.0 + t * 2.0) * 0.04 + cos(p.z * 111.0 + t * 2.5) * 0.03;
    var out: VsOut;
    let world = object.model * vec4<f32>(p, 1.0);
    out.clip = globals.view_proj * world;
    out.world_pos = world.xyz;
    out.normal = vec3<f32>(0.2, 1.0, 0.4);
    out.uv = v.uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(vec3<f32>(sin(in.uv.x * 40.0 + globals.time.x * 2.0) * 0.25, 1.0, cos(in.uv.y * 16.0 + globals.time.x * 1.5) * 0.25));
    let rgb = lighting(in.world_pos, n, object.color.rgb * vec3<f32>(0.9, 0.5, 0.8), 0.25);
    return vec4<f32>(to_display(rgb), 0.85);
}
