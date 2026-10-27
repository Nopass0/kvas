struct Lines {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> lines: Lines;

struct In {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};
struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(v: In) -> Out {
    var out: Out;
    out.clip = lines.view_proj * vec4<f32>(v.position, 1.0);
    out.color = v.color;
    return out;
}

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    return in.color;
}
