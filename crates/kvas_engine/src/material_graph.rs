//! Node-based materials. A material is a graph of small nodes (textures, colours, maths, noise,
//! gradients...) saved as a `.kmat` file; [`to_wgsl`] turns it into a fragment shader that the
//! renderer compiles like any other custom shader. Drop a `.kmat` file on a model to use it.
//!
//! Every socket carries a `vec3` (a number is spread over all three channels), so any output can
//! feed any input. The three texture slots of a material (albedo, normal, ORM) are bound for the
//! "Texture" node.
//!
//! ```
//! use kvas_engine::material_graph::{MaterialGraph, to_wgsl};
//! let graph = MaterialGraph::default_graph();
//! let wgsl = to_wgsl(&graph).unwrap();
//! assert!(wgsl.contains("fn fs_main"));
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One input socket of a node.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Socket {
    /// Node whose output feeds this socket (`None` = use `value`).
    pub link: Option<u32>,
    /// Constant used when nothing is connected (a number is stored in all three channels).
    pub value: [f32; 3],
    /// Explicit constant override, including zero. Older files used nonzero values as overrides.
    #[serde(default)]
    pub override_default: bool,
    /// Which output of the linked node feeds this socket (0 = its main result).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub output: u8,
    /// How the linked value is converted, an index into [`crate::material_nodes_v4::CONVERSIONS`].
    #[serde(default, skip_serializing_if = "is_zero")]
    pub convert: u8,
}

fn is_zero(v: &u8) -> bool {
    *v == 0
}

impl Socket {
    /// An unconnected socket that uses the default of its spec.
    pub fn empty() -> Socket {
        Socket { link: None, value: [0.0; 3], override_default: false, output: 0, convert: 0 }
    }
}

/// What a graph produces.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GraphMode {
    /// A surface material (`.kmat`): colour, roughness, metal, glow, opacity, normal.
    #[default]
    Material,
    /// A full object shader (`.kshader`): colour, opacity, glow, lighting amount and vertex offset.
    Shader,
    /// A camera effect (`.kfx`): gets the picture and returns a new one.
    Effect,
}

impl GraphMode {
    /// Mode implied by a file name.
    pub fn from_path(path: &str) -> GraphMode {
        if path.ends_with(".kshader") {
            GraphMode::Shader
        } else if path.ends_with(".kfx") {
            GraphMode::Effect
        } else {
            GraphMode::Material
        }
    }

    /// File extension (without the dot).
    pub fn extension(self) -> &'static str {
        match self {
            GraphMode::Material => "kmat",
            GraphMode::Shader => "kshader",
            GraphMode::Effect => "kfx",
        }
    }

    /// Node type of the graph's result node.
    pub fn output_kind(self) -> &'static str {
        match self {
            GraphMode::Material => "Output",
            GraphMode::Shader => "Shader output",
            GraphMode::Effect => "Effect output",
        }
    }
}

/// Whether a node type can be used in a graph of this kind.
pub fn available(kind: &str, mode: GraphMode) -> bool {
    use crate::material_nodes::{EFFECT_ONLY, MESH_ONLY};
    if is_output(kind) {
        return kind == mode.output_kind();
    }
    match mode {
        GraphMode::Effect => !MESH_ONLY.contains(&kind) && !crate::material_nodes_ext::MESH_ONLY.contains(&kind) && !crate::material_nodes_v4::MESH_ONLY.contains(&kind),
        _ => !EFFECT_ONLY.contains(&kind),
    }
}

/// True for the three result nodes.
pub fn is_output(kind: &str) -> bool {
    matches!(kind, "Output" | "Shader output" | "Effect output")
}

/// A node of the graph.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MatNode {
    /// Unique id within the graph.
    pub id: u32,
    /// Node type name, see [`SPECS`].
    pub kind: String,
    /// Position on the canvas.
    pub pos: [f32; 2],
    /// Input sockets, in the order of the spec.
    pub inputs: Vec<Socket>,
    /// Numeric parameters, in the order of the spec.
    pub params: Vec<f32>,
    /// Note text (Comment nodes).
    #[serde(default)]
    pub text: String,
    /// Frame size on the canvas (Comment nodes).
    #[serde(default)]
    pub size: [f32; 2],
    /// Show a small live preview of this node's result inside the node (editor only).
    #[serde(default)]
    pub preview: bool,
}

/// A whole material.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct MaterialGraph {
    /// All nodes, including the single `Output` node.
    pub nodes: Vec<MatNode>,
    /// Images bound to the Texture node slots: albedo, normal map, ORM (occlusion/roughness/metallic).
    pub textures: [String; 3],
    /// Next free node id.
    pub next_id: u32,
    /// What the graph produces.
    pub mode: GraphMode,
}

impl Default for MaterialGraph {
    fn default() -> Self {
        Self::default_graph()
    }
}

