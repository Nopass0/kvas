#!/usr/bin/env python3
"""Generates the starter shader library of the editor (crates/kvas_editor/library/shaders/<category>/*.wgsl).

Mesh shaders get a short header with tweakable `const`s; camera effects live in shaders/fx/<category>/.
The renderer test `shader_library_validates` compiles every file. Run from the repository root:
    python tools/gen_shaders.py
"""
import os
import shutil

LIB = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "kvas_editor", "library", "shaders")
OUT = {}

PALETTE = {
    "Red": (1.0, 0.12, 0.1), "Orange": (1.0, 0.5, 0.05), "Yellow": (1.0, 0.9, 0.1), "Lime": (0.55, 1.0, 0.1), "Green": (0.1, 0.9, 0.3),
    "Teal": (0.05, 0.8, 0.7), "Cyan": (0.1, 0.9, 1.0), "Blue": (0.15, 0.4, 1.0), "Violet": (0.5, 0.2, 1.0), "Magenta": (1.0, 0.1, 0.8),
    "Pink": (1.0, 0.5, 0.75), "White": (1.0, 1.0, 1.0),
}
PAIRS = {
    "Sunset": ((0.15, 0.05, 0.35), (1.0, 0.55, 0.2)), "Ocean": ((0.0, 0.1, 0.35), (0.2, 0.8, 0.9)), "Forest": ((0.03, 0.15, 0.05), (0.5, 0.85, 0.25)),
    "Fire": ((0.3, 0.0, 0.0), (1.0, 0.85, 0.2)), "Candy": ((1.0, 0.4, 0.7), (0.5, 0.8, 1.0)), "Toxic": ((0.1, 0.2, 0.0), (0.7, 1.0, 0.1)),
    "Ice": ((0.5, 0.75, 1.0), (0.95, 1.0, 1.0)), "Night": ((0.0, 0.0, 0.1), (0.4, 0.3, 0.9)), "Desert": ((0.65, 0.4, 0.2), (1.0, 0.85, 0.5)),
    "Lava": ((0.1, 0.0, 0.0), (1.0, 0.4, 0.0)), "Gold": ((0.5, 0.3, 0.05), (1.0, 0.85, 0.3)), "Mono": ((0.05, 0.05, 0.05), (0.95, 0.95, 0.95)),
}


def vec(c):
    return f"vec3<f32>({c[0]}, {c[1]}, {c[2]})"


def mesh(category, name, desc, body, consts="", vertex=None, ret=None, libs=""):
    """body computes `col` (vec3); ret overrides the default return statement; vertex overrides vs_main."""
    vs = vertex or "@vertex\nfn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }"
    ret = ret or "return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), object.color.a);"
    text = f"// {desc}\n// Edit the constants at the top to tune it. Object colour comes from the MeshRenderer.\n#include \"shaders/kvas_mesh_api.wgsl\"\n{libs}{consts}\n{vs}\n\n@fragment\nfn fs_main(in: VsOut) -> @location(0) vec4<f32> {{\n    let n = normalize(in.normal);\n    let v = normalize(globals.camera_pos.xyz - in.world_pos);\n    let t = globals.time.x;\n{body}\n    {ret}\n}}\n"
    assert f"{category}/{name}" not in OUT, name
    OUT[f"{category}/{name}.wgsl"] = text


def fx(category, name, desc, body, consts="", libs=""):
    text = f"// {desc}\n// params.x is the effect strength (0..1) unless noted. Edit the constants to tune it.\n#include \"shaders/kvas_camera_api.wgsl\"\n#include \"builtin:lib/color\"\n#include \"builtin:lib/noise\"\n{libs}{consts}\nfn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {{\n    let k = clamp(params.x, 0.0, 1.0);\n{body}\n}}\n"
    OUT[f"fx/{category}/{name}.wgsl"] = text


LIT = "    let l = normalize(globals.sun_dir.xyz);\n    let shadow = shadow_factor(in.world_pos, n);\n    let lit = object.color.rgb * (globals.ambient.rgb * 0.8 + globals.sun_color.rgb * max(dot(n, l), 0.0) * shadow);\n"

