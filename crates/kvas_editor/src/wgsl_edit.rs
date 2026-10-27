//! WGSL code editing: syntax colours, comments, and an API reference with autocomplete hints.

use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};

/// `(name, signature, explanation)` of everything a Kvas shader can use.
pub const API: &[(&str, &str, &str)] = &[
    ("vs_main", "fn vs_main(v: VertexIn) -> VsOut", "Vertex shader. Usually just: return standard_vertex(v);"),
    ("fs_main", "fn fs_main(in: VsOut) -> @location(0) vec4<f32>", "Fragment shader: returns the final colour (rgb) and opacity (a) of each pixel."),
    ("effect", "fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32>", "Camera shader entry: uv 0..1 on screen, color = current pixel, params = 4 numbers from the Inspector. Return the new colour."),
    ("standard_vertex", "fn standard_vertex(v: VertexIn) -> VsOut", "Default vertex transform: model + camera matrices. Modify its result for wind, waves, etc."),
    ("globals", "var<uniform> globals: Globals", "Per-frame data: time.x (seconds), camera_pos, sun_dir, sun_color, ambient, sky."),
    ("object", "var<uniform> object: Object", "Per-object data: color, params (roughness, emissive, uv tiling), user0/user1 (8 free numbers from the Inspector)."),
    ("in.world_pos", "in.world_pos: vec3<f32>", "Position of this pixel in the world."),
    ("in.normal", "in.normal: vec3<f32>", "Surface direction (normalize it before use)."),
    ("in.uv", "in.uv: vec2<f32>", "Texture coordinates of this pixel."),
    ("in.clip", "in.clip: vec4<f32>", "Screen position; pass it to to_display_at."),
    ("lighting", "fn lighting(p, n, base: vec3, roughness: f32) -> vec3<f32>", "Simple lighting: sun, shadows, lamps and sky for a surface point."),
    ("lighting_pbr", "fn lighting_pbr(p, n, albedo: vec3, rough: f32, metal: f32, ao: f32) -> vec3<f32>", "Physically based lighting; use for metals and realistic materials."),
    ("shadow_factor", "fn shadow_factor(p: vec3, n: vec3) -> f32", "Sun shadow at a point: 1 = lit, 0 = fully shadowed."),
    ("env_color", "fn env_color(dir: vec3<f32>) -> vec3<f32>", "Sky/environment colour in a direction. Use with reflect() for reflections."),
    ("apply_fog", "fn apply_fog(col: vec3, world: vec3) -> vec3<f32>", "Mixes in the scene fog based on distance."),
    ("to_display_at", "fn to_display_at(linear: vec3, clip: vec4) -> vec3<f32>", "Final step: tone mapping, grading and dithering. Always return through this."),
    ("to_display", "fn to_display(linear: vec3<f32>) -> vec3<f32>", "Like to_display_at without screen-position dithering."),
    ("hash21", "fn hash21(p: vec2<f32>) -> f32", "Random number 0..1 from a 2D position."),
    ("hash31", "fn hash31(p: vec3<f32>) -> f32", "Random number 0..1 from a 3D position."),
    ("value_noise", "fn value_noise(p: vec2<f32>) -> f32", "Smooth 2D noise 0..1."),
    ("fbm2", "fn fbm2(p: vec2<f32>, octaves: i32) -> f32", "Layered 2D noise (clouds, terrain, dirt). More octaves = more detail."),
    ("fbm3", "fn fbm3(p: vec3<f32>, octaves: i32) -> f32", "Layered 3D noise."),
    ("scene_color", "fn scene_color(uv: vec2<f32>) -> vec3<f32>", "Camera shaders: colour of any pixel of the image (blur, distortion)."),
    ("pixel_size", "fn pixel_size() -> vec2<f32>", "Camera shaders: size of one pixel in uv units."),
    ("screen_aspect", "fn screen_aspect() -> f32", "Camera shaders: width / height."),
    ("lib_luma", "fn lib_luma(c: vec3<f32>) -> f32", "Brightness of a colour. #include \"builtin:lib/color\""),
    ("lib_hue_shift", "fn lib_hue_shift(c: vec3<f32>, turns: f32) -> vec3<f32>", "Rotates hue. #include \"builtin:lib/color\""),
    ("lib_saturation", "fn lib_saturation(c: vec3<f32>, amount: f32) -> vec3<f32>", "0 = grey, 1 = unchanged, >1 = vivid."),
    ("lib_contrast", "fn lib_contrast(c: vec3<f32>, amount: f32) -> vec3<f32>", "Contrast around mid grey."),
    ("lib_vignette", "fn lib_vignette(uv: vec2<f32>, strength: f32) -> f32", "Darkening factor towards the screen edges."),
    ("lib_noise", "fn lib_noise(p: vec2<f32>) -> f32", "Smooth noise. #include \"builtin:lib/noise\""),
    ("lib_fbm", "fn lib_fbm(p: vec2<f32>) -> f32", "Layered noise."),
    ("lib_voronoi", "fn lib_voronoi(p: vec2<f32>) -> vec2<f32>", "Cell noise: x = distance to nearest point, y = cell id."),
    ("lib_remap", "fn lib_remap(v, a, b, c, d: f32) -> f32", "Maps v from range a..b to c..d. #include \"builtin:lib/math\""),
    ("lib_pulse", "fn lib_pulse(t: f32, hz: f32) -> f32", "Smooth 0..1 oscillation."),
    ("lib_rot2", "fn lib_rot2(p: vec2<f32>, a: f32) -> vec2<f32>", "Rotates a 2D vector by an angle in radians."),
    ("mix", "mix(a, b, t)", "Blend: a when t = 0, b when t = 1."),
    ("smoothstep", "smoothstep(edge0, edge1, x)", "Smooth 0..1 ramp between two values."),
    ("clamp", "clamp(x, lo, hi)", "Limits x to lo..hi."),
    ("normalize", "normalize(v)", "Makes a vector length 1."),
    ("dot", "dot(a, b)", "Dot product; for unit vectors it is the cosine of the angle."),
    ("reflect", "reflect(incident, normal)", "Mirror direction, for reflections."),
    ("pow", "pow(x, e)", "x to the power e."),
    ("fract", "fract(x)", "Fractional part (repeats 0..1)."),
    ("floor", "floor(x)", "Rounds down."),
    ("select", "select(if_false, if_true, condition)", "Chooses a value without branching. Handy for parameter defaults."),
    ("textureSample", "textureSample(tex, sampler, uv)", "Reads a texture, e.g. albedo_tex with albedo_smp."),
    ("#include", "#include \"path.wgsl\"", "Pulls functions from another file (once). builtin:lib/color, noise, math are provided by Kvas."),
];