/// Static description of a node type.
pub struct NodeSpec {
    /// Type name stored in files.
    pub kind: &'static str,
    /// Menu group.
    pub category: &'static str,
    /// Inputs: name and the WGSL expression used when unconnected (`vec3<f32>`).
    pub inputs: &'static [(&'static str, &'static str)],
    /// Parameters: name, default, min, max.
    pub params: &'static [(&'static str, f32, f32, f32)],
    /// WGSL expression with `{0}`.. for inputs and `{p0}`.. for parameters.
    pub expr: &'static str,
    /// The first three parameters are a colour (shown with a colour picker).
    pub color: bool,
}

const COORD: &str = "vec3<f32>(in.uv, 0.0)";
const ZERO: &str = "vec3<f32>(0.0)";

/// Every node type.
pub const SPECS: &[NodeSpec] = &[
    NodeSpec { kind: "Output", category: "Output", inputs: &[("Base color", "vec3<f32>(0.8)"), ("Roughness", "vec3<f32>(0.6)"), ("Metallic", "vec3<f32>(0.0)"), ("Emission", "vec3<f32>(0.0)"), ("Alpha", "vec3<f32>(1.0)"), ("Occlusion", "vec3<f32>(1.0)"), ("Normal", "n"), ("Height", "vec3<f32>(0.0)")], params: &[], expr: "", color: false },
    // inputs
    NodeSpec { kind: "Color", category: "Input", inputs: &[], params: &[("r", 0.8, 0.0, 1.0), ("g", 0.8, 0.0, 1.0), ("b", 0.8, 0.0, 1.0)], expr: "vec3<f32>({p0}, {p1}, {p2})", color: true },
    NodeSpec { kind: "Value", category: "Input", inputs: &[], params: &[("value", 0.5, -100.0, 100.0)], expr: "vec3<f32>({p0})", color: false },
    NodeSpec { kind: "Texture", category: "Input", inputs: &[("UV", COORD)], params: &[("slot (0 albedo, 1 normal, 2 ORM)", 0.0, 0.0, 2.0), ("tiling", 1.0, 0.01, 100.0)], expr: "", color: false },
    NodeSpec { kind: "UV", category: "Geometry", inputs: &[], params: &[], expr: "vec3<f32>(in.uv, 0.0)", color: false },
    NodeSpec { kind: "World position", category: "Geometry", inputs: &[], params: &[], expr: "in.world_pos", color: false },
    NodeSpec { kind: "Normal", category: "Geometry", inputs: &[], params: &[], expr: "n", color: false },
    NodeSpec { kind: "Height", category: "Geometry", inputs: &[], params: &[], expr: "vec3<f32>(in.world_pos.y)", color: false },
    NodeSpec { kind: "Time", category: "Input", inputs: &[], params: &[], expr: "vec3<f32>(globals.time.x)", color: false },
    NodeSpec { kind: "Object colour", category: "Input", inputs: &[], params: &[], expr: "object.color.rgb", color: false },
    NodeSpec { kind: "Fresnel", category: "Light", inputs: &[("Power", "vec3<f32>(3.0)")], params: &[], expr: "vec3<f32>(pow(1.0 - max(dot(n, view), 0.0), max({0}.x, 0.01)))", color: false },
    // maths
    NodeSpec { kind: "Add", category: "Math", inputs: &[("A", ZERO), ("B", ZERO)], params: &[], expr: "({0} + {1})", color: false },
    NodeSpec { kind: "Subtract", category: "Math", inputs: &[("A", ZERO), ("B", ZERO)], params: &[], expr: "({0} - {1})", color: false },
    NodeSpec { kind: "Multiply", category: "Math", inputs: &[("A", "vec3<f32>(1.0)"), ("B", "vec3<f32>(1.0)")], params: &[], expr: "({0} * {1})", color: false },
    NodeSpec { kind: "Divide", category: "Math", inputs: &[("A", "vec3<f32>(1.0)"), ("B", "vec3<f32>(1.0)")], params: &[], expr: "({0} / select(vec3<f32>(0.0001), {1}, abs({1}) >= vec3<f32>(0.0001)))", color: false },
    NodeSpec { kind: "Mix", category: "Math", inputs: &[("A", ZERO), ("B", "vec3<f32>(1.0)"), ("Factor", "vec3<f32>(0.5)")], params: &[], expr: "mix({0}, {1}, clamp({2}.x, 0.0, 1.0))", color: false },
    NodeSpec { kind: "Power", category: "Math", inputs: &[("Value", "vec3<f32>(0.5)"), ("Exponent", "vec3<f32>(2.0)")], params: &[], expr: "pow(max({0}, vec3<f32>(0.0)), {1})", color: false },
    NodeSpec { kind: "Clamp", category: "Math", inputs: &[("Value", ZERO)], params: &[("min", 0.0, -100.0, 100.0), ("max", 1.0, -100.0, 100.0)], expr: "clamp({0}, vec3<f32>({p0}), vec3<f32>({p1}))", color: false },
    NodeSpec { kind: "One minus", category: "Math", inputs: &[("Value", ZERO)], params: &[], expr: "(vec3<f32>(1.0) - {0})", color: false },
    NodeSpec { kind: "Sine", category: "Math", inputs: &[("Value", ZERO)], params: &[("frequency", 1.0, 0.0, 100.0)], expr: "(sin({0} * {p0}) * 0.5 + vec3<f32>(0.5))", color: false },
    NodeSpec { kind: "Smoothstep", category: "Math", inputs: &[("Value", "vec3<f32>(0.5)")], params: &[("edge 0", 0.3, -10.0, 10.0), ("edge 1", 0.7, -10.0, 10.0)], expr: "smoothstep(vec3<f32>({p0}), vec3<f32>({p1}), {0})", color: false },
    NodeSpec { kind: "Abs", category: "Math", inputs: &[("Value", ZERO)], params: &[], expr: "abs({0})", color: false },
    NodeSpec { kind: "Min", category: "Math", inputs: &[("A", ZERO), ("B", ZERO)], params: &[], expr: "min({0}, {1})", color: false },
    NodeSpec { kind: "Max", category: "Math", inputs: &[("A", ZERO), ("B", ZERO)], params: &[], expr: "max({0}, {1})", color: false },
    // colour
    NodeSpec { kind: "Hue shift", category: "Colour", inputs: &[("Colour", "vec3<f32>(0.8)")], params: &[("turns", 0.0, -1.0, 1.0)], expr: "lib_hue_shift({0}, {p0})", color: false },
    NodeSpec { kind: "Saturation", category: "Colour", inputs: &[("Colour", "vec3<f32>(0.8)")], params: &[("amount", 1.0, 0.0, 3.0)], expr: "lib_saturation({0}, {p0})", color: false },
    NodeSpec { kind: "Contrast", category: "Colour", inputs: &[("Colour", "vec3<f32>(0.5)")], params: &[("amount", 1.0, 0.0, 3.0)], expr: "lib_contrast({0}, {p0})", color: false },
    NodeSpec { kind: "Brightness", category: "Colour", inputs: &[("Colour", "vec3<f32>(0.5)")], params: &[("scale", 1.0, 0.0, 10.0)], expr: "({0} * {p0})", color: false },
    NodeSpec { kind: "Gradient", category: "Colour", inputs: &[("Position", "vec3<f32>(0.5)"), ("From", "vec3<f32>(0.05, 0.1, 0.3)"), ("To", "vec3<f32>(0.9, 0.6, 0.2)")], params: &[], expr: "mix({1}, {2}, clamp({0}.x, 0.0, 1.0))", color: false },
    NodeSpec { kind: "To grey", category: "Colour", inputs: &[("Colour", "vec3<f32>(0.5)")], params: &[], expr: "vec3<f32>(lib_luma({0}))", color: false },
    // procedural
    NodeSpec { kind: "Noise", category: "Noise", inputs: &[("Coords", "in.world_pos")], params: &[("scale", 2.0, 0.01, 100.0)], expr: "vec3<f32>(lib_fbm({0}.xz * {p0}))", color: false },
    NodeSpec { kind: "Checker", category: "Procedural", inputs: &[("Coords", COORD)], params: &[("scale", 8.0, 1.0, 200.0)], expr: "vec3<f32>(abs((floor({0}.x * {p0}) + floor({0}.y * {p0})) % 2.0))", color: false },
    NodeSpec { kind: "Stripes", category: "Procedural", inputs: &[("Coords", COORD)], params: &[("scale", 10.0, 1.0, 200.0), ("sharpness", 0.5, 0.0, 1.0)], expr: "vec3<f32>(smoothstep(0.5 - {p1} * 0.5, 0.5 + {p1} * 0.5 + 0.001, abs(fract({0}.x * {p0}) - 0.5) * 2.0))", color: false },
    NodeSpec { kind: "Voronoi", category: "Noise", inputs: &[("Coords", "in.world_pos")], params: &[("scale", 3.0, 0.01, 100.0)], expr: "vec3<f32>(lib_voronoi({0}.xz * {p0}).x)", color: false },
    NodeSpec { kind: "Cell colour", category: "Noise", inputs: &[("Coords", "in.world_pos")], params: &[("scale", 3.0, 0.01, 100.0)], expr: "vec3<f32>(lib_voronoi({0}.xz * {p0}).y, lib_voronoi({0}.xz * {p0} + 11.0).y, lib_voronoi({0}.xz * {p0} + 23.0).y)", color: false },
    NodeSpec { kind: "Bricks", category: "Procedural", inputs: &[("Coords", COORD)], params: &[("scale", 6.0, 1.0, 100.0), ("mortar", 0.06, 0.0, 0.3)], expr: "vec3<f32>(smoothstep({p1}, {p1} + 0.02, min(min(fract({0}.x * {p0} + 0.5 * (floor({0}.y * {p0} * 2.0) % 2.0)), 1.0 - fract({0}.x * {p0} + 0.5 * (floor({0}.y * {p0} * 2.0) % 2.0))), min(fract({0}.y * {p0} * 2.0), 1.0 - fract({0}.y * {p0} * 2.0)))))", color: false },
    NodeSpec { kind: "Slope mask", category: "Geometry", inputs: &[], params: &[("from", 0.5, 0.0, 1.0), ("to", 0.9, 0.0, 1.0)], expr: "vec3<f32>(smoothstep({p0}, {p1} + 0.001, n.y))", color: false },
    NodeSpec { kind: "Sun light", category: "Light", inputs: &[], params: &[], expr: "vec3<f32>(max(dot(n, normalize(globals.sun_dir.xyz)), 0.0))", color: false },
    NodeSpec { kind: "Camera distance", category: "Geometry", inputs: &[], params: &[("range", 50.0, 1.0, 1000.0)], expr: "vec3<f32>(clamp(distance(globals.camera_pos.xyz, in.world_pos) / {p0}, 0.0, 1.0))", color: false },
    NodeSpec { kind: "Fract", category: "Math", inputs: &[("Value", "vec3<f32>(0.5)")], params: &[], expr: "fract({0})", color: false },
    NodeSpec { kind: "Step", category: "Math", inputs: &[("Value", "vec3<f32>(0.5)")], params: &[("edge", 0.5, 0.0, 1.0)], expr: "step(vec3<f32>({p0}), {0})", color: false },
    NodeSpec { kind: "Remap", category: "Math", inputs: &[("Value", "vec3<f32>(0.5)")], params: &[("in min", 0.0, -10.0, 10.0), ("in max", 1.0, -10.0, 10.0), ("out min", 0.0, -10.0, 10.0), ("out max", 1.0, -10.0, 10.0)], expr: "(({0}) - vec3<f32>({p0})) / vec3<f32>({p1} - {p0} + 0.00001) * vec3<f32>({p3} - {p2}) + vec3<f32>({p2})", color: false },
    NodeSpec { kind: "Dots", category: "Procedural", inputs: &[("Coords", COORD)], params: &[("scale", 10.0, 1.0, 200.0), ("size", 0.3, 0.01, 0.5)], expr: "vec3<f32>(1.0 - smoothstep({p1}, {p1} + 0.05, length(fract({0}.xy * {p0}) - vec2<f32>(0.5))))", color: false },
];