# ---------------------------------------------------------------- toon
for bands in (2, 3, 4, 5):
    for rim_name in ("White", "Cyan", "Orange"):
        rim = PALETTE[rim_name]
        mesh("toon", f"Toon{bands}Bands{rim_name}Rim", f"Cel shading with {bands} light bands and a {rim_name.lower()} rim.",
             f"    let l = normalize(globals.sun_dir.xyz);\n    let band = floor(max(dot(n, l), 0.0) * BANDS + 0.5) / BANDS;\n    let shadow = shadow_factor(in.world_pos, n);\n    let rim = smoothstep(0.55, 0.7, 1.0 - max(dot(n, v), 0.0));\n    let col = object.color.rgb * (globals.ambient.rgb * 0.8 + globals.sun_color.rgb * band * shadow) + RIM * rim * 0.6;",
             f"const BANDS: f32 = {bands}.0;\nconst RIM: vec3<f32> = {vec(rim)};")
for name, w in (("Thin", 0.8), ("Thick", 0.45), ("Heavy", 0.25), ("Ink", 0.1)):
    mesh("toon", f"Outline{name}", f"Inverted-hull style outline using the view angle: edges darker than {w}.",
         f"{LIT}    let edge = smoothstep({w}, {w + 0.1}, max(dot(n, v), 0.0));\n    let col = mix(vec3<f32>(0.02), floor(lit * 3.0 + 0.5) / 3.0, edge);")

# ---------------------------------------------------------------- rim / fresnel glow
for cname, c in PALETTE.items():
    mesh("glow", f"RimGlow{cname}", f"A {cname.lower()} glow on the silhouette; the base stays lit.",
         f"{LIT}    let rim = pow(1.0 - max(dot(n, v), 0.0), POWER);\n    let col = lit + COLOR * rim * STRENGTH;", f"const COLOR: vec3<f32> = {vec(c)};\nconst POWER: f32 = 3.0;\nconst STRENGTH: f32 = 2.0;")
for cname in ("Cyan", "Magenta", "Green", "Orange", "Violet", "White", "Red", "Blue"):
    c = PALETTE[cname]
    mesh("glow", f"Ghost{cname}", f"A see-through {cname.lower()} ghost: transparent in the middle, glowing at the edges.",
         "    let f = pow(1.0 - max(dot(n, v), 0.0), POWER);\n    let col = COLOR * (0.4 + 1.6 * f);",
         f"const COLOR: vec3<f32> = {vec(c)};\nconst POWER: f32 = 2.0;",
         ret="return vec4<f32>(to_display_at(col, in.clip), clamp(f * 1.2 + 0.1, 0.0, 1.0) * object.color.a);")
for cname in ("Cyan", "Green", "Orange", "Magenta", "Blue", "White", "Red", "Yellow"):
    c = PALETTE[cname]
    mesh("glow", f"Hologram{cname}", f"A {cname.lower()} hologram with scan lines and flicker.",
         "    let scan = 0.55 + 0.45 * sin(in.world_pos.y * 60.0 - t * 4.0);\n    let flicker = 0.9 + 0.1 * sin(t * 40.0);\n    let f = pow(1.0 - max(dot(n, v), 0.0), 2.0);\n    let col = COLOR * (0.5 + f * 1.5) * scan * flicker;",
         f"const COLOR: vec3<f32> = {vec(c)};",
         ret="return vec4<f32>(to_display_at(col, in.clip), (0.35 + 0.65 * f) * object.color.a);")
for cname in ("Red", "Orange", "Yellow", "Green", "Cyan", "Blue", "Violet", "Magenta"):
    c = PALETTE[cname]
    mesh("glow", f"Pulse{cname}", f"The object glows {cname.lower()} and pulses.",
         f"{LIT}    let pulse = 0.5 + 0.5 * sin(t * SPEED);\n    let col = lit * 0.4 + COLOR * (0.3 + pulse * 2.0);", f"const COLOR: vec3<f32> = {vec(c)};\nconst SPEED: f32 = 3.0;")
for cname in ("Cyan", "Magenta", "Orange", "Green", "Violet", "White"):
    c = PALETTE[cname]
    mesh("glow", f"ForceField{cname}", f"A {cname.lower()} energy shield: moving hex-like bands and a bright rim.",
         "    let f = pow(1.0 - max(dot(n, v), 0.0), 2.0);\n    let bands = 0.5 + 0.5 * sin(in.world_pos.y * 12.0 + in.world_pos.x * 6.0 - t * 2.0);\n    let col = COLOR * (0.3 + f * 2.5 + bands * 0.8);",
         f"const COLOR: vec3<f32> = {vec(c)};",
         ret="return vec4<f32>(to_display_at(col, in.clip), clamp(0.15 + f * 0.8 + bands * 0.2, 0.0, 1.0) * object.color.a);")
