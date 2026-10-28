//! The fourth part of the node library: general "Math" / "Vector math" nodes with an operation
//! selector, Blender-style texture generators with several outputs, an image texture with its
//! own file, curves, mapping and colour tools. It also holds the table of extra outputs that any
//! node may have and the conversions a linked input can apply. See [`crate::material_graph`].

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

/// Operations of the "Math" node, in the order of its selector.
pub const MATH_OPS: &[&str] = &[
    "Add", "Subtract", "Multiply", "Divide", "Power", "Logarithm", "Square root", "Absolute", "Minimum", "Maximum", "Less than", "Greater than", "Round", "Floor",
    "Ceil", "Fraction", "Modulo", "Sine", "Cosine", "Tangent", "Arctan2", "Smooth minimum", "Smooth maximum", "Snap", "Ping-pong", "Multiply add",
];
/// Operations of the "Vector math" node.
pub const VECTOR_OPS: &[&str] = &[
    "Add", "Subtract", "Multiply", "Divide", "Cross product", "Project", "Reflect", "Dot product", "Distance", "Length", "Scale", "Normalize", "Absolute", "Minimum",
    "Maximum", "Floor", "Fraction",
];
/// Blend modes of "Mix colour" (same order as `lib_blend`).
pub const BLEND_MODES: &[&str] = &["Mix", "Multiply", "Screen", "Overlay", "Add", "Subtract", "Difference", "Darken", "Lighten", "Soft light", "Colour dodge", "Colour burn", "Hard light"];

/// Named choices of selector parameters (`node kind`, `parameter index`) shown as drop-downs.
pub fn choices(kind: &str, param: usize) -> Option<&'static [&'static str]> {
    Some(match (kind, param) {
        ("Math", 0) => MATH_OPS,
        ("Vector math", 0) => VECTOR_OPS,
        ("Mix colour", 0) => BLEND_MODES,
        ("Noise texture", 0) => &["fBm", "Ridged", "Billow"],
        ("Voronoi texture", 0) => &["Distance (F1)", "Second (F2)", "Edge (F2-F1)", "Cell value"],
        ("Wave texture", 0) => &["Bands", "Rings"],
        ("Wave texture", 1) => &["Sine", "Saw", "Triangle"],
        ("Gradient texture", 0) => &["Linear", "Quadratic", "Eased", "Diagonal", "Radial", "Spherical", "Quadratic sphere"],
        ("Image texture" | "Texture" | "Triplanar texture", 0) => &["Slot 0 (albedo)", "Slot 1 (normal)", "Slot 2 (ORM)"],
        ("Color ramp", 4) => &["Linear", "Constant", "Ease"],
        ("Math", 1) | ("Mix colour", 1) => &["Off", "Clamp 0..1"],
        _ => return None,
    })
}