/// Looks a node type up by name.
pub fn spec(kind: &str) -> Option<&'static NodeSpec> {
    all_specs().find(|s| s.kind == kind)
}

/// Every node type of both lists.
pub fn all_specs() -> impl Iterator<Item = &'static NodeSpec> {
    SPECS.iter().chain(crate::material_nodes::MORE_SPECS.iter()).chain(crate::material_nodes_ext::EXT_SPECS.iter()).chain(crate::material_nodes_v4::V4_SPECS.iter())
}

impl MaterialGraph {
    /// A new graph with only an Output node (grey, rough).
    pub fn default_graph() -> Self {
        Self::default_graph_for(GraphMode::Material)
    }

    /// A starter graph for the given kind: the result node plus one connected node.
    pub fn default_graph_for(mode: GraphMode) -> Self {
        let mut g = MaterialGraph { nodes: Vec::new(), textures: Default::default(), next_id: 1, mode };
        g.add_node(mode.output_kind(), [420.0, 80.0]);
        let first = match mode {
            GraphMode::Effect => g.add_node("Screen color", [60.0, 60.0]),
            _ => g.add_node("Color", [60.0, 60.0]),
        };
        g.connect(first, 1, 0);
        g
    }

    /// Adds a node of a known type with its default values.
    pub fn add_node(&mut self, kind: &str, pos: [f32; 2]) -> u32 {
        let spec = spec(kind).expect("known node type");
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(MatNode {
            id,
            kind: kind.to_string(),
            pos,
            inputs: spec.inputs.iter().map(|_| Socket::empty()).collect(),
            params: spec.params.iter().map(|p| p.1).collect(),
            text: if kind == "Comment" { "Comment".into() } else { String::new() },
            size: if kind == "Comment" { [260.0, 160.0] } else { [0.0, 0.0] },
            preview: matches!(spec.category, "Noise" | "Procedural"),
        });
        id
    }