for i, (cname, c) in enumerate(list(PALETTE.items())[:6]):
    mesh("glow", f"Iridescent{cname}", f"Colour shifts with the viewing angle, starting from {cname.lower()} (soap bubble, beetle shell).",
         f"{LIT}    let h = fract(START + (1.0 - max(dot(n, v), 0.0)) * 1.3 + t * 0.05);\n    let rainbow = clamp(abs(fract(h + vec3<f32>(0.0, 0.666, 0.333)) * 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));\n    let col = mix(lit, rainbow, 0.65);", f"const START: f32 = {i * 0.15};")

# ---------------------------------------------------------------- gradients and patterns
for pname, (a, b) in PAIRS.items():
    mesh("pattern", f"HeightGradient{pname}", f"World-height gradient from {pname.lower()} bottom to top (lit).",
         f"{LIT}    let k = smoothstep(Y0, Y1, in.world_pos.y);\n    let col = mix(A, B, k) * (0.45 + 0.55 * lit);", f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};\nconst Y0: f32 = 0.0;\nconst Y1: f32 = 3.0;")
    mesh("pattern", f"RadialGradient{pname}", f"Gradient by the distance from the object centre: {pname.lower()}.",
         f"{LIT}    let d = clamp(length(in.world_pos - object.model[3].xyz) / RADIUS, 0.0, 1.0);\n    let col = mix(A, B, d) * (0.45 + 0.55 * lit);", f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};\nconst RADIUS: f32 = 1.5;")
for axis, comp in (("X", "x"), ("Y", "y"), ("Z", "z")):
    for sname, (a, b) in list(PAIRS.items())[:3]:
        mesh("pattern", f"Stripes{axis}{sname}", f"World-space stripes along {axis} in {sname.lower()} colours.",
             f"{LIT}    let s = step(0.5, fract(in.world_pos.{comp} * FREQ));\n    let col = mix(A, B, s) * (0.5 + 0.5 * lit);", f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};\nconst FREQ: f32 = 2.0;")
for sname, (a, b) in list(PAIRS.items())[:8]:
    mesh("pattern", f"Checker3D{sname}", f"A 3D checkerboard in {sname.lower()} colours.",
         f"{LIT}    let c = floor(in.world_pos * SCALE);\n    let s = abs(fract((c.x + c.y + c.z) * 0.5) * 2.0);\n    let col = mix(A, B, step(0.5, s)) * (0.5 + 0.5 * lit);", f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};\nconst SCALE: f32 = 2.0;")
for cname in ("Cyan", "Green", "Magenta", "Orange", "White", "Blue"):
    c = PALETTE[cname]
    mesh("pattern", f"Grid{cname}", f"A glowing {cname.lower()} world grid on a dark surface.",
         "    let g = abs(fract(in.world_pos * SCALE - 0.5) - 0.5) / fwidth(in.world_pos * SCALE);\n    let line = 1.0 - min(min(g.x, g.y), g.z);\n    let col = vec3<f32>(0.02) + COLOR * clamp(line, 0.0, 1.0) * 2.0;", f"const COLOR: vec3<f32> = {vec(c)};\nconst SCALE: f32 = 1.0;")
for sname, (a, b) in list(PAIRS.items())[:6]:
    mesh("pattern", f"PolkaDots{sname}", f"Polka dots on the surface UV in {sname.lower()} colours.",
         f"{LIT}    let cell = fract(in.uv * COUNT) - 0.5;\n    let dot_mask = 1.0 - smoothstep(RADIUS, RADIUS + 0.03, length(cell));\n    let col = mix(A, B, dot_mask) * (0.5 + 0.5 * lit);", f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};\nconst COUNT: f32 = 8.0;\nconst RADIUS: f32 = 0.28;")

