// Window glass in the rain: drops run down the pane and bend what is behind it (real refraction of
// the scene behind), streaks and a cold reflection. Keep "glass" in the file name so it draws after
// the opaque scene. object.user0.x = rain amount (default 1), y = speed (default 1).
#include "shaders/kvas_mesh_api.wgsl"   // editor help only: declares engine names so the editor shows no errors
@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

fn drops(p: vec2<f32>, t: f32, scale: f32, seed: f32) -> vec3<f32> {
    var q = p * scale;
    let col = floor(q.x);
    q.y += t * (0.2 + 0.4 * hash21(vec2<f32>(col, seed))) + hash21(vec2<f32>(col, seed + 2.0)) * 9.0;
    let cell = floor(q);
    let f = fract(q) - vec2<f32>(0.5);
    if (hash21(cell + vec2<f32>(seed, 1.3)) < 0.5) { return vec3<f32>(0.0); }
    let wob = sin(q.y * 5.0 + col) * 0.1;
    let g = f - vec2<f32>(wob, 0.0);
    let drop = smoothstep(0.27, 0.15, length(g * vec2<f32>(1.0, 1.35)));
    let trail = smoothstep(0.07, 0.0, abs(g.x)) * smoothstep(-0.5, 0.15, f.y) * 0.4;
    return vec3<f32>(g * drop, max(drop, trail));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let amount = select(object.user0.x, 1.0, object.user0.x == 0.0);
    let speed = select(object.user0.y, 1.0, object.user0.y == 0.0);
    let t = globals.time.x * speed;
    let p = vec2<f32>(dot(in.world_pos, normalize(object.model[0].xyz)), -in.world_pos.y);
    let a = drops(p, t, 9.0, 1.0);
    let b = drops(p, t * 0.7, 17.0, 5.0);
    let bend = (a.xy * 30.0 + b.xy * 18.0) * amount;
    let wet = max(a.z, b.z) * amount;
    let dims = vec2<i32>(textureDimensions(scene_color_tex));
    var behind = vec3<f32>(0.05);
    if (dims.x > 1) {
        let px = clamp(vec2<i32>(in.clip.xy + bend), vec2<i32>(0), dims - vec2<i32>(1));
        behind = textureLoad(scene_color_tex, px, 0).rgb;
    }
    let n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let fres = 0.04 + 0.96 * pow(1.0 - abs(dot(n, v)), 5.0);
    let refl = to_display_at(env_color(reflect(-v, n)), in.clip);
    var col = mix(behind * (0.92 + wet * 0.25), refl, fres * 0.6);
    col = mix(col, col * object.color.rgb, 0.3);
    let k = clamp(0.25 + wet * 0.75 + fres, 0.0, 1.0);
    // The pane bends and tints what is behind it, so it is drawn fully opaque.
    return vec4<f32>(mix(behind, col, k), 1.0);
}