    /// Connects `from`'s main output to input `input` of node `to`. Returns false for self-links or cycles.
    pub fn connect(&mut self, from: u32, to: u32, input: usize) -> bool {
        self.connect_output(from, 0, to, input, 0)
    }

    /// Connects output `output` of `from` to input `input` of `to`, converting with `convert`
    /// (see [`crate::material_nodes_v4::CONVERSIONS`]).
    pub fn connect_output(&mut self, from: u32, output: u8, to: u32, input: usize, convert: u8) -> bool {
        if !self.nodes.iter().any(|n| n.id == from && !is_output(&n.kind) && n.kind != "Comment") {
            return false;
        }
        if from == to || self.reaches(from, to) {
            return false;
        }
        match self.nodes.iter_mut().find(|n| n.id == to).and_then(|n| n.inputs.get_mut(input)) {
            Some(socket) => {
                socket.link = Some(from);
                socket.output = output;
                socket.convert = convert;
                true
            }
            None => false,
        }
    }

    /// Whether `target` is reachable going upstream from `start` (a link would close a loop).
    fn reaches(&self, start: u32, target: u32) -> bool {
        let mut stack = vec![start];
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = stack.pop() {
            if id == target {
                return true;
            }
            if !seen.insert(id) {
                continue;
            }
            if let Some(n) = self.nodes.iter().find(|n| n.id == id) {
                stack.extend(n.inputs.iter().filter_map(|s| s.link));
            }
        }
        false
    }

