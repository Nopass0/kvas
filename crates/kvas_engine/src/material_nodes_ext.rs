//! The third part of the node library: light inputs, geometry masks, normal tools, advanced noises,
//! patterns, a colour ramp, blend modes and extra maths. See [`crate::material_graph`].
//!
//! Helper functions used here (`lib_fractal`, `lib_blend`...) live in the renderer's `builtin:lib/nodes`.

use crate::material_graph::NodeSpec;

const COORD: &str = "vec3<f32>(in.uv, 0.0)";
const ZERO: &str = "vec3<f32>(0.0)";
const ONE: &str = "vec3<f32>(1.0)";
const HALF: &str = "vec3<f32>(0.5)";
const WORLD: &str = "in.world_pos";

macro_rules! node {
    ($kind:expr, $cat:expr, [$($i:expr),*], [$($p:expr),*], $expr:expr) => {
        NodeSpec { kind: $kind, category: $cat, inputs: &[$($i),*], params: &[$($p),*], expr: $expr, color: false }
    };
}
macro_rules! tinted {
    ($kind:expr, $cat:expr, [$($i:expr),*], [$($p:expr),*], $expr:expr) => {
        NodeSpec { kind: $kind, category: $cat, inputs: &[$($i),*], params: &[$($p),*], expr: $expr, color: true }
    };
}

/// Menu order of the node categories.
pub const CATEGORIES: &[&str] = &["Input", "Geometry", "Light", "Normal", "Noise", "Procedural", "Colour", "Math", "Vector", "Utility", "Effect"];

