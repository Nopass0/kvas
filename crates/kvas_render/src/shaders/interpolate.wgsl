// Experimental vendor-independent block optical flow. No neural inference or DLSS.
@group(0) @binding(0) var previous: texture_2d<f32>;
@group(0) @binding(1) var current: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;
@group(0) @binding(3) var flow: texture_2d<f32>;

@vertex fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[id], 0.0, 1.0);
}
fn read_prev(p: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(previous, linear_sampler, clamp(p / vec2<f32>(textureDimensions(previous)), vec2<f32>(0.0), vec2<f32>(1.0)), 0.0).rgb;
}
fn read_curr(p: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(current, linear_sampler, clamp(p / vec2<f32>(textureDimensions(current)), vec2<f32>(0.0), vec2<f32>(1.0)), 0.0).rgb;
}
@fragment fn fs_flow(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let center = (floor(p.xy) + 0.5) * 16.0;
    var best = 1e6;
    var delta = vec2<f32>(0.0);
    for (var y = -8; y <= 8; y += 4) {
        for (var x = -8; x <= 8; x += 4) {
            let d = vec2<f32>(f32(x), f32(y));
            var error = dot(d, d) * 0.000005;
            for (var j = 0; j < 4; j++) {
                let offset = vec2<f32>(f32(j % 2) * 8.0 - 4.0, f32(j / 2) * 8.0 - 4.0);
                let diff = read_prev(center + offset + d) - read_curr(center + offset);
                error += dot(diff, diff) * 0.25;
            }
            if error < best { best = error; delta = d; }
        }
    }
    return vec4<f32>(delta, best, 1.0);
}
@fragment fn fs_interpolate(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(current));
    let motion = textureSampleLevel(flow, linear_sampler, p.xy / size, 0.0);
    let a = read_prev(p.xy + motion.xy * 0.5);
    let b = read_curr(p.xy - motion.xy * 0.5);
    // Disocclusions, cuts and high mismatch fall back to the current real pixel.
    let confidence = 1.0 - smoothstep(0.01, 0.08, max(motion.z, dot(a - b, a - b)));
    return vec4<f32>(mix(read_curr(p.xy), (a + b) * 0.5, confidence), 1.0);
}