    /// Removes a node (the Output node stays) and every link that used it.
    pub fn remove_node(&mut self, id: u32) {
        if self.nodes.iter().any(|n| n.id == id && is_output(&n.kind)) {
            return;
        }
        self.nodes.retain(|n| n.id != id);
        for n in &mut self.nodes {
            for s in &mut n.inputs {
                if s.link == Some(id) {
                    s.link = None;
                }
            }
        }
    }

    /// Parses a `.kmat` file.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let mut g: MaterialGraph = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if g.nodes.iter().any(|n| n.id >= u32::MAX - 1) { return Err("node id space is exhausted".into()); }
        // Files written by older or hand-edited versions may have too few sockets or parameters.
        for n in &mut g.nodes {
            if let Some(spec) = spec(&n.kind) {
                n.inputs.resize(spec.inputs.len(), Socket::empty());
                while n.params.len() < spec.params.len() {
                    n.params.push(spec.params[n.params.len()].1);
                }
            }
        }
        for n in &mut g.nodes {
            if n.kind == "Comment" && n.size == [0.0, 0.0] {
                n.size = [260.0, 160.0];
            }
        }
        g.next_id = g.next_id.max(g.nodes.iter().map(|n| n.id + 1).max().unwrap_or(1));
        Ok(g)
    }

    /// Serialises to pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Checks graph identity, sockets and links before generating executable shader code.
    /// Unconnected nodes are checked too, so saving an invalid graph cannot hide broken links.
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = std::collections::HashSet::new();
        let mut outputs = 0;
        for node in &self.nodes {
            if !ids.insert(node.id) { return Err(format!("duplicate node id {}", node.id)); }
            let s = spec(&node.kind).ok_or_else(|| format!("unknown node type '{}'", node.kind))?;
            if !available(&node.kind, self.mode) { return Err(format!("node '{}' is unavailable in {:?} graphs", node.kind, self.mode)); }
            outputs += usize::from(is_output(&node.kind));
            if node.inputs.len() != s.inputs.len() || node.params.len() < s.params.len() {
                return Err(format!("node {} ({}) has invalid sockets or parameters", node.id, node.kind));
            }
            if node.params.iter().chain(node.inputs.iter().flat_map(|s| &s.value)).any(|v| !v.is_finite()) {
                return Err(format!("node {} contains a non-finite value", node.id));
            }
        }
        if outputs != 1 { return Err(format!("expected one '{}' node, found {outputs}", self.mode.output_kind())); }
        let nodes: HashMap<_, _> = self.nodes.iter().map(|n| (n.id, n)).collect();
        // Iterative DFS keeps malformed or hand-edited graphs from overflowing the stack.
        let mut state = HashMap::new();
        for node in &self.nodes {
            let mut stack = vec![(node.id, false, 0usize)];
            while let Some((id, leave, depth)) = stack.pop() {
                if leave { state.insert(id, 2); continue; }
                match state.get(&id) {
                    Some(1) => return Err(format!("the graph has a loop at node {id}")),
                    Some(2) => continue,
                    _ => {}
                }
                if depth > 256 { return Err("graph exceeds the 256-node dependency depth limit".into()); }
                let n = nodes.get(&id).ok_or_else(|| format!("a link points to missing node {id}"))?;
                state.insert(id, 1);
                stack.push((id, true, depth));
                for link in n.inputs.iter().filter_map(|s| s.link) {
                    let source = nodes.get(&link).ok_or_else(|| format!("node {id} links to missing node {link}"))?;
                    if is_output(&source.kind) || source.kind == "Comment" { return Err(format!("node {link} has no output socket")); }
                    stack.push((link, false, depth + 1));
                }
            }
        }
        Ok(())
    }
}