/// Nodes added in the third wave.
pub const EXT_SPECS: &[NodeSpec] = &[
    // ---- light
    node!("Light angle", "Light", [], [], "vec3<f32>(dot(n, normalize(globals.sun_dir.xyz)))"),
    node!("Half lambert", "Light", [], [("power", 1.0, 0.1, 4.0)], "vec3<f32>(pow(dot(n, normalize(globals.sun_dir.xyz)) * 0.5 + 0.5, {p0}))"),
    node!("Specular highlight", "Light", [], [("shininess", 64.0, 1.0, 512.0), ("intensity", 1.0, 0.0, 10.0)], "vec3<f32>(pow(max(dot(n, normalize(normalize(globals.sun_dir.xyz) + view)), 0.0), {p0}) * {p1})"),
    tinted!("Rim light", "Light", [], [("r", 0.4, 0.0, 1.0), ("g", 0.7, 0.0, 1.0), ("b", 1.0, 0.0, 1.0), ("power", 3.0, 0.1, 12.0), ("intensity", 1.0, 0.0, 20.0)], "(vec3<f32>({p0}, {p1}, {p2}) * pow(1.0 - max(dot(n, view), 0.0), max({p3}, 0.01)) * {p4})"),
    node!("Backlight", "Light", [], [("sharpness", 6.0, 1.0, 64.0)], "vec3<f32>(pow(max(dot(view, -normalize(globals.sun_dir.xyz)), 0.0), {p0}))"),
    node!("Facing ratio", "Light", [], [], "vec3<f32>(max(dot(n, view), 0.0))"),
    node!("Sun with shadow", "Light", [], [], "vec3<f32>(max(dot(n, normalize(globals.sun_dir.xyz)), 0.0) * shadow_factor(in.world_pos, n))"),
    node!("Sun lighting", "Light", [], [], "(globals.sun_color.rgb * max(dot(n, normalize(globals.sun_dir.xyz)), 0.0) * shadow_factor(in.world_pos, n))"),
    node!("Sky light", "Light", [], [], "env_color(n)"),
    tinted!("Glow", "Light", [], [("r", 1.0, 0.0, 1.0), ("g", 0.6, 0.0, 1.0), ("b", 0.2, 0.0, 1.0), ("intensity", 2.0, 0.0, 50.0)], "(vec3<f32>({p0}, {p1}, {p2}) * {p3})"),
    tinted!("Pulsing glow", "Light", [], [("r", 1.0, 0.0, 1.0), ("g", 0.3, 0.0, 1.0), ("b", 0.1, 0.0, 1.0), ("intensity", 3.0, 0.0, 50.0), ("speed", 1.0, 0.0, 20.0)], "(vec3<f32>({p0}, {p1}, {p2}) * {p3} * (0.5 + 0.5 * sin(globals.time.x * {p4} * 6.2831853)))"),
    // ---- geometry
    node!("Direction mask", "Geometry", [], [("x", 0.0, -1.0, 1.0), ("y", 1.0, -1.0, 1.0), ("z", 0.0, -1.0, 1.0), ("threshold", 0.5, -1.0, 1.0), ("softness", 0.2, 0.001, 2.0)], "vec3<f32>(smoothstep({p3}, {p3} + {p4}, dot(n, normalize(vec3<f32>({p0}, {p1}, {p2}) + vec3<f32>(0.00001)))))"),
    node!("Surface sides", "Geometry", [], [], "vec3<f32>(smoothstep(0.55, 0.95, n.y), 1.0 - smoothstep(0.55, 0.95, abs(n.y)), smoothstep(0.55, 0.95, -n.y))"),
    node!("Triplanar weights", "Geometry", [], [("sharpness", 4.0, 1.0, 32.0)], "(pow(abs(n), vec3<f32>({p0})) / max(dot(pow(abs(n), vec3<f32>({p0})), vec3<f32>(1.0)), 0.0001))"),
    node!("Local position", "Geometry", [], [], "(in.world_pos - object.model[3].xyz)"),
    node!("Object distance", "Geometry", [], [("range", 5.0, 0.1, 500.0)], "vec3<f32>(clamp(distance(in.world_pos, object.model[3].xyz) / {p0}, 0.0, 1.0))"),
    node!("World gradient", "Geometry", [], [("x", 0.0, -1.0, 1.0), ("y", 1.0, -1.0, 1.0), ("z", 0.0, -1.0, 1.0), ("from", 0.0, -1000.0, 1000.0), ("to", 4.0, -1000.0, 1000.0)], "vec3<f32>(clamp((dot(in.world_pos, vec3<f32>({p0}, {p1}, {p2})) - {p3}) / (({p4}) - ({p3}) + 0.0001), 0.0, 1.0))"),
    node!("Planar UV", "Geometry", [], [("axis (0 X, 1 Y, 2 Z)", 1.0, 0.0, 2.0), ("scale", 1.0, 0.01, 100.0)], "(select(select(vec3<f32>(in.world_pos.yz, 0.0), vec3<f32>(in.world_pos.xz, 0.0), {p0} > 0.5), vec3<f32>(in.world_pos.xy, 0.0), {p0} > 1.5) * {p1})"),
    node!("Normal UV", "Geometry", [], [], "vec3<f32>(atan2(n.z, n.x) / 6.2831853 + 0.5, acos(clamp(n.y, -1.0, 1.0)) / 3.14159265, 0.0)"),
    node!("Wave offset", "Geometry", [], [("height", 0.1, 0.0, 5.0), ("frequency", 2.0, 0.0, 30.0), ("speed", 1.0, -10.0, 10.0)], "vec3<f32>(0.0, sin(in.world_pos.x * {p1} + globals.time.x * {p2}) * cos(in.world_pos.z * {p1} * 0.8 + globals.time.x * {p2} * 0.7) * {p0}, 0.0)"),
    node!("Wind sway", "Geometry", [], [("strength", 0.15, 0.0, 3.0), ("speed", 1.5, 0.0, 20.0), ("scale", 0.8, 0.0, 10.0)], "(vec3<f32>(sin(globals.time.x * {p1} + in.world_pos.x * {p2}), 0.0, cos(globals.time.x * {p1} * 0.8 + in.world_pos.z * {p2})) * {p0} * max(in.world_pos.y - object.model[3].y, 0.0))"),
    // ---- normals and heights
    node!("Normal strength", "Normal", [("Normal", "n")], [("strength", 1.0, 0.0, 4.0)], "normalize(n + ({0} - n) * {p0})"),
    node!("Normal blend", "Normal", [("Base", "n"), ("Detail", "n")], [], "normalize({0} + {1} - n)"),
    node!("Normal to colour", "Normal", [("Normal", "n")], [], "({0} * 0.5 + vec3<f32>(0.5))"),
    node!("Noise bump", "Normal", [("Coords", WORLD)], [("scale", 4.0, 0.05, 100.0), ("strength", 1.0, 0.0, 10.0)], "lib_bump(lib_fractal(({0}) * {p0}, 4.0, 2.0, 0.5, 0.0), in.world_pos, n, {p1})"),
    node!("Height to normal", "Normal", [("UV", COORD)], [("slot (0 albedo, 1 normal, 2 ORM)", 0.0, 0.0, 2.0), ("tiling", 1.0, 0.01, 100.0), ("strength", 4.0, 0.0, 32.0)], "lib_tex_normal({tex}, albedo_smp, ({0}).xy * {p1}, {p2})"),
    node!("Height blend", "Math", [("Height A", ZERO), ("Height B", ZERO), ("Blend", HALF)], [("sharpness", 0.2, 0.001, 2.0)], "clamp(((({1} + {2}) - ({0} + (vec3<f32>(1.0) - {2}))) / max({p0}, 0.001)) + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0))"),
    node!("Triplanar texture", "Input", [], [("slot (0 albedo, 1 normal, 2 ORM)", 0.0, 0.0, 2.0), ("scale", 1.0, 0.01, 50.0), ("sharpness", 4.0, 1.0, 32.0)], "lib_triplanar({tex}, albedo_smp, in.world_pos * {p1}, n, {p2})"),
    // ---- noise
    node!("Fractal noise", "Noise", [("Coords", WORLD)], [("scale", 3.0, 0.01, 100.0), ("octaves", 5.0, 1.0, 8.0), ("lacunarity", 2.0, 1.2, 4.0), ("gain", 0.5, 0.1, 0.9), ("type (0 fBm, 1 ridged, 2 billow)", 0.0, 0.0, 2.0)], "vec3<f32>(lib_fractal(({0}) * {p0}, {p1}, {p2}, {p3}, {p4}))"),
    node!("Perlin noise", "Noise", [("Coords", WORLD)], [("scale", 4.0, 0.01, 100.0)], "vec3<f32>(lib_gnoise3(({0}) * {p0}))"),
    node!("Voronoi cells", "Noise", [("Coords", WORLD)], [("scale", 4.0, 0.01, 100.0), ("randomness", 1.0, 0.0, 1.0), ("output (0 distance, 1 edges, 2 cell id, 3 F2)", 0.0, 0.0, 3.0)], "vec3<f32>(lib_voro_out(lib_voronoi3(({0}) * {p0}, {p1}), {p2}))"),
    node!("Cracks", "Noise", [("Coords", WORLD)], [("scale", 3.0, 0.01, 100.0), ("width", 0.08, 0.005, 0.5)], "vec3<f32>(lib_cracks(({0}) * {p0}, {p1}))"),
    node!("Warped noise", "Noise", [("Coords", WORLD)], [("scale", 3.0, 0.01, 100.0), ("warp", 1.5, 0.0, 8.0)], "vec3<f32>(lib_warp(({0}) * {p0}, {p1}))"),
    node!("Clouds", "Noise", [("Coords", WORLD)], [("scale", 2.0, 0.01, 100.0), ("coverage", 0.45, 0.0, 1.0), ("softness", 0.3, 0.01, 1.0)], "vec3<f32>(smoothstep({p1}, {p1} + {p2}, lib_fractal(({0}) * {p0}, 5.0, 2.0, 0.5, 0.0)))"),
    node!("Grunge", "Noise", [("Coords", WORLD)], [("scale", 3.0, 0.01, 100.0), ("contrast", 1.8, 0.1, 8.0)], "vec3<f32>(clamp((lib_fractal(({0}) * {p0}, 6.0, 2.2, 0.55, 0.0) - 0.5) * {p1} + 0.5, 0.0, 1.0))"),
    node!("Scratches", "Noise", [("Coords", COORD)], [("density", 40.0, 2.0, 400.0), ("angle", 0.4, -3.15, 3.15)], "vec3<f32>(lib_scratches(({0}).xy, {p0}, {p1}))"),
    // ---- patterns
    node!("Grid lines", "Procedural", [("Coords", COORD)], [("cells", 8.0, 1.0, 200.0), ("line width", 0.05, 0.001, 0.5)], "vec3<f32>(lib_grid(({0}).xy, {p0}, {p1}))"),
    node!("Planks", "Procedural", [("Coords", COORD)], [("rows", 6.0, 1.0, 100.0), ("plank length", 3.0, 0.2, 40.0), ("gap", 0.03, 0.0, 0.3)], "vec3<f32>(lib_planks(({0}).xy, {p0}, {p1}, {p2}))"),
    node!("Weave", "Procedural", [("Coords", COORD)], [("threads", 12.0, 1.0, 200.0)], "vec3<f32>(lib_weave(({0}).xy, {p0}))"),
    // ---- colour
    node!("Color ramp", "Colour", [("Position", HALF), ("Stop 1", "vec3<f32>(0.02, 0.02, 0.05)"), ("Stop 2", "vec3<f32>(0.7, 0.15, 0.1)"), ("Stop 3", "vec3<f32>(1.0, 0.7, 0.2)"), ("Stop 4", "vec3<f32>(1.0, 1.0, 0.9)")], [("position 1", 0.0, 0.0, 1.0), ("position 2", 0.33, 0.0, 1.0), ("position 3", 0.66, 0.0, 1.0), ("position 4", 1.0, 0.0, 1.0)], "mix(mix(mix({1}, {2}, clamp((({0}).x - {p0}) / max({p1} - {p0}, 0.0001), 0.0, 1.0)), {3}, clamp((({0}).x - {p1}) / max({p2} - {p1}, 0.0001), 0.0, 1.0)), {4}, clamp((({0}).x - {p2}) / max({p3} - {p2}, 0.0001), 0.0, 1.0))"),
    node!("Blend colours", "Colour", [("A", HALF), ("B", HALF), ("Factor", ONE)], [("mode (0 mix .. 12 hard light)", 0.0, 0.0, 12.0)], "mix({0}, lib_blend({0}, {1}, {p0}), clamp(({2}).x, 0.0, 1.0))"),
    node!("HSV adjust", "Colour", [("Colour", HALF)], [("hue shift", 0.0, -0.5, 0.5), ("saturation", 1.0, 0.0, 3.0), ("value", 1.0, 0.0, 3.0)], "lib_hsv_adjust({0}, {p0}, {p1}, {p2})"),
    node!("HSV to RGB", "Colour", [("Hue", ZERO), ("Saturation", ONE), ("Value", ONE)], [], "lib_hsv_to_rgb(vec3<f32>(({0}).x, ({1}).x, ({2}).x))"),
    node!("RGB to HSV", "Colour", [("Colour", HALF)], [], "lib_rgb_to_hsv({0})"),
    node!("Lift gamma gain", "Colour", [("Colour", HALF)], [("lift", 0.0, -0.5, 0.5), ("gamma", 1.0, 0.1, 4.0), ("gain", 1.0, 0.0, 4.0)], "pow(max({0} * {p2} + vec3<f32>({p0}), vec3<f32>(0.0)), vec3<f32>(1.0 / max({p1}, 0.01)))"),
    node!("Colour temperature", "Colour", [("Colour", HALF)], [("warmth", 0.0, -1.0, 1.0)], "({0} * vec3<f32>(1.0 + {p0} * 0.3, 1.0, 1.0 - {p0} * 0.3))"),
    tinted!("Colour key", "Colour", [("Colour", HALF)], [("r", 1.0, 0.0, 1.0), ("g", 0.0, 0.0, 1.0), ("b", 0.0, 0.0, 1.0), ("tolerance", 0.3, 0.0, 1.75), ("softness", 0.1, 0.0, 1.0)], "vec3<f32>(1.0 - smoothstep({p3}, {p3} + {p4} + 0.0001, distance({0}, vec3<f32>({p0}, {p1}, {p2}))))"),
    node!("S-curve", "Colour", [("Value", HALF)], [("amount", 0.7, 0.05, 0.95)], "lib_gain({0}, {p0})"),
    node!("Bias curve", "Colour", [("Value", HALF)], [("bias", 0.5, 0.01, 0.99)], "lib_bias({0}, {p0})"),
    // ---- maths
    node!("Exponential", "Math", [("Value", ZERO)], [], "exp({0})"),
    node!("Logarithm", "Math", [("Value", ONE)], [], "log(max({0}, vec3<f32>(0.0001)))"),
    node!("Reciprocal", "Math", [("Value", ONE)], [], "(vec3<f32>(1.0) / select(vec3<f32>(0.0001), {0}, abs({0}) >= vec3<f32>(0.0001)))"),
    node!("Inverse lerp", "Math", [("Value", HALF)], [("from", 0.0, -100.0, 100.0), ("to", 1.0, -100.0, 100.0)], "(({0} - vec3<f32>({p0})) / vec3<f32>({p1} - {p0} + 0.00001))"),
    node!("Map range", "Math", [("Value", HALF)], [("in min", 0.0, -100.0, 100.0), ("in max", 1.0, -100.0, 100.0), ("out min", 0.0, -100.0, 100.0), ("out max", 1.0, -100.0, 100.0)], "(clamp(({0} - vec3<f32>({p0})) / vec3<f32>({p1} - {p0} + 0.00001), vec3<f32>(0.0), vec3<f32>(1.0)) * vec3<f32>({p3} - {p2}) + vec3<f32>({p2}))"),
    node!("Snap", "Math", [("Value", HALF)], [("step", 0.1, 0.0001, 100.0)], "(floor({0} / vec3<f32>({p0}) + vec3<f32>(0.5)) * vec3<f32>({p0}))"),
    node!("Arc tangent 2", "Math", [("Y", ZERO), ("X", ONE)], [], "vec3<f32>(atan2(({0}).x, ({1}).x))"),
    node!("Smootherstep", "Math", [("Value", HALF)], [("edge 0", 0.0, -10.0, 10.0), ("edge 1", 1.0, -10.0, 10.0)], "lib_smoother({0}, {p0}, {p1})"),
    node!("Less than", "Math", [("A", ZERO), ("B", HALF)], [], "select(vec3<f32>(0.0), vec3<f32>(1.0), {0} < {1})"),
    node!("Average", "Math", [("A", ZERO), ("B", ONE)], [], "(({0} + {1}) * 0.5)"),
    // ---- vectors
    node!("Rotate around axis", "Vector", [("Vector", "vec3<f32>(1.0, 0.0, 0.0)"), ("Axis", "vec3<f32>(0.0, 1.0, 0.0)")], [("angle", 0.0, -6.3, 6.3)], "lib_rotate_axis({0}, {1}, {p0})"),
    node!("Refract", "Vector", [("Incident", "vec3<f32>(0.0, -1.0, 0.0)"), ("Normal", "vec3<f32>(0.0, 1.0, 0.0)")], [("index ratio", 0.75, 0.1, 2.0)], "refract({0}, normalize({1} + vec3<f32>(0.00001)), {p0})"),
    node!("Twirl coords", "Vector", [("Coords", COORD)], [("amount", 3.0, -20.0, 20.0)], "vec3<f32>(lib_twirl(({0}).xy, {p0}), 0.0)"),
];