const KEYWORDS: &[&str] = &[
    "fn", "let", "var", "const", "struct", "return", "if", "else", "for", "while", "loop", "break", "continue", "switch", "case", "default", "discard", "true", "false", "override", "alias", "continuing",
];
const TYPES: &[&str] = &[
    "f32", "i32", "u32", "bool", "f16", "vec2", "vec3", "vec4", "mat2x2", "mat3x3", "mat4x4", "array", "sampler", "sampler_comparison", "texture_2d", "texture_3d", "texture_depth_2d", "ptr", "atomic",
];

fn colors(dark: bool) -> [Color32; 7] {
    if dark {
        [
            Color32::from_rgb(0xCC, 0x78, 0x32), // keyword
            Color32::from_rgb(0x56, 0xA8, 0xF5), // type
            Color32::from_rgb(0x6A, 0x9F, 0x59), // comment
            Color32::from_rgb(0x68, 0x97, 0xBB), // number
            Color32::from_rgb(0xFF, 0xC6, 0x6D), // Kvas API
            Color32::from_rgb(0xBB, 0xB5, 0x29), // attribute / directive
            Color32::from_rgb(0xBC, 0xBE, 0xC4), // plain
        ]
    } else {
        [
            Color32::from_rgb(0x0A, 0x30, 0x9E),
            Color32::from_rgb(0x00, 0x62, 0x8C),
            Color32::from_rgb(0x4F, 0x7F, 0x3A),
            Color32::from_rgb(0x1A, 0x1A, 0xE0),
            Color32::from_rgb(0x9A, 0x5B, 0x00),
            Color32::from_rgb(0x80, 0x80, 0x00),
            Color32::from_rgb(0x20, 0x20, 0x20),
        ]
    }
}

