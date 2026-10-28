//! The second half of the node library: shader-graph outputs, vector / maths / colour / procedural
//! nodes, camera-effect nodes and the node help texts. See [`crate::material_graph`].

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

/// Nodes added after the first release; searched together with [`crate::material_graph::SPECS`].
pub const MORE_SPECS: &[NodeSpec] = &[
    // ---- outputs of the other graph kinds
    NodeSpec {
        kind: "Shader output",
        category: "Output",
        inputs: &[("Color", "vec3<f32>(0.8)"), ("Alpha", ONE), ("Emission", ZERO), ("Light amount", ONE), ("Roughness", "vec3<f32>(0.6)"), ("Vertex offset", ZERO)],
        params: &[],
        expr: "",
        color: false,
    },
    NodeSpec { kind: "Effect output", category: "Output", inputs: &[("Color", "color")], params: &[], expr: "", color: false },
    // ---- utility
    node!("Reroute", "Utility", [("Value", ZERO)], [], "{0}"),
    node!("Comment", "Utility", [], [], "vec3<f32>(0.0)"),
    node!("Inspector number", "Utility", [], [("slot 0-7 (user N)", 0.0, 0.0, 7.0), ("default", 1.0, -1000.0, 1000.0), ("zero uses default (0/1)", 1.0, 0.0, 1.0)], "vec3<f32>(0.0)"),
    node!("Time scaled", "Utility", [], [("speed", 1.0, -50.0, 50.0)], "vec3<f32>(globals.time.x * {p0})"),
    node!("Pulse", "Utility", [], [("hertz", 1.0, 0.0, 20.0)], "vec3<f32>(0.5 + 0.5 * sin(globals.time.x * {p0} * 6.2831853))"),
    // ---- scene inputs (surface shaders)
    node!("View direction", "Geometry", [], [], "view"),
    node!("Reflection", "Geometry", [], [], "reflect(-view, n)"),
    node!("Sky reflection", "Light", [], [], "env_color(reflect(-view, n))"),
    node!("Sun direction", "Light", [], [], "normalize(globals.sun_dir.xyz)"),
    node!("Sun colour", "Light", [], [], "globals.sun_color.rgb"),
    node!("Ambient light", "Light", [], [], "globals.ambient.rgb"),
    node!("Shadow", "Light", [], [], "vec3<f32>(shadow_factor(in.world_pos, n))"),
    node!("Lighting", "Light", [("Base", "vec3<f32>(0.8)")], [("roughness", 0.6, 0.0, 1.0)], "lighting(in.world_pos, n, {0}, {p0})"),
    node!("Camera position", "Geometry", [], [], "globals.camera_pos.xyz"),
    node!("Object position", "Geometry", [], [], "object.model[3].xyz"),
    node!("Object random", "Geometry", [], [], "vec3<f32>(hash31(object.model[3].xyz))"),
    node!("Screen position", "Geometry", [], [], "vec3<f32>(in.clip.xy * globals.viewport.zw, 0.0)"),
    // ---- camera effect inputs
    node!("Screen color", "Effect", [("Coords", COORD)], [], "scene_color(({0}).xy)"),
    node!("Pixel colour", "Effect", [], [], "color"),
    node!("Pixel size", "Effect", [], [], "vec3<f32>(pixel_size(), 0.0)"),
    node!("Aspect ratio", "Effect", [], [], "vec3<f32>(screen_aspect())"),
    node!("Blur", "Effect", [("Coords", COORD)], [("radius px", 2.0, 0.0, 30.0)], "((scene_color(({0}).xy) + scene_color(({0}).xy + vec2<f32>({p0}, 0.0) * pixel_size()) + scene_color(({0}).xy - vec2<f32>({p0}, 0.0) * pixel_size()) + scene_color(({0}).xy + vec2<f32>(0.0, {p0}) * pixel_size()) + scene_color(({0}).xy - vec2<f32>(0.0, {p0}) * pixel_size())) / 5.0)"),
    node!("Chromatic split", "Effect", [("Coords", COORD)], [("amount px", 3.0, 0.0, 30.0)], "vec3<f32>(scene_color(({0}).xy + vec2<f32>({p0}, 0.0) * pixel_size()).r, scene_color(({0}).xy).g, scene_color(({0}).xy - vec2<f32>({p0}, 0.0) * pixel_size()).b)"),
    node!("Edge detect", "Effect", [("Coords", COORD)], [("strength", 4.0, 0.0, 20.0)], "vec3<f32>(clamp((abs(lib_luma(scene_color(({0}).xy + vec2<f32>(pixel_size().x, 0.0))) - lib_luma(scene_color(({0}).xy - vec2<f32>(pixel_size().x, 0.0)))) + abs(lib_luma(scene_color(({0}).xy + vec2<f32>(0.0, pixel_size().y))) - lib_luma(scene_color(({0}).xy - vec2<f32>(0.0, pixel_size().y))))) * {p0}, 0.0, 1.0))"),
    node!("Pixelate", "Effect", [("Coords", COORD)], [("cells across", 160.0, 8.0, 1000.0)], "scene_color(floor(({0}).xy * {p0}) / {p0})"),
    node!("Vignette", "Effect", [("Coords", COORD)], [("strength", 0.4, 0.0, 2.0)], "vec3<f32>(lib_vignette(({0}).xy, {p0}))"),
    node!("Scanlines", "Effect", [("Coords", COORD)], [("lines", 300.0, 10.0, 2000.0)], "vec3<f32>(0.5 + 0.5 * sin(({0}).y * {p0} * 3.1415926))"),
    // ---- maths
    node!("Negate", "Math", [("Value", ZERO)], [], "(-{0})"),
    node!("Square root", "Math", [("Value", HALF)], [], "sqrt(max({0}, vec3<f32>(0.0)))"),
    node!("Floor", "Math", [("Value", HALF)], [], "floor({0})"),
    node!("Ceil", "Math", [("Value", HALF)], [], "ceil({0})"),
    node!("Round", "Math", [("Value", HALF)], [], "round({0})"),
    node!("Sign", "Math", [("Value", ZERO)], [], "sign({0})"),
    node!("Saturate", "Math", [("Value", ZERO)], [], "clamp({0}, vec3<f32>(0.0), vec3<f32>(1.0))"),
    node!("Modulo", "Math", [("A", ZERO), ("B", ONE)], [], "({0} - {1} * floor({0} / max({1}, vec3<f32>(0.0001))))"),
    node!("Multiply add", "Math", [("A", ONE), ("B", ONE), ("C", ZERO)], [], "({0} * {1} + {2})"),
    node!("Cosine", "Math", [("Value", ZERO)], [("frequency", 1.0, 0.0, 100.0)], "(cos({0} * {p0}) * 0.5 + vec3<f32>(0.5))"),
    node!("Triangle wave", "Math", [("Value", ZERO)], [("frequency", 1.0, 0.0, 100.0)], "(abs(fract({0} * {p0}) - vec3<f32>(0.5)) * 2.0)"),
    node!("Sawtooth", "Math", [("Value", ZERO)], [("frequency", 1.0, 0.0, 100.0)], "fract({0} * {p0})"),
    node!("Square wave", "Math", [("Value", ZERO)], [("frequency", 1.0, 0.0, 100.0)], "step(vec3<f32>(0.5), fract({0} * {p0}))"),
    node!("Posterize", "Math", [("Value", HALF)], [("steps", 4.0, 2.0, 64.0)], "(floor({0} * {p0} + vec3<f32>(0.5)) / {p0})"),
    node!("Greater than", "Math", [("A", ZERO), ("B", HALF)], [], "select(vec3<f32>(0.0), vec3<f32>(1.0), {0} > {1})"),
    node!("Branch", "Math", [("A", ZERO), ("B", HALF), ("If A > B", ONE), ("Otherwise", ZERO)], [], "select({3}, {2}, {0} > {1})"),
    // ---- vector
    node!("Red", "Vector", [("Vector", ZERO)], [], "vec3<f32>(({0}).x)"),
    node!("Green", "Vector", [("Vector", ZERO)], [], "vec3<f32>(({0}).y)"),
    node!("Blue", "Vector", [("Vector", ZERO)], [], "vec3<f32>(({0}).z)"),
    node!("Combine", "Vector", [("R", ZERO), ("G", ZERO), ("B", ZERO)], [], "vec3<f32>(({0}).x, ({1}).x, ({2}).x)"),
    node!("Length", "Vector", [("Vector", ZERO)], [], "vec3<f32>(length({0}))"),
    node!("Distance", "Vector", [("A", ZERO), ("B", ZERO)], [], "vec3<f32>(distance({0}, {1}))"),
    node!("Dot", "Vector", [("A", ZERO), ("B", ZERO)], [], "vec3<f32>(dot({0}, {1}))"),
    node!("Cross", "Vector", [("A", ZERO), ("B", ZERO)], [], "cross({0}, {1})"),
    node!("Normalize", "Vector", [("Vector", "vec3<f32>(0.0, 1.0, 0.0)")], [], "({0} / max(length({0}), 0.00001))"),
    node!("Reflect", "Vector", [("Incident", "vec3<f32>(0.0, -1.0, 0.0)"), ("Normal", "vec3<f32>(0.0, 1.0, 0.0)")], [], "reflect({0}, normalize({1} + vec3<f32>(0.00001)))"),
    node!("Rotate coords", "Vector", [("Coords", COORD)], [("angle", 0.0, -6.3, 6.3)], "vec3<f32>(lib_rot2(({0}).xy - vec2<f32>(0.5), {p0}) + vec2<f32>(0.5), 0.0)"),
    node!("Scale offset coords", "Vector", [("Coords", COORD)], [("scale x", 1.0, 0.01, 100.0), ("scale y", 1.0, 0.01, 100.0), ("offset x", 0.0, -100.0, 100.0), ("offset y", 0.0, -100.0, 100.0)], "vec3<f32>((({0}).xy - vec2<f32>(0.5)) * vec2<f32>({p0}, {p1}) + vec2<f32>(0.5) + vec2<f32>({p2}, {p3}), 0.0)"),
    node!("Scroll coords", "Vector", [("Coords", COORD)], [("speed x", 0.1, -10.0, 10.0), ("speed y", 0.0, -10.0, 10.0)], "vec3<f32>(({0}).xy + vec2<f32>({p0}, {p1}) * globals.time.x, 0.0)"),
    node!("Polar coords", "Vector", [("Coords", COORD)], [], "vec3<f32>(lib_polar(({0}).xy), 0.0)"),
    node!("Mirror tile", "Vector", [("Coords", COORD)], [("tiles", 2.0, 0.1, 50.0)], "vec3<f32>(abs(fract(({0}).xy * {p0} * 0.5) * 2.0 - vec2<f32>(1.0)), 0.0)"),
    node!("Flipbook", "Vector", [("Coords", COORD)], [("columns", 4.0, 1.0, 32.0), ("rows", 4.0, 1.0, 32.0), ("frames per second", 8.0, 0.0, 60.0)], "vec3<f32>(lib_flipbook(({0}).xy, {p0}, {p1}, floor(globals.time.x * {p2})), 0.0)"),
    node!("Distort coords", "Vector", [("Coords", COORD), ("Amount", "vec3<f32>(0.05)")], [("scale", 4.0, 0.1, 100.0)], "({0} + vec3<f32>(lib_noise(({0}).xy * {p0}) - 0.5, lib_noise(({0}).xy * {p0} + vec2<f32>(7.3, 1.7)) - 0.5, 0.0) * ({1}).x)"),
    node!("Normal map", "Normal", [("Map", "vec3<f32>(0.5, 0.5, 1.0)"), ("UV", COORD)], [("strength", 1.0, 0.0, 4.0)], "lib_normal_map(n, in.world_pos, ({1}).xy, {0}, {p0})"),
    node!("Bump from height", "Normal", [("Height", ZERO)], [("strength", 1.0, 0.0, 10.0)], "lib_bump(({0}).x, in.world_pos, n, {p0})"),
    // ---- colour
    node!("Overlay", "Colour", [("Base", HALF), ("Blend", HALF)], [], "lib_overlay(clamp({0}, vec3<f32>(0.0), vec3<f32>(1.0)), {1})"),
    node!("Screen blend", "Colour", [("Base", HALF), ("Blend", HALF)], [], "lib_screen({0}, {1})"),
    node!("Colourize", "Colour", [("Value", HALF), ("Tint", "vec3<f32>(1.0, 0.7, 0.4)")], [], "(vec3<f32>(lib_luma({0})) * {1})"),
    node!("Ramp 3 colours", "Colour", [("Position", HALF), ("Low", "vec3<f32>(0.02, 0.03, 0.15)"), ("Middle", "vec3<f32>(0.8, 0.2, 0.3)"), ("High", "vec3<f32>(1.0, 0.9, 0.4)")], [], "mix(mix({1}, {2}, clamp(({0}).x * 2.0, 0.0, 1.0)), {3}, clamp(({0}).x * 2.0 - 1.0, 0.0, 1.0))"),
    node!("Gamma", "Colour", [("Colour", HALF)], [("gamma", 2.2, 0.1, 5.0)], "pow(max({0}, vec3<f32>(0.0)), vec3<f32>(1.0 / {p0}))"),
    node!("Exposure", "Colour", [("Colour", HALF)], [("stops", 0.0, -8.0, 8.0)], "({0} * exp2({p0}))"),
    node!("Levels", "Colour", [("Colour", HALF)], [("black", 0.0, 0.0, 1.0), ("white", 1.0, 0.0, 2.0)], "clamp(({0} - vec3<f32>({p0})) / vec3<f32>({p1} - {p0} + 0.0001), vec3<f32>(0.0), vec3<f32>(1.0))"),
    node!("Threshold", "Colour", [("Colour", HALF)], [("level", 0.5, 0.0, 1.0)], "vec3<f32>(step({p0}, lib_luma({0})))"),
    node!("Tone map", "Colour", [("Colour", HALF)], [], "lib_tonemap({0})"),
    // ---- procedural
    node!("Noise 3D", "Noise", [("Coords", WORLD)], [("scale", 2.0, 0.01, 100.0)], "vec3<f32>(fbm3({0} * {p0}, 4))"),
    node!("Ridged noise", "Noise", [("Coords", WORLD)], [("scale", 2.0, 0.01, 100.0)], "vec3<f32>(lib_ridged(({0}).xz * {p0}))"),
    node!("Turbulence", "Noise", [("Coords", WORLD)], [("scale", 2.0, 0.01, 100.0)], "vec3<f32>(lib_turbulence(({0}).xz * {p0}))"),
    node!("White noise", "Noise", [("Coords", COORD)], [("cells", 64.0, 1.0, 1000.0)], "vec3<f32>(lib_hash21(floor(({0}).xy * {p0})))"),
    node!("Sparkle", "Noise", [("Coords", COORD)], [("cells", 80.0, 4.0, 1000.0), ("density", 0.03, 0.0, 1.0)], "vec3<f32>(step(1.0 - {p1}, lib_hash21(floor(({0}).xy * {p0}))))"),
    node!("Caustics", "Procedural", [("Coords", WORLD)], [("scale", 1.5, 0.05, 50.0), ("speed", 0.6, 0.0, 10.0)], "vec3<f32>(lib_caustic(({0}).xz * {p0}, globals.time.x * {p1}))"),
    node!("Hexagons", "Procedural", [("Coords", COORD)], [("scale", 6.0, 0.5, 100.0), ("edge width", 0.06, 0.0, 0.5)], "vec3<f32>(smoothstep(0.5 - {p1}, 0.5, lib_hex(({0}).xy * {p0})))"),
    node!("Wave rings", "Procedural", [("Coords", COORD)], [("frequency", 20.0, 1.0, 200.0), ("speed", 2.0, -20.0, 20.0)], "vec3<f32>(sin(length(({0}).xy - vec2<f32>(0.5)) * {p0} - globals.time.x * {p1}) * 0.5 + 0.5)"),
    node!("Linear gradient", "Procedural", [("Coords", COORD)], [], "vec3<f32>(clamp(({0}).x, 0.0, 1.0))"),
    node!("Radial gradient", "Procedural", [("Coords", COORD)], [], "vec3<f32>(clamp(1.0 - length(({0}).xy - vec2<f32>(0.5)) * 2.0, 0.0, 1.0))"),
    node!("Angular gradient", "Procedural", [("Coords", COORD)], [], "vec3<f32>(atan2(({0}).y - 0.5, ({0}).x - 0.5) / 6.2831853 + 0.5)"),
    node!("Circle", "Procedural", [("Coords", COORD)], [("radius", 0.3, 0.0, 1.0), ("softness", 0.02, 0.0, 0.5)], "vec3<f32>(1.0 - smoothstep({p0}, {p0} + {p1} + 0.0001, length(({0}).xy - vec2<f32>(0.5))))"),
    node!("Box", "Procedural", [("Coords", COORD)], [("width", 0.6, 0.0, 1.0), ("height", 0.4, 0.0, 1.0), ("softness", 0.02, 0.0, 0.5)], "vec3<f32>(1.0 - smoothstep(0.0, {p2} + 0.0001, lib_sdf_box(({0}).xy - vec2<f32>(0.5), vec2<f32>({p0}, {p1}) * 0.5)))"),
    node!("Wood rings", "Procedural", [("Coords", WORLD)], [("scale", 4.0, 0.1, 50.0), ("wobble", 0.3, 0.0, 2.0)], "vec3<f32>(lib_wood({0} * {p0}, {p1}))"),
    node!("Marble veins", "Procedural", [("Coords", WORLD)], [("scale", 1.5, 0.05, 20.0), ("warp", 4.0, 0.0, 20.0)], "vec3<f32>(sin((({0}).x * {p0} + fbm3({0} * {p0}, 5) * {p1}) * 6.2831853) * 0.5 + 0.5)"),
];