fn f(v: f32) -> String {
    if v.is_finite() { format!("{v:?}") } else { "0.0".into() }
}

struct Gen<'a> {
    graph: &'a MaterialGraph,
    lines: Vec<String>,
    done: HashMap<u32, String>,
    visiting: Vec<u32>,
    /// Variables of the extra outputs already generated, keyed by `id | output << 32`.
    extra: HashMap<u64, String>,
    /// The (named) input values of every generated node, for its extra outputs.
    inputs_of: HashMap<u32, Vec<String>>,
    /// Generating the vertex stage (textures are not available there).
    vertex: bool,
}

/// Adapts a surface expression to the camera-effect context (where `in.uv`, `time` ... differ).
fn for_effect(expr: &str) -> String {
    expr.replace("in.world_pos", "vec3<f32>(uv, 0.0)").replace("in.uv", "uv").replace("globals.time.x", "time")
}

impl Gen<'_> {
    fn ctx(&self, expr: String) -> String {
        if self.graph.mode == GraphMode::Effect {
            for_effect(&expr)
        } else {
            expr
        }
    }

    fn input(&mut self, node: &MatNode, index: usize, spec: &NodeSpec) -> Result<String, String> {
        let socket = node.inputs.get(index);
        match socket.and_then(|s| s.link) {
            Some(id) => {
                let socket = socket.unwrap();
                let value = self.eval_output(id, socket.output as usize)?;
                match crate::material_nodes_v4::CONVERSIONS.get(socket.convert as usize) {
                    Some((_, conv)) if socket.convert > 0 => Ok(conv.replace("{v}", &value)),
                    _ => Ok(value),
                }
            }
            None => {
                // A socket the user typed a value into overrides the default expression.
                match socket.map(|s| s.value) {
                    Some(v) if socket.is_some_and(|s| s.override_default) || v != [0.0; 3] => Ok(format!("vec3<f32>({}, {}, {})", f(v[0]), f(v[1]), f(v[2]))),
                    _ => Ok(self.ctx(spec.inputs.get(index).map_or(ZERO, |i| i.1).to_string())),
                }
            }
        }
    }

    /// The value of output `output` of node `id` (0 is the main result).
    fn eval_output(&mut self, id: u32, output: usize) -> Result<String, String> {
        let main = self.eval(id)?;
        let node = self.graph.nodes.iter().find(|n| n.id == id).ok_or("a link points to a missing node")?;
        // Output 0 is the main result, unless the node's first named output reshapes it
        // (Voronoi "Factor", Brick "Colour"...).
        let Some((_, template)) = crate::material_nodes_v4::outputs(&node.kind).get(output) else { return Ok(main) };
        if *template == "{v}" {
            return Ok(main);
        }
        let key = id as u64 | ((output as u64) << 32);
        if let Some(v) = self.extra.get(&key) {
            return Ok(v.clone());
        }
        let mut e = template.replace("{v}", &main);
        let inputs = self.inputs_of.get(&id).cloned().unwrap_or_default();
        for (i, input) in inputs.iter().enumerate() {
            e = e.replace(&format!("{{{i}}}"), &format!("({input})"));
        }
        for (i, p) in node.params.iter().enumerate() {
            e = e.replace(&format!("{{p{i}}}"), &f(*p));
        }
        let e = self.ctx(e);
        let name = format!("v{id}_{output}");
        self.lines.push(format!("    let {name} = {e};"));
        self.extra.insert(key, name.clone());
        Ok(name)
    }

    fn eval(&mut self, id: u32) -> Result<String, String> {
        if let Some(v) = self.done.get(&id) {
            return Ok(v.clone());
        }
        if self.visiting.contains(&id) {
            return Err("the graph has a loop".into());
        }
        let node = self.graph.nodes.iter().find(|n| n.id == id).ok_or("a link points to a missing node")?.clone();
        let spec = spec(&node.kind).ok_or_else(|| format!("unknown node type '{}'", node.kind))?;
        if !available(&node.kind, self.graph.mode) {
            return Err(format!("the node '{}' cannot be used in this kind of graph", node.kind));
        }
        if self.vertex && (matches!(node.kind.as_str(), "Texture" | "Normal map" | "Bump from height" | "Shadow" | "Lighting" | "Sky reflection" | "Screen position") || crate::material_nodes_ext::FRAGMENT_ONLY.contains(&node.kind.as_str()) || crate::material_nodes_v4::FRAGMENT_ONLY.contains(&node.kind.as_str())) {
            return Err(format!("'{}' requires fragment-stage sampling or derivatives and cannot feed Vertex offset", node.kind));
        }
        self.visiting.push(id);
        let mut inputs = Vec::new();
        for i in 0..spec.inputs.len() {
            inputs.push(self.input(&node, i, spec)?);
        }
        self.visiting.pop();
        // Inputs get short names, so extra outputs that reuse them do not repeat long expressions.
        let mut short = Vec::new();
        for (i, input) in inputs.iter().enumerate() {
            if input.starts_with('v') && input[1..].chars().all(|c| c.is_ascii_digit() || c == '_') {
                short.push(input.clone());
            } else {
                let name = format!("i{id}_{i}");
                self.lines.push(format!("    let {name} = {input};"));
                short.push(name);
            }
        }
        let inputs = short;
        self.inputs_of.insert(id, inputs.clone());
        let expr = if node.kind == "Texture" {
            let slot = node.params.first().copied().unwrap_or(0.0).round() as i32;
            let tex = match slot {
                1 => "normal_tex",
                2 => "orm_tex",
                _ => "albedo_tex",
            };
            let tiling = f(node.params.get(1).copied().unwrap_or(1.0));
            format!("textureSample({tex}, albedo_smp, ({}).xy * {tiling}).rgb", inputs[0])
        } else if node.kind == "Inspector number" {
            let slot = node.params.first().copied().unwrap_or(0.0).round().clamp(0.0, 7.0) as usize;
            if self.graph.mode == GraphMode::Effect && slot > 3 { return Err("camera effect Inspector number has only slots 0..3 (params.xyzw)".into()); }
            let default = f(node.params.get(1).copied().unwrap_or(1.0));
            let comp = ["x", "y", "z", "w"][slot % 4];
            let source = if self.graph.mode == GraphMode::Effect { format!("params.{comp}") } else { format!("object.user{}.{comp}", slot / 4) };
            if node.params.get(2).copied().unwrap_or(1.0) >= 0.5 { format!("vec3<f32>(select({source}, {default}, {source} == 0.0))") }
            else { format!("vec3<f32>({source})") }
        } else {
            let mut e = spec.expr.to_string();
            if e.contains("{tex}") {
                let slot = node.params.first().copied().unwrap_or(0.0).round() as i32;
                e = e.replace("{tex}", match slot { 1 => "normal_tex", 2 => "orm_tex", _ => "albedo_tex" });
            }
            for (i, input) in inputs.iter().enumerate() {
                e = e.replace(&format!("{{{i}}}"), &format!("({input})"));
            }
            for (i, p) in node.params.iter().enumerate() {
                e = e.replace(&format!("{{p{i}}}"), &f(*p));
            }
            self.ctx(e)
        };
        let name = format!("v{id}");
        self.lines.push(format!("    let {name} = {expr};"));
        self.done.insert(id, name.clone());
        Ok(name)
    }
}