# ---------------------------------------------------------------- noise and animated
for sname, (a, b) in PAIRS.items():
    mesh("noise", f"Noise{sname}", f"Fractal noise colouring from {sname.lower()} dark to light, lit.",
         f"{LIT}    let k = fbm3(in.world_pos * SCALE, 4);\n    let col = mix(A, B, smoothstep(0.25, 0.75, k)) * (0.4 + 0.6 * lit);", f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};\nconst SCALE: f32 = 2.5;")
    mesh("noise", f"Flow{sname}", f"Animated flowing noise in {sname.lower()} colours (plasma, lava lamp, magic).",
         f"    let p = in.world_pos * SCALE + vec3<f32>(0.0, t * SPEED, t * SPEED * 0.4);\n    let k = fbm3(p + fbm3(p * 1.7, 3) * 1.5, 4);\n    let col = mix(A, B, smoothstep(0.2, 0.8, k)) * GLOW;",
         f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};\nconst SCALE: f32 = 1.5;\nconst SPEED: f32 = 0.4;\nconst GLOW: f32 = 1.3;")

for cname in ("Orange", "Cyan", "Green", "Violet", "Red", "White"):
    c = PALETTE[cname]
    mesh("fx3d", f"Dissolve{cname}", f"Dissolves with noise and a {cname.lower()} burning edge. user 1 = amount 0..1 (animates by itself when 0).",
         f"{LIT}    let amount = select(object.user0.x, 0.5 + 0.5 * sin(t * 0.7), object.user0.x == 0.0);\n    let k = fbm3(in.world_pos * 3.0, 4);\n    if (k < amount) {{ discard; }}\n    let edge = 1.0 - smoothstep(0.0, 0.06, k - amount);\n    let col = lit + EDGE * edge * 4.0;", f"const EDGE: vec3<f32> = {vec(c)};")
for cname in ("Orange", "Red", "Blue", "Green", "Violet", "Cyan"):
    c = PALETTE[cname]
    mesh("fx3d", f"Flame{cname}", f"A flickering {cname.lower()} flame colour, hotter towards the middle.",
         f"    let p = in.world_pos * 2.0 + vec3<f32>(0.0, -t * 1.6, 0.0);\n    let k = fbm3(p, 4);\n    let heat = clamp(k * 1.6 - (in.world_pos.y - object.model[3].y) * 0.3, 0.0, 1.0);\n    let col = mix(COLOR * 0.4, vec3<f32>(1.0, 0.95, 0.7), heat * heat) * (1.0 + heat * 2.0);", f"const COLOR: vec3<f32> = {vec(c)};")
for cname in ("White", "Green", "Pink", "Blue", "Orange", "Violet"):
    c = PALETTE[cname]
    mesh("fx3d", f"Cover{cname}", f"A {cname.lower()} layer (snow, moss, dust, paint) settles on upward-facing surfaces. Cover amount via the constants.",
         f"{LIT}    let up = smoothstep(THRESHOLD, THRESHOLD + 0.2, n.y + (fbm3(in.world_pos * 4.0, 3) - 0.5) * 0.4);\n    let col = mix(lit, COVER * (0.55 + 0.45 * max(dot(n, l), 0.0)), up);", f"const COVER: vec3<f32> = {vec(c)};\nconst THRESHOLD: f32 = 0.35;")
for cname in ("White", "Yellow", "Cyan", "Pink"):
    c = PALETTE[cname]
    mesh("fx3d", f"Glitter{cname}", f"Sparkling {cname.lower()} glitter that twinkles with the camera angle.",
         f"{LIT}    let cell = floor(in.world_pos * 40.0);\n    let r = hash31(cell);\n    let sparkle = pow(max(sin(t * 3.0 + r * 40.0 + dot(v, n) * 20.0), 0.0), 20.0) * step(0.8, r);\n    let col = lit + COLOR * sparkle * 4.0;", f"const COLOR: vec3<f32> = {vec(c)};")
for name, (a, b) in list(PAIRS.items())[:6]:
    mesh("fx3d", f"MatCap{name}", f"A fake lit sphere look (MatCap) in {name.lower()} colours: shading depends only on the view angle.",
         "    let vn = normalize((globals.view_proj * vec4<f32>(n, 0.0)).xyz);\n    let k = clamp(vn.y * 0.5 + 0.5, 0.0, 1.0);\n    let col = mix(A, B, k * k) * (0.6 + 0.4 * max(dot(n, v), 0.0));", f"const A: vec3<f32> = {vec(a)};\nconst B: vec3<f32> = {vec(b)};")