/// Nodes added in the fourth wave.
pub const V4_SPECS: &[NodeSpec] = &[
    // ---- input
    node!("Image texture", "Input", [("Vector", COORD)], [("slot", 0.0, 0.0, 2.0), ("tiling x", 1.0, 0.01, 100.0), ("tiling y", 1.0, 0.01, 100.0), ("offset x", 0.0, -10.0, 10.0), ("offset y", 0.0, -10.0, 10.0), ("rotation", 0.0, -1.0, 1.0)], "textureSample({tex}, albedo_smp, lib_rot2(({0}).xy * vec2<f32>({p1}, {p2}), {p5} * 6.2831853) + vec2<f32>({p3}, {p4})).rgb"),
    node!("Texture coordinate", "Geometry", [], [], "vec3<f32>(in.uv, 0.0)"),
    node!("Geometry info", "Geometry", [], [], "in.world_pos"),
    node!("Layer weight", "Light", [("Blend", HALF)], [], "vec3<f32>(pow(1.0 - max(dot(n, view), 0.0), 1.0 / max(1.0 - clamp(({0}).x, 0.0, 0.99), 0.01)))"),
    node!("RGB", "Input", [("Colour", "vec3<f32>(0.8, 0.5, 0.2)")], [], "{0}"),
    // ---- maths
    node!("Math", "Math", [("A", HALF), ("B", HALF), ("C", HALF)], [("operation", 2.0, 0.0, 25.0), ("clamp", 0.0, 0.0, 1.0)], "select(lib_math({0}, {1}, {2}, {p0}), clamp(lib_math({0}, {1}, {2}, {p0}), vec3<f32>(0.0), vec3<f32>(1.0)), {p1} > 0.5)"),
    node!("Vector math", "Vector", [("A", ZERO), ("B", ONE)], [("operation", 0.0, 0.0, 16.0)], "lib_vmath({0}, {1}, {p0})"),
    node!("Separate RGB", "Vector", [("Colour", "vec3<f32>(0.8, 0.5, 0.2)")], [], "{0}"),
    node!("Separate XYZ", "Vector", [("Vector", WORLD)], [], "{0}"),
    node!("Combine XYZ", "Vector", [("X", ZERO), ("Y", ZERO), ("Z", ZERO)], [], "vec3<f32>(({0}).x, ({1}).x, ({2}).x)"),
    node!("Mapping", "Vector", [("Vector", COORD)], [("location x", 0.0, -100.0, 100.0), ("location y", 0.0, -100.0, 100.0), ("location z", 0.0, -100.0, 100.0), ("rotation", 0.0, -1.0, 1.0), ("scale x", 1.0, -100.0, 100.0), ("scale y", 1.0, -100.0, 100.0), ("scale z", 1.0, -100.0, 100.0)], "lib_mapping({0}, vec3<f32>({p0}, {p1}, {p2}), {p3}, vec3<f32>({p4}, {p5}, {p6}))"),
    node!("Jitter coords", "Vector", [("Vector", WORLD), ("Amount", "vec3<f32>(0.2)")], [("scale", 4.0, 0.01, 100.0)], "lib_jitter({0}, ({1}).x, {p0})"),
    node!("Float curve", "Math", [("Value", HALF)], [("y at 0", 0.0, -1.0, 2.0), ("y at 0.25", 0.25, -1.0, 2.0), ("y at 0.5", 0.5, -1.0, 2.0), ("y at 0.75", 0.75, -1.0, 2.0), ("y at 1", 1.0, -1.0, 2.0)], "lib_curve_rgb({0}, {p0}, {p1}, {p2}, {p3}, {p4})"),
    node!("Clamp range", "Math", [("Value", HALF), ("Min", ZERO), ("Max", ONE)], [], "clamp({0}, min({1}, {2}), max({1}, {2}))"),
    // ---- texture generators
    node!("Noise texture", "Noise", [("Vector", WORLD), ("Scale", "vec3<f32>(4.0)"), ("Detail", "vec3<f32>(5.0)"), ("Roughness", HALF), ("Distortion", ZERO)], [("type", 0.0, 0.0, 2.0), ("lacunarity", 2.0, 1.2, 4.0)], "vec3<f32>(lib_fractal(({0}) * ({1}).x + ({4}).x * (vec3<f32>(lib_vnoise3(({0}) * ({1}).x * 0.5 + vec3<f32>(3.1)), lib_vnoise3(({0}) * ({1}).x * 0.5 + vec3<f32>(7.4)), lib_vnoise3(({0}) * ({1}).x * 0.5 + vec3<f32>(1.7))) - vec3<f32>(0.5)), clamp(({2}).x, 1.0, 8.0), {p1}, clamp(({3}).x, 0.0, 1.0), {p0}))"),
    node!("Voronoi texture", "Noise", [("Vector", WORLD), ("Scale", "vec3<f32>(5.0)"), ("Randomness", ONE)], [("feature", 0.0, 0.0, 3.0)], "lib_voronoi3(({0}) * ({1}).x, clamp(({2}).x, 0.0, 1.0))"),
    node!("Wave texture", "Procedural", [("Vector", WORLD), ("Scale", "vec3<f32>(3.0)"), ("Distortion", "vec3<f32>(2.0)"), ("Detail", "vec3<f32>(3.0)")], [("type", 0.0, 0.0, 1.0), ("profile", 0.0, 0.0, 2.0)], "vec3<f32>(lib_wave(({0}) * ({1}).x, {p0}, {p1}, ({2}).x, clamp(({3}).x, 1.0, 8.0)))"),
    node!("Gradient texture", "Procedural", [("Vector", COORD)], [("type", 0.0, 0.0, 6.0)], "vec3<f32>(lib_gradient({0}, {p0}))"),
    node!("Brick texture", "Procedural", [("Vector", COORD), ("Colour 1", "vec3<f32>(0.62, 0.24, 0.16)"), ("Colour 2", "vec3<f32>(0.45, 0.17, 0.12)"), ("Mortar", "vec3<f32>(0.75, 0.72, 0.66)"), ("Scale", "vec3<f32>(6.0)")], [("brick ratio", 2.0, 0.2, 6.0), ("row offset", 0.5, 0.0, 1.0), ("mortar size", 0.05, 0.0, 0.3), ("mortar smooth", 0.02, 0.0, 0.3), ("colour variation", 0.6, 0.0, 1.0)], "lib_brick(({0}).xy, ({4}).x, {p0}, {p1}, {p2}, {p3})"),
    // ---- colour
    node!("Mix colour", "Colour", [("Factor", HALF), ("A", "vec3<f32>(0.2)"), ("B", "vec3<f32>(0.8)")], [("blend mode", 0.0, 0.0, 12.0), ("clamp result", 0.0, 0.0, 1.0)], "select(mix({1}, lib_blend({1}, {2}, {p0}), clamp(({0}).x, 0.0, 1.0)), clamp(mix({1}, lib_blend({1}, {2}, {p0}), clamp(({0}).x, 0.0, 1.0)), vec3<f32>(0.0), vec3<f32>(1.0)), {p1} > 0.5)"),
    node!("Hue saturation value", "Colour", [("Colour", "vec3<f32>(0.8, 0.4, 0.2)"), ("Hue", HALF), ("Saturation", ONE), ("Value", ONE), ("Factor", ONE)], [], "mix({0}, lib_hsv_adjust({0}, ({1}).x - 0.5, ({2}).x, ({3}).x), clamp(({4}).x, 0.0, 1.0))"),
    node!("Invert", "Colour", [("Colour", "vec3<f32>(0.8)"), ("Factor", ONE)], [], "mix({0}, vec3<f32>(1.0) - {0}, clamp(({1}).x, 0.0, 1.0))"),
    node!("RGB curves", "Colour", [("Colour", HALF), ("Factor", ONE)], [("y at 0", 0.0, -1.0, 2.0), ("y at 0.25", 0.22, -1.0, 2.0), ("y at 0.5", 0.5, -1.0, 2.0), ("y at 0.75", 0.78, -1.0, 2.0), ("y at 1", 1.0, -1.0, 2.0)], "mix({0}, lib_curve_rgb({0}, {p0}, {p1}, {p2}, {p3}, {p4}), clamp(({1}).x, 0.0, 1.0))"),
    node!("Bright contrast", "Colour", [("Colour", HALF), ("Bright", ZERO), ("Contrast", ZERO)], [], "max((({0}) - vec3<f32>(0.5)) * (1.0 + ({2}).x) + vec3<f32>(0.5) + ({1}).x, vec3<f32>(0.0))"),
    // ---- normal
    node!("Bump", "Normal", [("Height", ZERO), ("Strength", ONE), ("Distance", "vec3<f32>(0.1)")], [("invert", 0.0, 0.0, 1.0)], "lib_bump(({0}).x * select(1.0, -1.0, {p0} > 0.5) * ({2}).x * 10.0, in.world_pos, n, clamp(({1}).x, 0.0, 4.0))"),
];