/// Nodes of this file that read the mesh, light or camera.
pub const MESH_ONLY: &[&str] = &[
    "Light angle", "Half lambert", "Specular highlight", "Rim light", "Backlight", "Facing ratio", "Sun with shadow", "Sun lighting", "Sky light", "Direction mask", "Surface sides", "Triplanar weights",
    "Local position", "Object distance", "World gradient", "Planar UV", "Normal UV", "Wave offset", "Wind sway", "Normal strength", "Normal blend", "Normal to colour", "Noise bump", "Height to normal",
    "Triplanar texture",
];

/// Nodes that cannot feed the vertex stage (derivatives, shadow or texture sampling).
pub const FRAGMENT_ONLY: &[&str] = &["Sun with shadow", "Sun lighting", "Sky light", "Noise bump", "Height to normal", "Triplanar texture"];

/// `(kind, English help, Russian help)` for the nodes of this file.
pub const HELP: &[(&str, &str, &str)] = &[
    ("Light angle", "Dot product of the normal and the sun direction: 1 facing the sun, -1 facing away.", "Скалярное произведение нормали и направления на солнце: 1 - к солнцу, -1 - от него."),
    ("Half lambert", "Soft wrap-around sun lighting (0..1) without a hard dark side.", "Мягкая освещённость солнцем (0..1) без жёсткой тёмной стороны."),
    ("Specular highlight", "Blinn-Phong glint of the sun. Higher shininess = smaller, sharper spot.", "Блик солнца по Блинну-Фонгу. Чем больше блеск, тем меньше и резче пятно."),
    ("Rim light", "A coloured glow on the edges that face away from the camera. Plug into Emission.", "Цветное свечение на краях, отвёрнутых от камеры. Подключайте в Emission."),
    ("Backlight", "Bright when the camera looks towards the sun through the surface (leaves, cloth, ice).", "Ярко, когда камера смотрит на солнце сквозь поверхность (листья, ткань, лёд)."),
    ("Facing ratio", "1 where the surface faces the camera, 0 on silhouettes.", "1 там, где поверхность смотрит на камеру, 0 на силуэте."),
    ("Sun with shadow", "Sun lighting amount (0..1) that respects shadows.", "Освещённость солнцем (0..1) с учётом теней."),
    ("Sun lighting", "Sun colour times the lighting amount, with shadows. Add Sky light for a full diffuse term.", "Цвет солнца на освещённость с тенями. Добавьте Sky light для полного диффузного света."),
    ("Sky light", "Ambient light from the sky in the direction of the normal.", "Рассеянный свет неба в направлении нормали."),
    ("Glow", "A constant colour multiplied by an intensity. Plug into Emission to make things shine.", "Постоянный цвет, умноженный на яркость. В Emission - чтобы объект светился."),
    ("Pulsing glow", "A glow that pulses over time.", "Свечение, пульсирующее во времени."),
    ("Direction mask", "White where the surface faces the chosen direction (x, y, z). Top = (0,1,0).", "Белое там, где поверхность смотрит в выбранном направлении (x, y, z). Верх = (0,1,0)."),
    ("Surface sides", "Red = top faces, green = side faces, blue = bottom faces. Split with Red / Green / Blue nodes.", "Красный - верхние грани, зелёный - боковые, синий - нижние. Разделите нодами Red / Green / Blue."),
    ("Triplanar weights", "How strongly each world axis projection (X, Y, Z) applies on this surface.", "С какой силой действует проекция по каждой мировой оси (X, Y, Z)."),
    ("Local position", "Position relative to the object's origin.", "Позиция относительно центра объекта."),
    ("Object distance", "0 at the object's centre, 1 at the chosen range.", "0 в центре объекта, 1 на выбранном расстоянии."),
    ("World gradient", "A 0..1 ramp along a world direction between two values (fog by height, snow line).", "Градиент 0..1 вдоль мирового направления между двумя значениями (туман по высоте, снеговая линия)."),
    ("Planar UV", "Texture coordinates projected from world space along an axis.", "Текстурные координаты, спроецированные из мирового пространства вдоль оси."),
    ("Normal UV", "Spherical coordinates from the normal (for sky-like mapping).", "Сферические координаты из нормали (для небесного маппинга)."),
    ("Wave offset", "Vertex offset for waves. Connect to Shader output > Vertex offset.", "Смещение вершин для волн. Подключается в Shader output > Vertex offset."),
    ("Wind sway", "Vertex offset that bends things in the wind, more at the top. Connect to Vertex offset.", "Смещение вершин, раскачивающее объект ветром сильнее наверху. В Vertex offset."),
    ("Normal strength", "Blends between the surface normal and the connected normal.", "Смешивает нормаль поверхности и подключённую нормаль."),
    ("Normal blend", "Adds a detail normal on top of a base normal.", "Накладывает детальную нормаль на базовую."),
    ("Normal to colour", "Shows a normal as a colour (for previews and debugging).", "Показывает нормаль как цвет (для превью и отладки)."),
    ("Noise bump", "Bumps the surface with fractal noise. Connect to Output > Normal.", "Бугры по фрактальному шуму. Подключайте в Output > Normal."),
    ("Height to normal", "Makes a normal-map colour from the height in a texture slot. Feed it to Normal map.", "Строит цвет карты нормалей из высоты в слоте текстуры. Подключайте в Normal map."),
    ("Height blend", "A blend mask where the higher of two heights wins (rock through snow, moss in cracks).", "Маска смешивания, где побеждает более высокая из двух высот (камень сквозь снег, мох в трещинах)."),
    ("Triplanar texture", "Projects a material image from three world axes, no stretching on any slope.", "Проецирует картинку материала по трём осям мира, без растяжения на склонах."),
    ("Fractal noise", "Layered noise with control over octaves, lacunarity and gain. Types: fBm, ridged, billow.", "Слоистый шум с настройкой октав, лакунарности и усиления. Типы: fBm, ridged, billow."),
    ("Perlin noise", "Smooth gradient noise (0..1).", "Гладкий градиентный шум (0..1)."),
    ("Voronoi cells", "3D cell noise. Outputs: distance, cell edges, random id per cell, second distance.", "3D-ячеистый шум. Выходы: расстояние, края ячеек, случайный id, второе расстояние."),
    ("Cracks", "Thin cracks along cell borders (dry ground, ice, shattered stone).", "Тонкие трещины по границам ячеек (сухая земля, лёд, разбитый камень)."),
    ("Warped noise", "Noise bent by other noise - swirly, organic shapes.", "Шум, искривлённый другим шумом - завитки и органичные формы."),
    ("Clouds", "Soft cloud-like blobs. Coverage sets how much is filled.", "Мягкие облакообразные пятна. Coverage задаёт заполненность."),
    ("Grunge", "Dirt and wear mask. Multiply with a base colour or use for roughness.", "Маска грязи и износа. Умножайте на цвет или используйте для шероховатости."),
    ("Scratches", "Thin stretched scratches at an angle.", "Тонкие вытянутые царапины под углом."),
    ("Grid lines", "A grid of thin lines (white lines on black).", "Сетка тонких линий (белые линии на чёрном)."),
    ("Planks", "Wooden floor boards with random tones and gaps.", "Доски пола со случайными оттенками и зазорами."),
    ("Weave", "Over-under fabric weave pattern.", "Тканое плетение нитей."),
    ("Color ramp", "Maps 0..1 to a gradient of four colour stops placed at four positions.", "Переводит 0..1 в градиент из четырёх цветовых точек в заданных позициях."),
    ("Blend colours", "Photoshop-style blend modes: 0 mix, 1 multiply, 2 screen, 3 overlay, 4 add, 5 subtract, 6 difference, 7 darken, 8 lighten, 9 soft light, 10 dodge, 11 burn, 12 hard light.", "Режимы наложения как в Photoshop: 0 mix, 1 multiply, 2 screen, 3 overlay, 4 add, 5 subtract, 6 difference, 7 darken, 8 lighten, 9 soft light, 10 dodge, 11 burn, 12 hard light."),
    ("HSV adjust", "Shifts hue and scales saturation and brightness.", "Сдвигает оттенок и масштабирует насыщенность и яркость."),
    ("HSV to RGB", "Builds a colour from hue, saturation and value.", "Собирает цвет из оттенка, насыщенности и яркости."),
    ("RGB to HSV", "Splits a colour into hue (red), saturation (green), value (blue).", "Раскладывает цвет на оттенок (R), насыщенность (G), яркость (B)."),
    ("Lift gamma gain", "Classic three-way colour correction.", "Классическая трёхточечная цветокоррекция."),
    ("Colour temperature", "Warm (orange) or cool (blue) tint.", "Тёплый (оранжевый) или холодный (синий) оттенок."),
    ("Colour key", "White where the colour is close to the key colour.", "Белое там, где цвет близок к выбранному."),
    ("S-curve", "Contrast curve that keeps black and white fixed.", "Контрастная кривая, сохраняющая чёрное и белое."),
    ("Bias curve", "Bends mid-tones towards dark or light.", "Сдвигает средние тона к тёмному или светлому."),
    ("Exponential", "e raised to the value.", "Экспонента."),
    ("Logarithm", "Natural logarithm.", "Натуральный логарифм."),
    ("Reciprocal", "1 divided by the value.", "1, делённая на значение."),
    ("Inverse lerp", "Where the value lies between from and to (0..1 inside).", "Где лежит значение между from и to (0..1 внутри)."),
    ("Map range", "Remaps a range to another and clamps the result.", "Переводит диапазон в другой с ограничением результата."),
    ("Snap", "Rounds to the nearest multiple of the step.", "Округляет до ближайшего кратного шагу."),
    ("Arc tangent 2", "Angle of the point (X, Y) in radians.", "Угол точки (X, Y) в радианах."),
    ("Smootherstep", "A smoother version of Smoothstep.", "Более гладкая версия Smoothstep."),
    ("Less than", "1 where A < B, otherwise 0.", "1 где A < B, иначе 0."),
    ("Average", "Mean of two values.", "Среднее двух значений."),
    ("Rotate around axis", "Rotates a vector around an axis by an angle (radians).", "Поворачивает вектор вокруг оси на угол (радианы)."),
    ("Refract", "Bends a direction through a surface (glass, water).", "Преломляет направление на поверхности (стекло, вода)."),
    ("Twirl coords", "Swirls coordinates around the centre.", "Закручивает координаты вокруг центра."),
    ("Object colour", "The colour set in the MeshRenderer of the object.", "Цвет, заданный в MeshRenderer объекта."),
    ("Slope mask", "1 on flat ground, 0 on steep slopes (between the two thresholds).", "1 на ровной земле, 0 на крутых склонах (между двумя порогами)."),
    ("Camera distance", "0 at the camera, 1 at the chosen range (fog, level of detail).", "0 у камеры, 1 на выбранной дальности (туман, детализация)."),
    ("View direction", "Unit vector from the surface to the camera.", "Единичный вектор от поверхности к камере."),
    ("Reflection", "The view direction mirrored by the surface normal.", "Направление взгляда, отражённое нормалью поверхности."),
    ("Camera position", "World position of the camera.", "Позиция камеры в мире."),
    ("Object position", "World position of the object's origin.", "Позиция центра объекта в мире."),
    ("Object random", "A random number that is different for every object but constant over time.", "Случайное число, разное у каждого объекта, но постоянное во времени."),
    ("Screen position", "Pixel position on the screen (0..1).", "Позиция пикселя на экране (0..1)."),
    ("Sun light", "How much the surface faces the sun (0..1), without shadows.", "Насколько поверхность обращена к солнцу (0..1), без теней."),
    ("Sun direction", "Unit vector pointing to the sun.", "Единичный вектор в сторону солнца."),
    ("Sun colour", "Colour and strength of the sun.", "Цвет и сила солнца."),
    ("Ambient light", "Constant ambient light colour of the scene.", "Цвет постоянного рассеянного света сцены."),
    ("Noise", "Simple smooth value noise (2D, uses the X and Z of the coordinates).", "Простой гладкий шум (2D, использует X и Z координат)."),
    ("Cell colour", "A random colour per Voronoi cell.", "Случайный цвет для каждой ячейки Вороного."),
    ("Noise 3D", "Smooth 3D noise that does not stretch on any surface.", "Гладкий 3D-шум без растяжения на любых поверхностях."),
    ("Ridged noise", "Noise with sharp ridges (mountains, veins).", "Шум с острыми гребнями (горы, прожилки)."),
    ("Turbulence", "Billowy noise with creases (smoke, fire).", "Клубящийся шум со складками (дым, огонь)."),
    ("White noise", "A random grey value per cell, no smoothing (TV static).", "Случайный серый на каждую ячейку без сглаживания (помехи)."),
    ("Sparkle", "Rare bright dots (stars, glitter).", "Редкие яркие точки (звёзды, блёстки)."),
    ("Checker", "A black and white checkerboard.", "Чёрно-белая шахматная доска."),
    ("Stripes", "Parallel stripes along X.", "Параллельные полосы вдоль X."),
    ("Bricks", "A brick wall pattern with mortar lines.", "Кирпичная кладка со швами."),
    ("Dots", "A grid of round dots.", "Сетка круглых точек."),
    ("Wave rings", "Animated rings spreading from the centre.", "Анимированные кольца, расходящиеся от центра."),
    ("Linear gradient", "0..1 from left to right (X of the coordinates).", "0..1 слева направо (X координат)."),
    ("Radial gradient", "1 in the centre, 0 at the edge.", "1 в центре, 0 на краю."),
    ("Angular gradient", "0..1 around the centre like a clock hand.", "0..1 по кругу вокруг центра, как стрелка часов."),
    ("Circle", "A soft disc.", "Мягкий круг."),
    ("Box", "A soft rectangle.", "Мягкий прямоугольник."),
    ("Hue shift", "Rotates the colours around the colour wheel.", "Поворачивает цвета по цветовому кругу."),
    ("Saturation", "0 = grey, 1 = unchanged, above 1 = more vivid.", "0 - серый, 1 - без изменений, больше 1 - ярче."),
    ("Contrast", "Pushes values away from mid grey.", "Отодвигает значения от среднего серого."),
    ("Brightness", "Multiplies the colour.", "Умножает цвет."),
    ("Gradient", "Blends two colours by a 0..1 position.", "Смешивает два цвета по позиции 0..1."),
    ("To grey", "Converts a colour to its brightness.", "Переводит цвет в его яркость."),
    ("Overlay", "Overlay blend: multiplies darks and screens lights.", "Наложение Overlay: тёмное умножается, светлое осветляется."),
    ("Screen blend", "Screen blend: always lightens.", "Наложение Screen: всегда осветляет."),
    ("Colourize", "Tints the brightness of a colour with another colour.", "Окрашивает яркость цвета другим цветом."),
    ("Gamma", "Gamma correction.", "Гамма-коррекция."),
    ("Exposure", "Brightens or darkens by photographic stops.", "Осветляет или затемняет в ступенях экспозиции."),
    ("Levels", "Sets the black and white points.", "Задаёт чёрную и белую точки."),
    ("Threshold", "Black and white by a brightness threshold.", "Чёрно-белое по порогу яркости."),
    ("Tone map", "Filmic (ACES) tone mapping of HDR colour.", "Кинематографичное (ACES) тональное отображение HDR-цвета."),
    ("Add", "A + B.", "A + B."),
    ("Subtract", "A - B.", "A - B."),
    ("Multiply", "A * B (tinting, masking).", "A * B (окрашивание, маски)."),
    ("Divide", "A / B (safe: never divides by zero).", "A / B (безопасно: без деления на ноль)."),
    ("Power", "Raises to a power (curves, sharpening masks).", "Возведение в степень (кривые, заострение масок)."),
    ("Clamp", "Limits a value to a range.", "Ограничивает значение диапазоном."),
    ("One minus", "1 - value (inverts a mask).", "1 - значение (инверсия маски)."),
    ("Sine", "A sine wave remapped to 0..1.", "Синусоида, приведённая к 0..1."),
    ("Smoothstep", "Smooth 0..1 transition between two edges.", "Плавный переход 0..1 между двумя границами."),
    ("Abs", "Absolute value.", "Модуль."),
    ("Min", "The smaller of two values.", "Меньшее из двух значений."),
    ("Max", "The larger of two values.", "Большее из двух значений."),
    ("Fract", "The fractional part (repeats 0..1).", "Дробная часть (повторяется 0..1)."),
    ("Step", "0 below the edge, 1 above it.", "0 ниже границы, 1 выше."),
    ("Remap", "Maps a range to another range (not clamped).", "Переводит диапазон в другой (без ограничения)."),
    ("Negate", "Changes the sign.", "Меняет знак."),
    ("Square root", "Square root.", "Квадратный корень."),
    ("Floor", "Rounds down.", "Округляет вниз."),
    ("Ceil", "Rounds up.", "Округляет вверх."),
    ("Round", "Rounds to the nearest whole number.", "Округляет до ближайшего целого."),
    ("Sign", "-1, 0 or 1.", "-1, 0 или 1."),
    ("Saturate", "Clamps to 0..1.", "Ограничивает диапазоном 0..1."),
    ("Modulo", "The remainder of A / B (repeating).", "Остаток от A / B (повтор)."),
    ("Multiply add", "A * B + C in one node.", "A * B + C в одной ноде."),
    ("Cosine", "A cosine wave remapped to 0..1.", "Косинусоида, приведённая к 0..1."),
    ("Triangle wave", "A zig-zag wave 0..1..0.", "Зигзагообразная волна 0..1..0."),
    ("Sawtooth", "A ramp that jumps back to 0.", "Нарастающая линия с резким возвратом в 0."),
    ("Square wave", "Alternates between 0 and 1.", "Чередует 0 и 1."),
    ("Posterize", "Reduces the number of distinct levels.", "Уменьшает число уровней значений."),
    ("Greater than", "1 where A > B, otherwise 0.", "1 где A > B, иначе 0."),
    ("Red", "Copies the red channel to all channels.", "Копирует красный канал во все каналы."),
    ("Green", "Copies the green channel to all channels.", "Копирует зелёный канал во все каналы."),
    ("Blue", "Copies the blue channel to all channels.", "Копирует синий канал во все каналы."),
    ("Length", "Length of a vector.", "Длина вектора."),
    ("Distance", "Distance between two points.", "Расстояние между двумя точками."),
    ("Dot", "Dot product (how much two directions agree).", "Скалярное произведение (насколько совпадают два направления)."),
    ("Cross", "Cross product (a direction perpendicular to both).", "Векторное произведение (направление, перпендикулярное обоим)."),
    ("Normalize", "Scales a vector to length 1.", "Приводит вектор к длине 1."),
    ("Reflect", "Mirrors a direction around a normal.", "Отражает направление относительно нормали."),
    ("Rotate coords", "Rotates texture coordinates around the centre.", "Поворачивает текстурные координаты вокруг центра."),
    ("Scale offset coords", "Tiles and shifts texture coordinates.", "Масштабирует и сдвигает текстурные координаты."),
    ("Scroll coords", "Moves coordinates over time (flowing water, conveyor belts).", "Двигает координаты во времени (вода, конвейер)."),
    ("Polar coords", "Converts coordinates to angle and radius.", "Переводит координаты в угол и радиус."),
    ("Mirror tile", "Repeats coordinates mirrored, so the seams disappear.", "Повторяет координаты зеркально, швы пропадают."),
    ("Pixel size", "Size of one screen pixel in 0..1 units.", "Размер одного пикселя экрана в единицах 0..1."),
    ("Aspect ratio", "Screen width divided by height.", "Ширина экрана, делённая на высоту."),
];