# ---------------------------------------------------------------- vertex animation
VERTEX = {
    "Wave": ("A travelling wave pushes vertices up and down.", "    o.world_pos.y += sin(o.world_pos.x * FREQ + globals.time.x * SPEED) * cos(o.world_pos.z * FREQ * 0.8 + globals.time.x * SPEED * 0.7) * AMP;"),
    "Wind": ("Vertices sway in the wind, more towards the top.", "    let h = max(o.world_pos.y - object.model[3].y, 0.0);\n    o.world_pos.x += sin(globals.time.x * SPEED + o.world_pos.x * FREQ) * AMP * h;\n    o.world_pos.z += cos(globals.time.x * SPEED * 0.8 + o.world_pos.z * FREQ) * AMP * h;"),
    "Ripple": ("Circular ripples spread from the object centre.", "    let d = length(o.world_pos.xz - object.model[3].xz);\n    o.world_pos.y += sin(d * FREQ - globals.time.x * SPEED) * AMP / (1.0 + d);"),
    "Breathe": ("The shape swells and shrinks along its normals.", "    o.world_pos += o.normal * sin(globals.time.x * SPEED) * AMP;"),
    "Twist": ("Vertices twist around the vertical axis over time.", "    let c = object.model[3].xyz;\n    let a = (o.world_pos.y - c.y) * FREQ * AMP + globals.time.x * SPEED * 0.2;\n    let p = o.world_pos - c;\n    o.world_pos = c + vec3<f32>(p.x * cos(a) - p.z * sin(a), p.y, p.x * sin(a) + p.z * cos(a));"),
    "Jitter": ("Vertices jitter like a cheap old console.", "    o.world_pos += vec3<f32>(hash31(floor(o.world_pos * 20.0) + floor(globals.time.x * 12.0)), 0.0, hash31(floor(o.world_pos * 20.0) + 7.0 + floor(globals.time.x * 12.0))) * AMP - vec3<f32>(AMP * 0.5, 0.0, AMP * 0.5);"),
    "Flag": ("A flag or cloth wave that is pinned on the left edge (x = object centre).", "    let pin = max(o.world_pos.x - object.model[3].x, 0.0);\n    o.world_pos.y += sin(o.world_pos.x * FREQ - globals.time.x * SPEED) * AMP * pin;\n    o.world_pos.z += cos(o.world_pos.x * FREQ * 0.7 - globals.time.x * SPEED) * AMP * 0.5 * pin;"),
    "Bounce": ("The whole object squashes and stretches like a ball.", "    let c = object.model[3].xyz;\n    let k = 1.0 + 0.2 * sin(globals.time.x * SPEED);\n    o.world_pos = c + (o.world_pos - c) * vec3<f32>(1.0 / sqrt(k), k, 1.0 / sqrt(k));"),
}
for name, (desc, code) in VERTEX.items():
    for lname, amp, speed in (("Soft", 0.08, 1.5), ("Strong", 0.3, 3.0)):
        vs = f"@vertex\nfn vs_main(v: VertexIn) -> VsOut {{\n    var o = standard_vertex(v);\n{code}\n    o.clip = globals.view_proj * vec4<f32>(o.world_pos, 1.0);\n    return o;\n}}"
        mesh("vertex", f"{name}{lname}", desc, f"{LIT}    let col = lit;", f"const AMP: f32 = {amp};\nconst SPEED: f32 = {speed};\nconst FREQ: f32 = 2.0;", vertex=vs)