/// Nodes of this file that sample textures or use derivatives (not in camera effects / vertex stage).
pub const MESH_ONLY: &[&str] = &["Image texture", "Layer weight", "Bump", "Geometry info", "Texture coordinate"];
/// Nodes that cannot feed a vertex offset.
pub const FRAGMENT_ONLY: &[&str] = &["Image texture", "Bump"];

/// Extra outputs of a node: name and a WGSL expression where `{v}` is the node's main result and
/// `{0}`.., `{p0}`.. are its inputs and parameters, like in the main expression. Output 0 is always
/// the main result; nodes missing here have only that one.
pub fn outputs(kind: &str) -> &'static [(&'static str, &'static str)] {
    const RGB: &[(&str, &str)] = &[("Colour", "{v}"), ("R", "vec3<f32>(({v}).x)"), ("G", "vec3<f32>(({v}).y)"), ("B", "vec3<f32>(({v}).z)"), ("Luma", "vec3<f32>(lib_luma({v}))")];
    const XYZ: &[(&str, &str)] = &[("Vector", "{v}"), ("X", "vec3<f32>(({v}).x)"), ("Y", "vec3<f32>(({v}).y)"), ("Z", "vec3<f32>(({v}).z)")];
    match kind {
        "Image texture" | "Texture" | "Triplanar texture" | "Color" | "RGB" | "Separate RGB" | "Cell colour" => RGB,
        "Separate XYZ" | "World position" | "Local position" | "Object position" | "Camera position" | "Mapping" | "Jitter coords" | "Sun direction" | "View direction" | "Reflection" => XYZ,
        "UV" => &[("UV", "{v}"), ("U", "vec3<f32>(({v}).x)"), ("V", "vec3<f32>(({v}).y)")],
        "Texture coordinate" => &[("UV", "{v}"), ("Object", "(in.world_pos - object.model[3].xyz)"), ("World", "in.world_pos"), ("Normal", "n"), ("Reflection", "reflect(-view, n)"), ("Screen", "vec3<f32>(in.clip.xy * globals.viewport.zw, 0.0)")],
        "Geometry info" => &[("Position", "{v}"), ("Normal", "n"), ("View", "view"), ("Height", "vec3<f32>(in.world_pos.y)"), ("Backfacing", "vec3<f32>(select(0.0, 1.0, dot(n, view) < 0.0))"), ("Slope", "vec3<f32>(1.0 - abs(n.y))")],
        "Normal" => &[("Normal", "{v}"), ("Up facing", "vec3<f32>(max(({v}).y, 0.0))"), ("Down facing", "vec3<f32>(max(-({v}).y, 0.0))")],
        "Time" => &[("Seconds", "{v}"), ("Sine", "vec3<f32>(sin(({v}).x) * 0.5 + 0.5)"), ("Fraction", "fract({v})")],
        "Fresnel" => &[("Fresnel", "{v}"), ("Facing", "(vec3<f32>(1.0) - {v})")],
        "Layer weight" => &[("Fresnel", "{v}"), ("Facing", "vec3<f32>(1.0 - pow(max(dot(n, view), 0.0), 1.0 / max(1.0 - clamp(({0}).x, 0.0, 0.99), 0.01)))")],
        "Noise texture" => &[("Factor", "{v}"), ("Colour", "vec3<f32>(({v}).x, lib_fractal(({0}) * ({1}).x + vec3<f32>(19.1, 3.7, 8.2), clamp(({2}).x, 1.0, 8.0), {p1}, clamp(({3}).x, 0.0, 1.0), {p0}), lib_fractal(({0}) * ({1}).x + vec3<f32>(4.3, 21.9, 13.4), clamp(({2}).x, 1.0, 8.0), {p1}, clamp(({3}).x, 0.0, 1.0), {p0}))")],
        "Voronoi texture" => &[("Factor", "vec3<f32>(lib_voro_out({v}, {p0}))"), ("Distance", "vec3<f32>(({v}).x)"), ("Edge", "vec3<f32>(({v}).y - ({v}).x)"), ("Cell value", "vec3<f32>(({v}).z)"), ("Cell colour", "lib_h33(vec3<f32>(({v}).z * 97.0, ({v}).z * 31.0, ({v}).z * 57.0))")],
        "Brick texture" => &[("Colour", "mix({3}, mix({1}, {2}, clamp(({v}).y * {p4} + (1.0 - {p4}) * 0.5, 0.0, 1.0)), ({v}).x)"), ("Mask", "vec3<f32>(({v}).x)"), ("Brick value", "vec3<f32>(({v}).y)"), ("Edge distance", "vec3<f32>(({v}).z)")],
        "Bricks" => &[("Mask", "{v}"), ("Mortar", "(vec3<f32>(1.0) - {v})")],
        "Checker" | "Stripes" | "Dots" | "Wave texture" | "Gradient texture" | "Noise" | "Voronoi" | "Fractal noise" | "Perlin noise" | "Marble veins" | "Clouds" | "Grunge" | "Cracks" | "Wood rings" =>
            &[("Factor", "{v}"), ("Inverted", "(vec3<f32>(1.0) - {v})"), ("Sharp", "smoothstep(vec3<f32>(0.45), vec3<f32>(0.55), {v})")],
        "Color ramp" => &[("Colour", "{v}"), ("Alpha", "vec3<f32>(lib_luma({v}))")],
        "Combine XYZ" | "Combine" => &[("Vector", "{v}"), ("Length", "vec3<f32>(length({v}))"), ("Normalized", "normalize({v} + vec3<f32>(0.00001))")],
        "Mix colour" | "Hue saturation value" | "RGB curves" | "Invert" | "Bright contrast" | "Blend colours" | "HSV adjust" => RGB,
        _ => &[],
    }
}

