"""Pixel-art shaders for tools/gen_shaders.py: full-screen effects and per-object looks."""

PALETTES = {
    "GameBoy": [(0.06, 0.22, 0.06), (0.19, 0.38, 0.19), (0.55, 0.67, 0.06), (0.61, 0.74, 0.06)],
    "GameBoyPocket": [(0.0, 0.0, 0.0), (0.33, 0.33, 0.33), (0.66, 0.66, 0.66), (1.0, 1.0, 1.0)],
    "Cga": [(0.0, 0.0, 0.0), (0.33, 1.0, 1.0), (1.0, 0.33, 1.0), (1.0, 1.0, 1.0)],
    "Sepia": [(0.1, 0.05, 0.02), (0.4, 0.25, 0.12), (0.75, 0.55, 0.3), (1.0, 0.9, 0.65)],
    "Neon": [(0.02, 0.0, 0.1), (0.5, 0.0, 0.8), (0.0, 0.8, 1.0), (1.0, 0.9, 1.0)],
    "Ice": [(0.02, 0.05, 0.15), (0.1, 0.3, 0.5), (0.5, 0.8, 0.95), (1.0, 1.0, 1.0)],
    "Autumn": [(0.1, 0.03, 0.02), (0.5, 0.15, 0.05), (0.9, 0.5, 0.1), (1.0, 0.9, 0.5)],
    "Candy": [(0.2, 0.05, 0.25), (0.9, 0.3, 0.6), (0.5, 0.8, 1.0), (1.0, 0.95, 0.9)],
}

# Interleaved gradient noise: a cheap, good-looking ordered dither.
IGN = "fn ign(p: vec2<f32>) -> f32 { return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715)))); }\n"

PICK = (
    "    let i = floor(l * 3.999);\n"
    "    var pal = P0;\n"
    "    if (i > 0.5) { pal = P1; }\n"
    "    if (i > 1.5) { pal = P2; }\n"
    "    if (i > 2.5) { pal = P3; }\n"
)


def palette_consts(vec, pal):
    return "".join(f"const P{i}: vec3<f32> = {vec(c)};\n" for i, c in enumerate(pal))


def register(mesh, fx, vec, LIT):
    for cells in (64, 96, 128, 160, 240, 320):
        for pname, pal in PALETTES.items():
            consts = f"const CELLS: f32 = {cells}.0;\nconst DITHER: f32 = 0.18;\n" + palette_consts(vec, pal) + IGN
            body = (
                "    let grid = vec2<f32>(CELLS, CELLS / screen_aspect());\n"
                "    let cell = floor(uv * grid);\n"
                "    let c = scene_color((cell + 0.5) / grid);\n"
                "    let l = clamp(lib_luma(c) + (ign(cell) - 0.5) * DITHER, 0.0, 1.0);\n"
                + PICK +
                "    return mix(color, pal, k);"
            )
            fx("pixel", f"Pixel{cells}{pname}", f"Pixel art at {cells} columns with a 4-colour {pname} palette and ordered dithering. Works on any 3D scene.", body, consts)

    for cells in (80, 120, 160, 240):
        for levels in (3, 4, 6):
            body = (
                "    let grid = vec2<f32>(CELLS, CELLS / screen_aspect());\n"
                "    let cell = floor(uv * grid);\n"
                "    let c = scene_color((cell + 0.5) / grid);\n"
                "    let d = (ign(cell) - 0.5) / LEVELS;\n"
                "    return mix(color, floor((c + vec3<f32>(d)) * LEVELS + 0.5) / LEVELS, k);"
            )
            fx("pixel", f"PixelColour{cells}x{levels}", f"Pixel art at {cells} columns keeping the scene colours: {levels} levels per channel and dithering.", body, f"const CELLS: f32 = {cells}.0;\nconst LEVELS: f32 = {levels}.0;\n" + IGN)

    for cells in (120, 200):
        body = (
            "    let grid = vec2<f32>(CELLS, CELLS / screen_aspect());\n"
            "    let cell = floor(uv * grid);\n"
            "    let q = (cell + 0.5) / grid;\n"
            "    let c = scene_color(q);\n"
            "    let o = 1.0 / grid;\n"
            "    let e = abs(lib_luma(scene_color(q + vec2<f32>(o.x, 0.0))) - lib_luma(scene_color(q - vec2<f32>(o.x, 0.0)))) + abs(lib_luma(scene_color(q + vec2<f32>(0.0, o.y))) - lib_luma(scene_color(q - vec2<f32>(0.0, o.y))));\n"
            "    return mix(color, mix(floor(c * 5.0 + 0.5) / 5.0, vec3<f32>(0.02), step(0.22, e)), k);"
        )
        fx("pixel", f"PixelOutline{cells}", f"Pixel art at {cells} columns with dark outlines where the picture changes sharply.", body, f"const CELLS: f32 = {cells}.0;\n")

    # A pixel look for single 3D objects: light is computed per world-space cell, banded and dithered.
    for res in (8, 16, 32):
        for pname, pal in list(PALETTES.items())[:6]:
            body = (
                "    let cell = floor(in.world_pos * RES);\n"
                "    let l = normalize(globals.sun_dir.xyz);\n"
                "    let shadow = shadow_factor((cell + 0.5) / RES, n);\n"
                "    let l = clamp(max(dot(n, l), 0.0) * shadow * 0.8 + 0.2 + (ign(floor(in.clip.xy)) - 0.5) * 0.15, 0.0, 1.0);\n"
                + PICK +
                "    let col = mix(pal, pal * object.color.rgb * 1.6, MIX);"
            ).replace("let l = normalize(globals.sun_dir.xyz);", "let sun = normalize(globals.sun_dir.xyz);").replace("dot(n, l)", "dot(n, sun)")
            consts = f"const RES: f32 = {res}.0;\nconst MIX: f32 = 0.5;\n" + palette_consts(vec, pal) + IGN
            mesh("pixel", f"VoxelLook{res}{pname}", f"Pixel-art look for a 3D object: light per {1.0 / res:.3f} m world cell, 4 bands of a {pname} palette, ordered dither.", body, consts)

    for tex in (16, 32, 64, 128):
        body = (
            f"{LIT}"
            "    let uv = floor(in.uv * TEX) / TEX + 0.5 / TEX;\n"
            "    let tex = textureSample(albedo_tex, albedo_smp, uv).rgb;\n"
            "    let col = floor(tex * lit * 5.0 + 0.5) / 5.0;"
        )
        mesh("pixel", f"PixelTexture{tex}", f"Shows the albedo texture as big crisp {tex}x{tex} pixels (nearest sampling) with banded lighting.", body, f"const TEX: f32 = {tex}.0;")

    for res in (6, 10, 16):
        body = (
            f"{LIT}"
            "    let k = fbm3(floor(in.world_pos * RES) / RES * 2.0, 3);\n"
            "    let col = floor(object.color.rgb * (0.6 + 0.8 * k) * (0.4 + 0.6 * lit) * 5.0 + 0.5) / 5.0;"
        )
        mesh("pixel", f"PixelNoise{res}", f"Chunky noise colouring on a {res}-per-metre grid: stone, grass or dirt in pixel-art games.", body, f"const RES: f32 = {res}.0;")