# ---------------------------------------------------------------- water
WATER = {
    "Ocean": ((0.0, 0.12, 0.3), (0.1, 0.5, 0.65), 0.04, 1.0),
    "Lake": ((0.02, 0.18, 0.25), (0.15, 0.55, 0.55), 0.02, 0.7),
    "Pool": ((0.1, 0.55, 0.8), (0.5, 0.95, 1.0), 0.015, 1.4),
    "Tropical": ((0.0, 0.45, 0.55), (0.3, 1.0, 0.85), 0.03, 1.1),
    "Swamp": ((0.05, 0.12, 0.04), (0.25, 0.4, 0.15), 0.01, 0.3),
    "River": ((0.05, 0.2, 0.25), (0.35, 0.6, 0.6), 0.035, 2.0),
    "Toxic": ((0.1, 0.3, 0.0), (0.6, 1.0, 0.1), 0.03, 0.9),
    "Lava": ((0.35, 0.03, 0.0), (1.0, 0.55, 0.05), 0.02, 0.3),
    "Ink": ((0.0, 0.0, 0.03), (0.1, 0.1, 0.25), 0.03, 0.6),
    "Milk": ((0.85, 0.85, 0.8), (1.0, 1.0, 0.95), 0.01, 0.4),
}
for name, (deep, shallow, amp, speed) in WATER.items():
    mesh("water", f"Water{name}", f"Realistic animated {name.lower()} water: moving wave normals, depth tint, Fresnel sky reflection and a sun glint.",
         "    let p = in.world_pos.xz * SCALE;\n    let w1 = vec2<f32>(sin(p.x * 1.3 + t * SPEED) + sin(p.y * 0.9 - t * SPEED * 0.8), cos(p.y * 1.1 + t * SPEED * 0.9) + cos(p.x * 0.7 - t * SPEED));\n    let w2 = vec2<f32>(sin(p.x * 3.1 - t * SPEED * 1.7), cos(p.y * 2.7 + t * SPEED * 1.4)) * 0.4;\n    let nn = normalize(n + vec3<f32>(w1.x + w2.x, 0.0, w1.y + w2.y) * AMP * 8.0);\n    let f = pow(1.0 - max(dot(nn, v), 0.0), 4.0);\n    let l = normalize(globals.sun_dir.xyz);\n    let spec = pow(max(dot(reflect(-l, nn), v), 0.0), 120.0);\n    let body = mix(SHALLOW, DEEP, clamp(length(in.world_pos - globals.camera_pos.xyz) / 40.0, 0.0, 1.0));\n    let col = mix(body, env_color(reflect(-v, nn)), f * 0.8) + globals.sun_color.rgb * spec;",
         f"const DEEP: vec3<f32> = {vec(deep)};\nconst SHALLOW: vec3<f32> = {vec(shallow)};\nconst AMP: f32 = {amp};\nconst SPEED: f32 = {speed};\nconst SCALE: f32 = 1.0;",
         ret="return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), clamp(0.55 + f * 0.4, 0.0, 1.0) * object.color.a);")
for name, (deep, shallow, amp, speed) in WATER.items():
    mesh("water", f"ToonWater{name}", f"Stylized {name.lower()} water: flat colour bands, drifting foam lines and a hard highlight.",
         "    let p = in.world_pos.xz * SCALE;\n    let wave = sin(p.x * 2.0 + t * SPEED) * 0.5 + sin(p.y * 2.6 - t * SPEED * 0.7) * 0.5 + fbm3(vec3<f32>(p * 0.8, t * 0.2), 3);\n    let band = floor(clamp(wave * 0.4 + 0.5, 0.0, 1.0) * 3.0) / 3.0;\n    let foam = step(0.92, fract(wave * 2.0 + t * 0.1)) * 0.5;\n    let col = mix(DEEP, SHALLOW, band) + foam + vec3<f32>(step(0.985, fbm3(vec3<f32>(p * 3.0, t * 0.5), 2)));",
         f"const DEEP: vec3<f32> = {vec(deep)};\nconst SHALLOW: vec3<f32> = {vec(shallow)};\nconst SPEED: f32 = {speed};\nconst SCALE: f32 = 1.0;")
for name, (deep, shallow, amp, speed) in list(WATER.items())[:6]:
    vs = "@vertex\nfn vs_main(v: VertexIn) -> VsOut {\n    var o = standard_vertex(v);\n    o.world_pos.y += (sin(o.world_pos.x * 1.2 + globals.time.x * SPEED) + cos(o.world_pos.z * 0.9 + globals.time.x * SPEED * 0.8)) * AMP;\n    o.clip = globals.view_proj * vec4<f32>(o.world_pos, 1.0);\n    return o;\n}"
    mesh("water", f"WavySurface{name}", f"A {name.lower()} water surface whose vertices move with the waves (use a subdivided plane).",
         "    let f = pow(1.0 - max(dot(n, v), 0.0), 3.0);\n    let col = mix(mix(SHALLOW, DEEP, 0.4), env_color(reflect(-v, n)), f);",
         f"const DEEP: vec3<f32> = {vec(deep)};\nconst SHALLOW: vec3<f32> = {vec(shallow)};\nconst AMP: f32 = {amp * 4};\nconst SPEED: f32 = {speed};", vertex=vs,
         ret="return vec4<f32>(to_display_at(apply_fog(col, in.world_pos), in.clip), 0.8 * object.color.a);")