/// Nodes that only exist for surfaces (they read the mesh, light or camera).
pub const MESH_ONLY: &[&str] = &[
    "Texture", "World position", "Normal", "Height", "Object colour", "Fresnel", "Slope mask", "Sun light", "Camera distance", "View direction", "Reflection", "Sky reflection", "Sun direction", "Sun colour",
    "Ambient light", "Shadow", "Lighting", "Camera position", "Object position", "Object random", "Screen position", "Normal map", "Bump from height", "Noise 3D", "Ridged noise", "Turbulence", "Caustics", "Wood rings",
    "Marble veins",
];

/// Nodes that only exist inside camera effects.
pub const EFFECT_ONLY: &[&str] = &["Screen color", "Pixel colour", "Pixel size", "Aspect ratio", "Blur", "Chromatic split", "Edge detect", "Pixelate", "Vignette", "Scanlines"];

/// One-line explanation of a node (tooltip in the editor).
pub fn help(kind: &str) -> &'static str {
    match kind {
        "Output" => "The material result: colour, roughness, metal, glow, opacity, occlusion and a normal.",
        "Shader output" => "The shader result. Light amount 0 = flat colour, 1 = fully lit. Vertex offset moves vertices in world space (waves, wind).",
        "Effect output" => "The final colour of the camera image.",
        "Color" => "A constant colour.",
        "Value" => "A constant number.",
        "Texture" => "Samples one of the material images (slot 0 albedo, 1 normal, 2 ORM).",
        "UV" => "Texture coordinates of the surface (0..1).",
        "World position" => "Position of the pixel in the world.",
        "Normal" => "Direction the surface faces.",
        "Height" => "World height (Y) of the pixel.",
        "Time" => "Seconds since the game started.",
        "Fresnel" => "Brighter at grazing angles (rim light, glass, water edges).",
        "Reroute" => "A tiny pass-through point to tidy long links.",
        "Comment" => "A frame with a note. Nodes inside it move with it.",
        "Inspector number" => "A number you can change per object in the Inspector (MeshRenderer user 1..8) or from a script.",
        "Time scaled" => "Time multiplied by a speed.",
        "Pulse" => "A smooth 0..1 wave over time.",
        "Screen color" => "Colour of the camera image at the given coordinates (blur, distortion).",
        "Pixel colour" => "The current pixel of the camera image.",
        "Blur" => "Cheap 5-tap blur of the camera image.",
        "Chromatic split" => "Shifts the red and blue channels apart.",
        "Edge detect" => "White where the picture changes sharply.",
        "Pixelate" => "Blocky retro look.",
        "Vignette" => "Darkens the screen edges.",
        "Scanlines" => "CRT-like horizontal lines.",
        "Mix" => "Blends A and B by Factor (0 = A, 1 = B).",
        "Branch" => "If A > B returns the third input, otherwise the fourth.",
        "Combine" => "Builds a vector from the red channels of three inputs.",
        "Normal map" => "Turns a normal-map colour into a surface direction. Connect the result to Output > Normal.",
        "Bump from height" => "Makes bumps from a height value. Connect the result to Output > Normal.",
        "Flipbook" => "Plays a grid of frames (sprite sheet) as animation.",
        "Distort coords" => "Wobbles coordinates with noise (heat haze, water).",
        "Ramp 3 colours" => "Maps 0..1 to a three-colour gradient.",
        "Voronoi" => "Cell pattern: distance to the nearest random point.",
        "Caustics" => "Moving light patterns like under water.",
        "Hexagons" => "Honeycomb pattern edges.",
        "Wood rings" => "Tree-ring pattern for wood.",
        "Marble veins" => "Wavy veins for marble.",
        "Lighting" => "Applies scene lighting (sun, shadows, lamps) to a colour.",
        "Shadow" => "1 in light, 0 in the sun shadow.",
        "Sky reflection" => "What the sky looks like in the mirror direction.",
        _ => "",
    }
}

/// Help text in the requested language (new nodes have Russian text; the rest fall back to English).
pub fn help_for(kind: &str, russian: bool) -> &'static str {
    crate::material_nodes_ext::help(kind, russian).unwrap_or_else(|| help(kind))
}