/// Language of a code box, chosen from the file extension.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Wgsl,
    Rust,
    Plain,
}

impl Lang {
    pub fn of(path: &str) -> Lang {
        match path.rsplit('.').next().unwrap_or("") {
            "wgsl" => Lang::Wgsl,
            "rs" => Lang::Rust,
            _ => Lang::Plain,
        }
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use", "mod", "const", "static", "return", "if", "else", "for", "while", "loop", "break", "continue", "match", "as", "in", "where", "self", "Self", "super", "crate", "true", "false", "move", "ref", "type", "unsafe", "dyn", "async", "await",
];
const RUST_TYPES: &[&str] = &["f32", "f64", "i32", "i64", "u32", "u64", "u8", "u16", "i8", "i16", "usize", "isize", "bool", "str", "String", "Vec", "Option", "Result", "Box"];

/// Functions of `Ctx` read from the engine source, so hints always match the real API.
pub fn rust_api() -> &'static Vec<(String, String, String)> {
    static API_RS: std::sync::OnceLock<Vec<(String, String, String)>> = std::sync::OnceLock::new();
    API_RS.get_or_init(|| {
        let src = include_str!("../../kvas_engine/src/script.rs");
        let mut out = Vec::new();
        let mut doc: Vec<String> = Vec::new();
        for line in src.lines() {
            let t = line.trim();
            if let Some(d) = t.strip_prefix("///") {
                doc.push(d.trim().to_string());
            } else if let Some(rest) = t.strip_prefix("pub fn ") {
                let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                let sig = format!("ctx.{}", t.trim_start_matches("pub fn ").trim_end_matches('{').trim());
                if !name.is_empty() && line.starts_with("    pub fn") {
                    out.push((name, sig, doc.join(" ")));
                }
                doc.clear();
            } else if !t.starts_with('#') {
                doc.clear();
            }
        }
        out
    })
}

/// Coloured text for the code box.
pub fn highlight(text: &str, dark: bool, font: FontId, wrap: f32) -> LayoutJob {
    highlight_lang(text, dark, font, wrap, Lang::Wgsl)
}

/// Coloured text for the code box in the given language.
pub fn highlight_lang(text: &str, dark: bool, font: FontId, wrap: f32, lang: Lang) -> LayoutJob {
    let c = colors(dark);
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap;
    let push = |job: &mut LayoutJob, s: &str, col: Color32| {
        if !s.is_empty() {
            job.append(s, 0.0, TextFormat { font_id: font.clone(), color: col, ..Default::default() });
        }
    };
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let ch = b[i] as char;
        if lang == Lang::Plain {
            let end = text.len();
            push(&mut job, &text[i..end], c[6]);
            i = end;
        } else if lang == Lang::Rust && text[i..].starts_with("/*") {
            let end = text[i..].find("*/").map_or(text.len(), |n| i + n + 2);
            push(&mut job, &text[i..end], c[2]);
            i = end;
        } else if text[i..].starts_with("//") {
            let end = text[i..].find('\n').map_or(text.len(), |n| i + n);
            let line = &text[i..end];
            // `// editor help only` includes keep a lighter tone; ordinary comments are green.
            push(&mut job, line, c[2]);
            i = end;
        } else if ch == '#' || ch == '@' {
            let end = text[i + 1..].find(|x: char| !(x.is_alphanumeric() || x == '_')).map_or(text.len(), |n| i + 1 + n);
            push(&mut job, &text[i..end], c[5]);
            i = end;
        } else if ch.is_ascii_digit() {
            let end = text[i..].find(|x: char| !(x.is_ascii_alphanumeric() || x == '.' || x == '_')).map_or(text.len(), |n| i + n);
            push(&mut job, &text[i..end], c[3]);
            i = end;
        } else if ch == '"' {
            let end = text[i + 1..].find('"').map_or(text.len(), |n| i + n + 2);
            push(&mut job, &text[i..end], c[2]);
            i = end;
        } else if ch.is_alphabetic() || ch == '_' {
            let end = text[i..].find(|x: char| !(x.is_alphanumeric() || x == '_' || (x == '.' && lang == Lang::Wgsl))).map_or(text.len(), |n| i + n);
            let word = &text[i..end];
            // `in.world_pos` style members are looked up whole, plain words by their own name.
            let base = word.split('.').next().unwrap_or(word);
            let is_macro = lang == Lang::Rust && text[end..].starts_with('!');
            let col = if lang == Lang::Rust {
                if RUST_KEYWORDS.contains(&word) {
                    c[0]
                } else if RUST_TYPES.contains(&word) || word.chars().next().is_some_and(|x| x.is_uppercase()) {
                    c[1]
                } else if is_macro || rust_api().iter().any(|a| a.0 == word) {
                    c[4]
                } else {
                    c[6]
                }
            } else if KEYWORDS.contains(&word) {
                c[0]
            } else if TYPES.contains(&base) {
                c[1]
            } else if API.iter().any(|a| a.0 == word) || API.iter().any(|a| a.0 == base && word.contains('.')) {
                c[4]
            } else {
                c[6]
            };
            push(&mut job, word, col);
            i = end;
        } else {
            let end = text[i..].char_indices().nth(1).map_or(text.len(), |(n, _)| i + n);
            push(&mut job, &text[i..end], c[6]);
            i = end;
        }
    }
    job
}

