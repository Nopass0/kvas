// Built-in foliage shader for procedural trees (the prelude is prepended). Each card carries a
// small cluster of leaves (or needles) cut out here; the cards sway with the wind on the GPU.
//   object.color   main leaf colour          object.user0  x wind, y style (0 leaves, 1 needles, 2 fronds)
//   object.user1   xyz second leaf colour
//   uv.x           card corner (0..1) + card tint index (integer part), uv.y corner (0..1)

@vertex
fn vs_main(v: VertexIn) -> VsOut {
    var out = standard_vertex(v);
    let root = object.model[3].xyz;
    let up = max(out.world_pos.y - root.y, 0.0);
    let t = globals.time.x;
    let wind = object.user0.x;
    // Whole-crown sway grows with height; each leaf flutters on its own.
    let gust = 0.6 + 0.4 * sin(t * 0.7 + root.x * 0.11 + root.z * 0.07);
    let sway = sin(t * 1.25 + root.x * 0.3 + root.z * 0.2) * 0.018 * up * wind * gust;
    let phase = dot(out.world_pos, vec3<f32>(2.3, 1.7, 3.1)) + floor(v.uv.x) * 1.9;
    let flutter = sin(t * 6.5 + phase) * 0.035 * wind * gust;
    out.world_pos += vec3<f32>(sway + flutter, flutter * 0.6, sway * 0.55 - flutter * 0.4);
    out.clip = globals.view_proj * vec4<f32>(out.world_pos, 1.0);
    return out;
}

fn rot(p: vec2<f32>, a: f32) -> vec2<f32> {
    let s = sin(a);
    let c = cos(a);
    return vec2<f32>(c * p.x - s * p.y, s * p.x + c * p.y);
}

// Signed coverage of one leaf: positive inside. `q` is in leaf space, the leaf runs along +y from 0 to 1.
fn leaf(q: vec2<f32>) -> f32 {
    let y = q.y;
    if (y < 0.0 || y > 1.0) { return -1.0; }
    let width = 0.36 * pow(sin(y * 3.14159), 0.8) * (1.0 - 0.25 * y);
    return width - abs(q.x);
}

@fragment
fn fs_main(in: VsOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let card = floor(in.uv.x);
    let uv = vec2<f32>(fract(in.uv.x), in.uv.y);
    let p = uv * 2.0 - vec2<f32>(1.0);
    let style = object.user0.y;
    let seed = hash21(vec2<f32>(card, 3.7));

    // Coverage and a midrib mask.
    var inside = -1.0;
    var rib = 0.0;
    if (style > 0.5 && style < 1.5) {
        // Needles: thin strokes fanning out from a twig along the card.
        for (var i = 0; i < 9; i = i + 1) {
            let fi = f32(i);
            let base = vec2<f32>(0.0, -0.9 + fi * 0.2);
            let side = select(-1.0, 1.0, (i % 2) == 0);
            let q = rot(p - base, side * (0.9 + 0.2 * hash21(vec2<f32>(fi, card))));
            let d = 0.045 - abs(q.x);
            if (q.y > 0.0 && q.y < 0.75) { inside = max(inside, d); }
        }
        inside = max(inside, 0.05 - abs(p.x) * step(abs(p.y), 0.95));
    } else {
        // Three to five leaves around the card centre.
        let count = select(3, 5, seed > 0.5);
        for (var i = 0; i < count; i = i + 1) {
            let fi = f32(i);
            let a = fi / f32(count) * 6.2831853 + seed * 6.28;
            let q = rot(p * 1.15, a) / (0.95 + 0.2 * hash21(vec2<f32>(fi, card)));
            let c = leaf(vec2<f32>(q.x, q.y));
            if (c > inside) {
                inside = c;
                rib = 1.0 - smoothstep(0.0, 0.03, abs(q.x));
            }
        }
    }
    if (inside < 0.0) { discard; }

    // Colour: two tints mixed per card, darker towards the inside and the bottom of the crown.
    let tint = hash21(vec2<f32>(card, 1.3));
    var albedo = mix(object.color.rgb, object.user1.xyz, tint * tint);
    albedo *= 0.75 + 0.45 * hash21(vec2<f32>(card, 9.1));
    albedo *= 1.0 - rib * 0.25;
    let edge = smoothstep(0.0, 0.06, inside);
    albedo *= 0.8 + 0.2 * edge;

    var n = normalize(in.normal);
    let v = normalize(globals.camera_pos.xyz - in.world_pos);
    let l = normalize(globals.sun_dir.xyz);
    let shadow = shadow_factor(in.world_pos, n);
    // Soft wrapped light keeps the crown readable; light shining through when looking at the sun.
    let wrap = clamp(dot(n, l) * 0.5 + 0.5, 0.0, 1.0);
    let crown_ao = clamp(0.45 + 0.55 * (n.y * 0.5 + 0.5), 0.0, 1.0);
    var col = albedo * globals.sun_color.xyz * wrap * mix(0.35, 1.0, shadow);
    col += albedo * env_color(n) * 0.55 * crown_ao;
    let through = pow(max(dot(-v, l), 0.0), 4.0) * shadow;
    col += albedo * vec3<f32>(1.1, 1.25, 0.6) * globals.sun_color.xyz * through * 0.7;
    // A faint sheen on the leaf surface.
    let h = normalize(l + v);
    col += globals.sun_color.xyz * pow(max(dot(n, h), 0.0), 24.0) * 0.06 * shadow;
    if (!front) { col *= 0.92; }
    return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 1.0);
}