/// Help text of an extension node in the requested language, if it has one.
pub fn help(kind: &str, russian: bool) -> Option<&'static str> {
    HELP.iter().find(|h| h.0 == kind).map(|h| if russian { h.2 } else { h.1 })
}

#[cfg(test)]
mod tests {
    use crate::material_graph::{all_specs, is_output, GraphMode, MaterialGraph, to_wgsl};

    #[test]
    fn every_node_is_in_a_menu_category_and_has_help() {
        for s in all_specs().filter(|s| !is_output(s.kind)) {
            assert!(super::CATEGORIES.contains(&s.category), "{} has category {}", s.kind, s.category);
        }
        for h in super::HELP {
            assert!(crate::material_graph::spec(h.0).is_some(), "help for unknown node {}", h.0);
        }
    }

    #[test]
    fn colour_ramp_height_and_texture_nodes_generate() {
        let mut g = MaterialGraph::default_graph();
        let ramp = g.add_node("Color ramp", [0.0; 2]);
        let noise = g.add_node("Fractal noise", [0.0; 2]);
        g.connect(noise, ramp, 0);
        g.connect(ramp, 1, 0);
        let height = g.add_node("Triplanar texture", [0.0; 2]);
        g.connect(height, 1, 7);
        let wgsl = to_wgsl(&g).unwrap();
        assert!(wgsl.contains("lib_fractal") && wgsl.contains("lib_triplanar(albedo_tex"), "{wgsl}");
        assert!(wgsl.contains("lib_bump(("));
        // A shader graph may not feed the vertex stage with a texture node.
        let mut sg = MaterialGraph::default_graph_for(GraphMode::Shader);
        let tri = sg.add_node("Triplanar texture", [0.0; 2]);
        sg.connect(tri, 1, 5);
        assert!(to_wgsl(&sg).is_err());
    }
}