# ---------------------------------------------------------------- uv scroll with the albedo texture
for name, (dx, dy) in {"ScrollRight": (0.2, 0.0), "ScrollLeft": (-0.2, 0.0), "ScrollUp": (0.0, 0.2), "ScrollDown": (0.0, -0.2), "Diagonal": (0.15, 0.15), "Fast": (1.0, 0.0)}.items():
    mesh("texture", f"Conveyor{name}", "Scrolls the albedo texture over time (conveyor belts, flowing water, moving sky).",
         f"{LIT}    let uv = in.uv + vec2<f32>({dx}, {dy}) * t;\n    let tex = textureSample(albedo_tex, albedo_smp, uv).rgb;\n    let col = tex * lit;")

# ================================================================ camera effects
for cname, c in PALETTE.items():
    fx("grade", f"Tint{cname}", f"Tints the picture {cname.lower()} (multiply).", f"    return mix(color, color * TINT, k);", f"const TINT: vec3<f32> = {vec(tuple(0.4 + 0.6 * x for x in c))};")
GRADES = {
    "Warm": "color * vec3<f32>(1.1, 1.0, 0.88)", "Cool": "color * vec3<f32>(0.88, 1.0, 1.12)", "Matrix": "vec3<f32>(0.0, 1.0, 0.2) * lib_luma(color) * 1.4",
    "Noir": "vec3<f32>(lib_contrast(vec3<f32>(lib_luma(color)), 1.5))", "Fade": "mix(color, vec3<f32>(lib_luma(color)), 0.5) * 0.9 + vec3<f32>(0.08)",
    "Vivid": "lib_saturation(lib_contrast(color, 1.15), 1.5)", "Bleach": "mix(color, vec3<f32>(lib_luma(color)), 0.6) * 1.15", "Cross": "vec3<f32>(color.r * 1.1, color.g * 0.95, color.b * 1.15 + 0.03)",
    "Golden": "color * vec3<f32>(1.15, 1.02, 0.8) + vec3<f32>(0.02, 0.01, 0.0)", "Moon": "vec3<f32>(lib_luma(color)) * vec3<f32>(0.7, 0.85, 1.2)", "Toxic": "vec3<f32>(color.r * 0.8, color.g * 1.2, color.b * 0.7)",
    "Dusk": "color * vec3<f32>(1.1, 0.85, 0.95) + vec3<f32>(0.03, 0.0, 0.05)",
}
for name, expr in GRADES.items():
    fx("grade", f"Grade{name}", f"The {name.lower()} colour grade.", f"    return mix(color, {expr}, k);")
for s in (0.2, 0.35, 0.5, 0.7, 0.9, 1.2):
    fx("vignette", f"Vignette{int(s * 100)}", f"A vignette of strength {s}.", f"    return color * lib_vignette(uv, {s} * k * 2.0);")
for steps in (2, 3, 4, 6, 8, 12):
    fx("retro", f"Posterize{steps}", f"Reduces the colours to {steps} levels per channel.", f"    return mix(color, floor(color * {steps}.0 + 0.5) / {steps}.0, k);")
for cells in (60, 90, 120, 180, 240):
    fx("retro", f"Pixelate{cells}", f"Blocky pixels, {cells} columns.", f"    let p = floor(uv * {cells}.0) / {cells}.0;\n    return mix(color, scene_color(p), k);")
for lines, strength in ((120, 0.2), (200, 0.3), (300, 0.4), (480, 0.25)):
    fx("retro", f"Scanlines{lines}", f"CRT scan lines ({lines} lines).", f"    let s = 1.0 - {strength} * (0.5 + 0.5 * sin(uv.y * {lines}.0 * 3.14159));\n    return mix(color, color * s, k);")
for px in (1.0, 2.0, 4.0, 8.0):
    fx("lens", f"Chromatic{int(px)}", f"Splits red and blue by {px} pixels.", f"    let o = vec2<f32>({px}, 0.0) * pixel_size();\n    let c = vec3<f32>(scene_color(uv + o).r, color.g, scene_color(uv - o).b);\n    return mix(color, c, k);")