/// How a linked input converts the incoming value before use. Every socket carries a `vec3`, so
/// the conversion decides what a number input reads from a colour or a vector (and the other way).
pub const CONVERSIONS: &[(&str, &str)] = &[
    ("Auto", "{v}"),
    ("Red / X", "vec3<f32>(({v}).x)"),
    ("Green / Y", "vec3<f32>(({v}).y)"),
    ("Blue / Z", "vec3<f32>(({v}).z)"),
    ("Luminance", "vec3<f32>(lib_luma({v}))"),
    ("Average", "vec3<f32>(dot({v}, vec3<f32>(0.33333)))"),
    ("Length", "vec3<f32>(length({v}))"),
    ("Largest", "vec3<f32>(max(max(({v}).x, ({v}).y), ({v}).z))"),
    ("Invert (1 - x)", "(vec3<f32>(1.0) - {v})"),
    ("Negate", "(-{v})"),
    ("Clamp 0..1", "clamp({v}, vec3<f32>(0.0), vec3<f32>(1.0))"),
    ("To 0..1 (x * 0.5 + 0.5)", "({v} * 0.5 + vec3<f32>(0.5))"),
    ("To -1..1 (x * 2 - 1)", "({v} * 2.0 - vec3<f32>(1.0))"),
];

/// Short badge drawn next to a converted input.
pub fn conversion_badge(index: u8) -> &'static str {
    ["", "R", "G", "B", "L", "Avg", "Len", "Max", "1-x", "-x", "0..1", "½", "±"].get(index as usize).copied().unwrap_or("?")
}