/// The identifier ending at byte `pos`.
fn word_before(text: &str, pos: usize) -> (usize, &str) {
    let pos = pos.min(text.len());
    let start = text[..pos].rfind(|x: char| !(x.is_alphanumeric() || x == '_' || x == '.' || x == '#')).map_or(0, |n| n + text[n..].chars().next().map_or(1, |c| c.len_utf8()));
    (start, &text[start..pos])
}

/// Code box with colours, comment tones and an autocomplete / explanation strip underneath.
/// Returns true when the text changed.
pub fn code_editor(ui: &mut egui::Ui, id: egui::Id, text: &mut String) -> bool {
    let dark = ui.visuals().dark_mode;
    let font = FontId::monospace(13.0);
    let mut layouter = |ui: &egui::Ui, s: &dyn egui::TextBuffer, wrap: f32| ui.fonts_mut(|f| f.layout_job(highlight(s.as_str(), dark, FontId::monospace(13.0), wrap)));
    let _ = font;
    let mut changed = false;
    let height = (ui.available_height() - 120.0).max(160.0);
    let mut cursor_byte = None;
    egui::ScrollArea::both().id_salt((id, "scroll")).max_height(height).auto_shrink([false, false]).show(ui, |ui| {
        let out = egui::TextEdit::multiline(text).id(id).font(FontId::monospace(13.0)).code_editor().desired_width(f32::INFINITY).desired_rows(28).lock_focus(true).layouter(&mut layouter).show(ui);
        changed = out.response.changed();
        if let Some(r) = out.cursor_range {
            cursor_byte = Some(text.char_indices().nth(r.primary.index.0).map_or(text.len(), |(b, _)| b));
        }
    });

    // Hints for the word under the cursor.
    ui.separator();
    let (start, word) = cursor_byte.map(|p| word_before(text, p)).map(|(s, w)| (s, w.to_string())).unwrap_or((0, String::new()));
    let mut insert: Option<&str> = None;
    let matches: Vec<&(&str, &str, &str)> = if word.len() >= 2 { API.iter().filter(|a| a.0.starts_with(word.as_str()) || a.0.contains(word.as_str())).take(5).collect() } else { Vec::new() };
    if matches.is_empty() {
        ui.weak(crate::i18n::t("Type 2+ letters to see hints for Kvas functions. Green text is a comment; // starts one."));
    } else {
        for (name, sig, doc) in matches {
            ui.horizontal_wrapped(|ui| {
                if ui.link(egui::RichText::new(*name).monospace().strong()).on_hover_text(crate::i18n::t("Click to insert")).clicked() {
                    insert = Some(name);
                }
                ui.label(egui::RichText::new(*sig).monospace().weak());
            });
            ui.label(egui::RichText::new(*doc).size(11.5));
        }
    }
    if let (Some(name), Some(end)) = (insert, cursor_byte) {
        text.replace_range(start..end, name);
        changed = true;
    }
    changed
}