fn new_gen(graph: &MaterialGraph, vertex: bool) -> Gen<'_> {
    Gen { graph, lines: Vec::new(), done: HashMap::new(), visiting: Vec::new(), extra: HashMap::new(), inputs_of: HashMap::new(), vertex }
}

/// Generates the shader of a graph (the prelude and the library functions are added by the renderer):
/// a mesh shader for materials and shaders, an `effect` function for camera effects.
pub fn to_wgsl(graph: &MaterialGraph) -> Result<String, String> {
    graph.validate()?;
    let kind = graph.mode.output_kind();
    let output = graph.nodes.iter().find(|n| n.kind == kind).ok_or_else(|| format!("the graph has no '{kind}' node"))?.clone();
    let out_spec = spec(kind).unwrap();
    let mut gen = new_gen(graph, false);
    let mut results = Vec::new();
    // The vertex offset is evaluated in its own stage.
    let skip = if graph.mode == GraphMode::Shader { Some(5) } else { None };
    for i in 0..out_spec.inputs.len() {
        if Some(i) == skip {
            results.push(String::new());
            continue;
        }
        results.push(gen.input(&output, i, out_spec)?);
    }
    let body = gen.lines.join("\n");
    match graph.mode {
        GraphMode::Material => Ok(format!(
            "// Generated from a .kmat material graph.
@vertex
fn vs_main(v: VertexIn) -> VsOut {{ return standard_vertex(v); }}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {{
    let n0 = normalize(in.normal);
    let n = n0;
    let view = normalize(globals.camera_pos.xyz - in.world_pos);
{body}
    let albedo = max({0}, vec3<f32>(0.0));
    let rough = clamp(({1}).x, 0.04, 1.0);
    let metal = clamp(({2}).x, 0.0, 1.0);
    let emission = max({3}, vec3<f32>(0.0));
    let alpha = clamp(({4}).x, 0.0, 1.0);
    let occlusion = clamp(({5}).x, 0.0, 1.0);
    let bumped = lib_bump(({7}).x, in.world_pos, n, 1.0);
    let nrm = normalize({6} + (bumped - n) + vec3<f32>(0.00001));
    if (alpha < 0.003) {{ discard; }}
    let lit = lighting_pbr(in.world_pos, nrm, albedo, rough, metal, occlusion) + emission;
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), alpha * object.color.a);
}}
",
            results[0], results[1], results[2], results[3], results[4], results[5], results[6], results[7]
        )),
        GraphMode::Shader => {
            // Vertex stage: runs the nodes feeding "Vertex offset" with the standard vertex output as `in`.
            let mut vgen = new_gen(graph, true);
            let offset = vgen.input(&output, 5, out_spec)?;
            let vbody = vgen.lines.join("\n");
            Ok(format!(
                "// Generated from a .kshader graph.
@vertex
fn vs_main(v: VertexIn) -> VsOut {{
    var in = standard_vertex(v);
    let n = in.normal;
    let view = normalize(globals.camera_pos.xyz - in.world_pos);
{vbody}
    in.world_pos = in.world_pos + {offset};
    in.clip = globals.view_proj * vec4<f32>(in.world_pos, 1.0);
    return in;
}}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {{
    let n = normalize(in.normal);
    let view = normalize(globals.camera_pos.xyz - in.world_pos);
{body}
    let base = max({0}, vec3<f32>(0.0));
    let alpha = clamp(({1}).x, 0.0, 1.0);
    let emission = max({2}, vec3<f32>(0.0));
    let amount = clamp(({3}).x, 0.0, 1.0);
    let rough = clamp(({4}).x, 0.04, 1.0);
    let lit = mix(base, lighting(in.world_pos, n, base, rough), amount) + emission;
    if (alpha < 0.003) {{ discard; }}
    return vec4<f32>(to_display_at(apply_fog(lit, in.world_pos), in.clip), alpha * object.color.a);
}}
",
                results[0], results[1], results[2], results[3], results[4]
            ))
        }
        GraphMode::Effect => Ok(format!(
            "// Generated from a .kfx camera effect graph.
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {{
{body}
    return max({0}, vec3<f32>(0.0));
}}
",
            results[0]
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_zero_and_legacy_defaults_round_trip() {
        let mut g = MaterialGraph::default_graph();
        g.nodes[0].inputs[1].override_default = true;
        let src = to_wgsl(&MaterialGraph::from_json(&g.to_json()).unwrap()).unwrap();
        assert!(src.contains("let rough = clamp((vec3<f32>(0.0, 0.0, 0.0)).x"));
        g.nodes[0].inputs[1].override_default = false;
        assert!(to_wgsl(&g).unwrap().contains("clamp((vec3<f32>(0.6)).x"));
    }

    #[test]
    fn malformed_links_and_duplicate_ids_report_errors() {
        let mut g = MaterialGraph::default_graph();
        assert!(!g.connect(999, 1, 0));
        assert!(!g.connect(1, 2, 0));
        g.nodes[0].inputs[0].link = Some(999);
        assert!(to_wgsl(&g).unwrap_err().contains("missing node"));
        g.nodes[0].inputs[0].link = Some(2);
        g.nodes[1].id = 1;
        assert!(to_wgsl(&g).unwrap_err().contains("duplicate node id"));
    }

    #[test]
    fn hand_edited_cycles_and_vertex_derivatives_are_rejected() {
        let mut g = MaterialGraph::default_graph_for(GraphMode::Shader);
        let id = g.add_node("Bump from height", [0.0; 2]);
        assert!(g.connect(id, 1, 5));
        assert!(to_wgsl(&g).unwrap_err().contains("fragment-stage"));
        let a = g.add_node("Add", [0.0; 2]);
        let b = g.add_node("Add", [0.0; 2]);
        g.nodes.iter_mut().find(|n| n.id == a).unwrap().inputs[0].link = Some(b);
        g.nodes.iter_mut().find(|n| n.id == b).unwrap().inputs[0].link = Some(a);
        assert!(g.validate().unwrap_err().contains("loop"));
    }

    #[test]
    fn graph_with_every_node_generates() {
        for mode in [GraphMode::Material, GraphMode::Shader, GraphMode::Effect] {
            for spec in all_specs().filter(|s| !is_output(s.kind) && available(s.kind, mode)) {
                let mut g = MaterialGraph::default_graph_for(mode);
                let id = g.add_node(spec.kind, [0.0, 0.0]);
                g.connect(id, 1, 0);
                let wgsl = to_wgsl(&g).unwrap_or_else(|e| panic!("{:?} {}: {e}", mode, spec.kind));
                assert!(wgsl.contains("fn "), "{}", spec.kind);
            }
        }
    }

    #[test]
    fn loops_and_json_round_trip() {
        let mut g = MaterialGraph::default_graph();
        let a = g.add_node("Add", [0.0, 0.0]);
        let b = g.add_node("Multiply", [0.0, 0.0]);
        assert!(g.connect(a, b, 0));
        assert!(!g.connect(b, a, 0), "a loop must be refused");
        let back = MaterialGraph::from_json(&g.to_json()).unwrap();
        assert_eq!(back, g);
    }
}