/// What kind of data a socket naturally holds; used for colouring sockets and picking the
/// conversion of a new link.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SocketType {
    /// One number (stored in all three channels).
    Float,
    /// An RGB colour.
    Colour,
    /// A direction or position.
    Vector,
}

/// The type of input `index` of a node kind.
pub fn input_type(name: &str, default: &str) -> SocketType {
    let colour = matches!(
        name,
        "Base color" | "Emission" | "Colour" | "From" | "To" | "Color" | "Tint" | "Low" | "Middle" | "High" | "Stop 1" | "Stop 2" | "Stop 3" | "Stop 4" | "Colour 1" | "Colour 2" | "Mortar" | "Map"
    ) || (name == "A" || name == "B") && default.contains(',');
    if colour {
        return SocketType::Colour;
    }
    if matches!(name, "Vector" | "Coords" | "UV" | "Normal" | "Direction" | "Axis" | "Offset" | "Position") || default.contains("in.") || default == "n" {
        return SocketType::Vector;
    }
    SocketType::Float
}

/// The type of output `index` of a node kind.
pub fn output_type(kind: &str, category: &str, index: usize) -> SocketType {
    let outs = outputs(kind);
    let name = outs.get(index).map_or("", |o| o.0);
    match name {
        "Colour" | "Cell colour" => return SocketType::Colour,
        "Vector" | "UV" | "Object" | "World" | "Normal" | "Reflection" | "Screen" | "Position" | "View" | "Normalized" => return SocketType::Vector,
        "" => {}
        _ => return SocketType::Float,
    }
    match category {
        "Colour" | "Light" if !matches!(kind, "Fresnel" | "Facing ratio" | "Light angle" | "Half lambert" | "Backlight" | "Sun with shadow") => SocketType::Colour,
        "Input" if matches!(kind, "Color" | "Object colour" | "Texture" | "Image texture" | "RGB") => SocketType::Colour,
        "Geometry" | "Vector" if !matches!(kind, "Height" | "Slope mask" | "Camera distance" | "Length" | "Distance" | "Dot") => SocketType::Vector,
        _ => SocketType::Float,
    }
}

/// The conversion a fresh link from `from` to `to` should use: a colour or a vector plugged into
/// a number reads its brightness or length; everything else passes as it is.
pub fn default_conversion(from: SocketType, to: SocketType) -> u8 {
    match (from, to) {
        (SocketType::Colour, SocketType::Float) => 4,
        _ => 0,
    }
}