for amp, freq in ((0.003, 20.0), (0.006, 12.0), (0.012, 8.0), (0.02, 5.0)):
    fx("lens", f"WaveDistort{int(freq)}", "Wobbles the picture like a heat haze or a dream.", f"    let o = vec2<f32>(sin(uv.y * {freq} + time * 2.0), cos(uv.x * {freq} + time * 2.0)) * {amp};\n    return mix(color, scene_color(uv + o * k), 1.0);")
for name, expr in {"Invert": "vec3<f32>(1.0) - color", "Solarize": "mix(color, vec3<f32>(1.0) - color, step(0.5, lib_luma(color)))", "Threshold": "vec3<f32>(step(0.5, lib_luma(color)))", "Sepia": "vec3<f32>(lib_luma(color) * 1.2, lib_luma(color), lib_luma(color) * 0.8)", "Duotone": "mix(vec3<f32>(0.05, 0.0, 0.2), vec3<f32>(1.0, 0.8, 0.3), lib_luma(color))", "Heat": "mix(vec3<f32>(0.0, 0.0, 0.3), vec3<f32>(1.0, 0.3, 0.0), lib_luma(color) * 1.5)"}.items():
    fx("stylize", name, f"{name} look.", f"    return mix(color, {expr}, k);")
for contrast in (0.7, 1.2, 1.5, 2.0):
    fx("grade", f"Contrast{int(contrast * 10)}", f"Contrast x{contrast}.", f"    return mix(color, lib_contrast(color, {contrast}), k);")
for sat in (0.0, 0.4, 1.4, 2.0):
    fx("grade", f"Saturation{int(sat * 10)}", f"Saturation x{sat}.", f"    return mix(color, lib_saturation(color, {sat}), k);")
for name, hue in (("Shift60", 0.17), ("Shift120", 0.33), ("Shift180", 0.5), ("Shift240", 0.67)):
    fx("grade", f"Hue{name}", f"Rotates all hues by {name[5:]} degrees.", f"    return mix(color, lib_hue_shift(color, {hue}), k);")
for amt in (0.04, 0.08, 0.15):
    fx("lens", f"Grain{int(amt * 100)}", f"Film grain of strength {amt}.", f"    let g = (lib_hash21(uv * 1000.0 + vec2<f32>(time, 0.0)) - 0.5) * {amt};\n    return color + vec3<f32>(g) * k;")
fx("lens", "Sharpen", "Sharpens the picture.", "    let o = pixel_size();\n    let blur = (scene_color(uv + vec2<f32>(o.x, 0.0)) + scene_color(uv - vec2<f32>(o.x, 0.0)) + scene_color(uv + vec2<f32>(0.0, o.y)) + scene_color(uv - vec2<f32>(0.0, o.y))) * 0.25;\n    return color + (color - blur) * 1.5 * k;")
fx("lens", "SoftBlur", "A light 4-tap blur.", "    let o = pixel_size() * 1.5;\n    let b = (scene_color(uv + vec2<f32>(o.x, 0.0)) + scene_color(uv - vec2<f32>(o.x, 0.0)) + scene_color(uv + vec2<f32>(0.0, o.y)) + scene_color(uv - vec2<f32>(0.0, o.y)) + color) * 0.2;\n    return mix(color, b, k);")


import gen_shaders_pixel

gen_shaders_pixel.register(mesh, fx, vec, LIT)

def main():
    for sub in os.listdir(LIB):
        path = os.path.join(LIB, sub)
        # Only generated category folders are rewritten; hand-written files in the root and in `fx/` stay.
        if sub in {os.path.dirname(rel).split("/")[0] for rel in OUT} and os.path.isdir(path) and sub != "fx":
            shutil.rmtree(path)
    fx_gen = os.path.join(LIB, "fx")
    if os.path.isdir(fx_gen):
        for sub in os.listdir(fx_gen):
            if os.path.isdir(os.path.join(fx_gen, sub)):
                shutil.rmtree(os.path.join(fx_gen, sub))
    for rel, text in OUT.items():
        path = os.path.join(LIB, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
    print(f"{sum(1 for r in OUT if not r.startswith('fx/'))} mesh shaders, {sum(1 for r in OUT if r.startswith('fx/'))} camera effects")


main()
