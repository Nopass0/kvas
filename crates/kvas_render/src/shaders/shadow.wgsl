// Depth-only pass from the sun's point of view.

struct LightView {
    vp: mat4x4<f32>,
};

struct Object {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    color: vec4<f32>,
    params: vec4<f32>,
    params2: vec4<f32>,
    user0: vec4<f32>,
    user1: vec4<f32>,
};

@group(0) @binding(0) var<uniform> light: LightView;
@group(1) @binding(0) var<uniform> object: Object;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return light.vp * object.model * vec4<f32>(position, 1.0);
}