/// English and Russian help of the nodes in this file.
pub const HELP: &[(&str, &str, &str)] = &[
    ("Image texture", "An image file of the project. Pick it in the node; up to three images per material (slots 0..2). Outputs colour, channels and brightness.", "Картинка из проекта. Выберите файл прямо в ноде; до трёх картинок на материал (слоты 0..2). Выходы: цвет, каналы и яркость."),
    ("Texture coordinate", "Every coordinate system at once: UV, object, world, normal, reflection and screen.", "Все системы координат сразу: UV, объект, мир, нормаль, отражение и экран."),
    ("Geometry info", "Position, normal, view direction, height, back side and slope of the surface point.", "Позиция, нормаль, направление взгляда, высота, обратная сторона и наклон точки поверхности."),
    ("Layer weight", "Fresnel and facing masks with a blend control; good for coatings and rims.", "Маски Френеля и «лицом к камере» с регулировкой; для лаков и окантовок."),
    ("RGB", "A colour with R, G, B and brightness outputs.", "Цвет с выходами R, G, B и яркости."),
    ("Math", "One node for 26 operations on numbers (add, power, smooth min, snap, ping-pong...). Pick the operation in the list.", "Одна нода на 26 операций с числами (сложение, степень, плавный минимум, шаг, пинг-понг...). Операция выбирается в списке."),
    ("Vector math", "Operations on vectors: cross, dot, project, reflect, distance, normalize...", "Операции с векторами: векторное и скалярное произведение, проекция, отражение, расстояние, нормализация..."),
    ("Separate RGB", "Splits a colour into R, G and B.", "Разделяет цвет на R, G и B."),
    ("Separate XYZ", "Splits a vector into X, Y and Z.", "Разделяет вектор на X, Y и Z."),
    ("Combine XYZ", "Builds a vector from three numbers.", "Собирает вектор из трёх чисел."),
    ("Mapping", "Moves, turns and scales texture coordinates.", "Сдвигает, поворачивает и масштабирует координаты текстуры."),
    ("Jitter coords", "Adds smooth noise to coordinates to break visible repetition.", "Добавляет к координатам плавный шум, чтобы убрать заметные повторы."),
    ("Float curve", "Reshapes a value by a smooth curve through five points.", "Меняет значение по плавной кривой через пять точек."),
    ("Clamp range", "Keeps a value between two inputs.", "Ограничивает значение двумя входами."),
    ("Noise texture", "Fractal noise with scale, detail, roughness and distortion inputs; outputs a factor and a colour.", "Фрактальный шум с входами масштаба, детализации, шероховатости и искажения; выходы: коэффициент и цвет."),
    ("Voronoi texture", "Cells: distance, edges, a value and a colour per cell.", "Ячейки: расстояние, края, значение и цвет каждой ячейки."),
    ("Wave texture", "Bands or rings with sine, saw or triangle profile, bent by noise.", "Полосы или кольца с синусом, пилой или треугольником, изогнутые шумом."),
    ("Gradient texture", "Linear, quadratic, eased, diagonal, radial or spherical gradient.", "Линейный, квадратичный, плавный, диагональный, радиальный или сферический градиент."),
    ("Brick texture", "Bricks with two colours, mortar, row offset and a value per brick.", "Кирпичи двух цветов с раствором, сдвигом рядов и значением каждого кирпича."),
    ("Mix colour", "Blends A and B with a mode (mix, multiply, overlay, screen...) by a factor.", "Смешивает A и B в режиме (обычный, умножение, перекрытие, экран...) по коэффициенту."),
    ("Hue saturation value", "Shifts hue and scales saturation and brightness.", "Сдвигает оттенок и меняет насыщенность и яркость."),
    ("Invert", "Inverts a colour by a factor.", "Инвертирует цвет на заданную долю."),
    ("RGB curves", "Tone curve for colours through five points.", "Тоновая кривая цвета через пять точек."),
    ("Bright contrast", "Brightness and contrast as inputs.", "Яркость и контраст как входы."),
    ("Bump", "Surface relief from a height value; connect to Output > Normal.", "Рельеф поверхности из высоты; подключайте к Output > Normal."),
];
