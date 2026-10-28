//! Built-in procedural materials. Every preset is an ordinary node graph (open it in the node editor
//! and change anything). The editor installs them into `assets/materials/library/` of every project.
//!
//! The files are generated: `cargo run -p kvas_engine --example gen_material_library`.
//!
//! ```
//! let all = kvas_engine::material_presets::library();
//! assert!(all.len() >= 40);
//! assert!(all.iter().all(|p| kvas_engine::material_graph::to_wgsl(&p.graph).is_ok()));
//! ```

use crate::material_graph::{GraphMode, MaterialGraph};

/// One shipped material.
pub struct Preset {
    /// Folder inside the library (`Stone`, `Metal`...).
    pub category: &'static str,
    /// File name without extension.
    pub name: &'static str,
    /// The node graph.
    pub graph: MaterialGraph,
}

/// Output sockets of the Output node.
const BASE: usize = 0;
const ROUGH: usize = 1;
const METAL: usize = 2;
const EMIT: usize = 3;
const ALPHA: usize = 4;
const NORMAL: usize = 6;

struct B {
    g: MaterialGraph,
    out: u32,
    col: [f32; 8],
}

impl B {
    fn new() -> Self {
        let mut g = MaterialGraph { nodes: Vec::new(), textures: Default::default(), next_id: 1, mode: GraphMode::Material };
        let out = g.add_node("Output", [1100.0, 120.0]);
        B { g, out, col: [0.0; 8] }
    }

    /// Adds a node in a column (0 = leftmost); rows are stacked automatically.
    fn node(&mut self, kind: &str, column: usize) -> u32 {
        let y = self.col[column.min(7)];
        self.col[column.min(7)] += 190.0;
        self.g.add_node(kind, [40.0 + column as f32 * 250.0, 30.0 + y])
    }

    fn set(&mut self, id: u32, params: &[f32]) -> u32 {
        let n = self.g.nodes.iter_mut().find(|n| n.id == id).unwrap();
        for (i, v) in params.iter().enumerate() {
            n.params[i] = *v;
        }
        id
    }

    fn value(&mut self, id: u32, input: usize, v: [f32; 3]) {
        let n = self.g.nodes.iter_mut().find(|n| n.id == id).unwrap();
        n.inputs[input].value = v;
        n.inputs[input].override_default = true;
    }

    fn link(&mut self, from: u32, to: u32, input: usize) {
        assert!(self.g.connect(from, to, input), "link {from} -> {to}.{input}");
    }

    fn out(&mut self, from: u32, input: usize) {
        self.link(from, self.out, input);
    }

    fn out_value(&mut self, input: usize, v: f32) {
        let out = self.out;
        self.value(out, input, [v; 3]);
    }

    /// Four-stop colour ramp driven by `source`.
    fn ramp(&mut self, source: u32, stops: [[f32; 3]; 4], pos: [f32; 4], column: usize) -> u32 {
        let r = self.node("Color ramp", column);
        self.set(r, &pos);
        for (i, s) in stops.iter().enumerate() {
            self.value(r, i + 1, *s);
        }
        self.link(source, r, 0);
        r
    }

    fn map_range(&mut self, source: u32, lo: f32, hi: f32, column: usize) -> u32 {
        let m = self.node("Map range", column);
        self.set(m, &[0.0, 1.0, lo, hi]);
        self.link(source, m, 0);
        m
    }

    fn bump(&mut self, height: u32, strength: f32, column: usize) -> u32 {
        let b = self.node("Bump from height", column);
        self.set(b, &[strength]);
        self.link(height, b, 0);
        self.out(b, NORMAL);
        b
    }

    fn fractal(&mut self, scale: f32, octaves: f32, kind: f32, column: usize) -> u32 {
        let n = self.node("Fractal noise", column);
        self.set(n, &[scale, octaves, 2.0, 0.5, kind]);
        n
    }
}

/// A rough natural surface: layered noise coloured by a ramp, with matching roughness and bump.
#[allow(clippy::too_many_arguments)]
fn natural(stops: [[f32; 3]; 4], pos: [f32; 4], scale: f32, kind: f32, rough: (f32, f32), metal: f32, bump: f32) -> MaterialGraph {
    let mut b = B::new();
    let big = b.fractal(scale, 5.0, kind, 0);
    let fine = b.fractal(scale * 7.0, 3.0, 0.0, 0);
    let ramp = b.ramp(big, stops, pos, 1);
    let detail = b.node("Blend colours", 2);
    b.set(detail, &[1.0]);
    b.link(ramp, detail, 0);
    let grain = b.map_range(fine, 0.8, 1.15, 1);
    b.link(grain, detail, 1);
    b.out(detail, BASE);
    let r = b.map_range(big, rough.0, rough.1, 2);
    b.out(r, ROUGH);
    b.out_value(METAL, metal);
    // Bump from the smooth layer only: derivatives of fine noise look blocky at 2x2 pixels.
    b.bump(big, bump * 0.5, 3);
    b.g
}

fn metal_brushed(color: [f32; 3], dark: [f32; 3], rough: f32, scratch: f32) -> MaterialGraph {
    let mut b = B::new();
    let s = b.node("Scratches", 0);
    b.set(s, &[scratch, 0.0]);
    let n = b.fractal(3.0, 4.0, 0.0, 0);
    let mix = b.node("Mix", 1);
    b.value(mix, 0, color);
    b.value(mix, 1, dark);
    let soft = b.map_range(s, 0.0, 0.45, 1);
    b.link(soft, mix, 2);
    b.out(mix, BASE);
    let r = b.map_range(n, rough * 0.7, rough * 1.3, 1);
    b.out(r, ROUGH);
    b.out_value(METAL, 1.0);
    b.g
}

fn emissive(name_ramp: [[f32; 3]; 4], pos: [f32; 4], scale: f32, kind: f32, intensity: f32, base: [f32; 3], rough: f32) -> MaterialGraph {
    let mut b = B::new();
    let n = b.fractal(scale, 5.0, kind, 0);
    let ramp = b.ramp(n, name_ramp, pos, 1);
    let glow = b.node("Multiply", 2);
    b.link(ramp, glow, 0);
    b.value(glow, 1, [intensity; 3]);
    b.out(glow, EMIT);
    let dark = b.node("Mix", 2);
    b.value(dark, 0, base);
    b.value(dark, 1, [0.01; 3]);
    b.link(n, dark, 2);
    b.out(dark, BASE);
    b.out_value(ROUGH, rough);
    b.bump(n, 1.5, 3);
    b.g
}

fn tiles(color_a: [f32; 3], color_b: [f32; 3], count: f32, gap: f32, rough: f32, metal: f32) -> MaterialGraph {
    let mut b = B::new();
    let grid = b.node("Grid lines", 0);
    b.set(grid, &[count, gap]);
    let checker = b.node("Checker", 0);
    b.set(checker, &[count]);
    let mix = b.node("Mix", 1);
    b.value(mix, 0, color_a);
    b.value(mix, 1, color_b);
    b.link(checker, mix, 2);
    let grout = b.node("Mix", 2);
    b.link(mix, grout, 0);
    b.value(grout, 1, [0.08, 0.08, 0.08]);
    b.link(grid, grout, 2);
    b.out(grout, BASE);
    b.out_value(ROUGH, rough);
    b.out_value(METAL, metal);
    let inv = b.node("One minus", 1);
    b.link(grid, inv, 0);
    b.bump(inv, 2.0, 2);
    b.g
}

fn wood(light: [f32; 3], dark: [f32; 3], planks: bool) -> MaterialGraph {
    let mut b = B::new();
    let rings = b.node("Wood rings", 0);
    b.set(rings, &[4.0, 0.35]);
    let fine = b.fractal(40.0, 3.0, 0.0, 0);
    let grain = b.node("Blend colours", 1);
    b.set(grain, &[1.0]);
    b.link(rings, grain, 0);
    b.link(fine, grain, 1);
    let ramp = b.ramp(grain, [dark, [dark[0] * 1.5, dark[1] * 1.4, dark[2] * 1.3], light, [light[0] * 1.1, light[1] * 1.1, light[2] * 1.05]], [0.0, 0.35, 0.7, 1.0], 2);
    if planks {
        let pl = b.node("Planks", 0);
        b.set(pl, &[6.0, 3.0, 0.02]);
        let shade = b.node("Multiply", 3);
        b.link(ramp, shade, 0);
        b.link(pl, shade, 1);
        b.out(shade, BASE);
        b.bump(pl, 1.0, 3);
    } else {
        b.out(ramp, BASE);
        b.bump(fine, 0.6, 3);
    }
    b.out_value(ROUGH, 0.55);
    b.g
}

fn marble(base: [f32; 3], vein: [f32; 3], warp: f32) -> MaterialGraph {
    let mut b = B::new();
    let v = b.node("Marble veins", 0);
    b.set(v, &[1.6, warp]);
    let fine = b.fractal(9.0, 4.0, 0.0, 0);
    let mix = b.node("Mix", 1);
    b.value(mix, 0, base);
    b.value(mix, 1, vein);
    let sharp = b.node("Smoothstep", 1);
    b.set(sharp, &[0.0, 0.25]);
    b.link(v, sharp, 0);
    let inv = b.node("One minus", 2);
    b.link(sharp, inv, 0);
    b.link(inv, mix, 2);
    b.out(mix, BASE);
    let r = b.map_range(fine, 0.12, 0.3, 2);
    b.out_value(METAL, 0.0);
    b.out(r, ROUGH);
    b.g
}

fn bricks(brick: [f32; 3], mortar: [f32; 3]) -> MaterialGraph {
    let mut b = B::new();
    let br = b.node("Bricks", 0);
    b.set(br, &[6.0, 0.05]);
    let n = b.fractal(12.0, 4.0, 0.0, 0);
    let cells = b.node("Voronoi cells", 0);
    b.set(cells, &[8.0, 1.0, 2.0]);
    let tone = b.ramp(cells, [[brick[0] * 0.6, brick[1] * 0.6, brick[2] * 0.6], brick, [brick[0] * 1.25, brick[1] * 1.15, brick[2] * 1.1], brick], [0.0, 0.4, 0.8, 1.0], 1);
    let mix = b.node("Mix", 2);
    b.value(mix, 0, mortar);
    b.link(tone, mix, 1);
    b.link(br, mix, 2);
    let dirt = b.node("Blend colours", 3);
    b.set(dirt, &[1.0]);
    b.link(mix, dirt, 0);
    let grime = b.map_range(n, 0.7, 1.1, 1);
    b.link(grime, dirt, 1);
    b.out(dirt, BASE);
    b.out_value(ROUGH, 0.85);
    b.bump(br, 3.0, 2);
    b.g
}

fn weave(a: [f32; 3], bcol: [f32; 3], threads: f32, metal: f32, rough: f32) -> MaterialGraph {
    let mut b = B::new();
    let w = b.node("Weave", 0);
    b.set(w, &[threads]);
    let mix = b.node("Mix", 1);
    b.value(mix, 0, a);
    b.value(mix, 1, bcol);
    b.link(w, mix, 2);
    let shade = b.node("Multiply", 2);
    b.link(mix, shade, 0);
    b.link(w, shade, 1);
    b.out(shade, BASE);
    b.out_value(ROUGH, rough);
    b.out_value(METAL, metal);
    b.bump(w, 2.5, 1);
    b.g
}

fn hex_metal(base: [f32; 3], edge: [f32; 3], scale: f32) -> MaterialGraph {
    let mut b = B::new();
    let h = b.node("Hexagons", 0);
    b.set(h, &[scale, 0.08]);
    let mix = b.node("Mix", 1);
    b.value(mix, 0, edge);
    b.value(mix, 1, base);
    b.link(h, mix, 2);
    b.out(mix, BASE);
    b.out_value(ROUGH, 0.35);
    b.out_value(METAL, 1.0);
    b.bump(h, 2.0, 1);
    b.g
}

fn cracked(base: [f32; 3], crack: [f32; 3], scale: f32, rough: f32) -> MaterialGraph {
    let mut b = B::new();
    let c = b.node("Cracks", 0);
    b.set(c, &[scale, 0.07]);
    let n = b.fractal(6.0, 5.0, 0.0, 0);
    let mix = b.node("Mix", 1);
    b.value(mix, 0, base);
    b.value(mix, 1, crack);
    b.link(c, mix, 2);
    let shade = b.node("Blend colours", 2);
    b.set(shade, &[1.0]);
    b.link(mix, shade, 0);
    let m = b.map_range(n, 0.75, 1.1, 1);
    b.link(m, shade, 1);
    b.out(shade, BASE);
    b.out_value(ROUGH, rough);
    let inv = b.node("One minus", 1);
    b.link(c, inv, 0);
    b.bump(inv, 2.5, 2);
    b.g
}

fn neon_grid(line: [f32; 3], intensity: f32) -> MaterialGraph {
    let mut b = B::new();
    let g = b.node("Grid lines", 0);
    b.set(g, &[10.0, 0.03]);
    let glow = b.node("Multiply", 1);
    b.value(glow, 0, line);
    b.link(g, glow, 1);
    let boost = b.node("Multiply", 2);
    b.link(glow, boost, 0);
    b.value(boost, 1, [intensity; 3]);
    let pulse = b.node("Pulse", 0);
    b.set(pulse, &[0.5]);
    let mixed = b.node("Mix", 3);
    b.link(boost, mixed, 1);
    b.value(mixed, 0, [0.0; 3]);
    b.link(pulse, mixed, 2);
    b.out(boost, EMIT);
    b.value(b.out, BASE, [0.02, 0.02, 0.03]);
    b.out_value(ROUGH, 0.3);
    b.g
}

fn glass(tint: [f32; 3], alpha: f32, rough: f32) -> MaterialGraph {
    let mut b = B::new();
    let f = b.node("Fresnel", 0);
    b.value(f, 0, [3.0; 3]);
    let a = b.map_range(f, alpha, 1.0, 1);
    b.out(a, ALPHA);
    b.value(b.out, BASE, tint);
    b.out_value(ROUGH, rough);
    b.out_value(METAL, 0.0);
    b.g
}

fn plain(color: [f32; 3], rough: f32, metal: f32) -> MaterialGraph {
    let mut b = B::new();
    b.value(b.out, BASE, color);
    b.out_value(ROUGH, rough);
    b.out_value(METAL, metal);
    b.g
}

fn p(category: &'static str, name: &'static str, graph: MaterialGraph) -> Preset {
    Preset { category, name, graph }
}

/// Every shipped material.
pub fn library() -> Vec<Preset> {
    let sand = [[0.55, 0.45, 0.28], [0.72, 0.6, 0.38], [0.82, 0.72, 0.5], [0.9, 0.82, 0.62]];
    let mut v = vec![
        // ---- stone and ground
        p("Stone", "Granite", natural([[0.18, 0.18, 0.19], [0.42, 0.41, 0.42], [0.62, 0.6, 0.6], [0.8, 0.78, 0.78]], [0.0, 0.35, 0.65, 1.0], 6.0, 0.0, (0.4, 0.7), 0.0, 1.0)),
        p("Stone", "Rock", natural([[0.12, 0.11, 0.1], [0.28, 0.25, 0.22], [0.45, 0.4, 0.35], [0.62, 0.57, 0.5]], [0.0, 0.3, 0.65, 1.0], 2.5, 1.0, (0.75, 1.0), 0.0, 4.0)),
        p("Stone", "Sandstone", natural([[0.5, 0.36, 0.2], [0.68, 0.5, 0.3], [0.8, 0.62, 0.4], [0.88, 0.72, 0.5]], [0.0, 0.3, 0.7, 1.0], 3.5, 2.0, (0.8, 1.0), 0.0, 2.0)),
        p("Stone", "Slate", natural([[0.06, 0.07, 0.09], [0.14, 0.16, 0.2], [0.22, 0.25, 0.3], [0.32, 0.35, 0.4]], [0.0, 0.4, 0.7, 1.0], 4.0, 1.0, (0.45, 0.8), 0.0, 1.5)),
        p("Stone", "Concrete", natural([[0.3, 0.3, 0.3], [0.42, 0.42, 0.41], [0.52, 0.52, 0.5], [0.6, 0.6, 0.58]], [0.0, 0.4, 0.7, 1.0], 5.0, 0.0, (0.85, 1.0), 0.0, 0.8)),
        p("Stone", "Asphalt", natural([[0.04, 0.04, 0.045], [0.08, 0.08, 0.09], [0.14, 0.14, 0.15], [0.22, 0.22, 0.23]], [0.0, 0.4, 0.75, 1.0], 12.0, 0.0, (0.8, 1.0), 0.0, 1.5)),
        p("Stone", "Cobblestone", cracked([0.32, 0.3, 0.28], [0.05, 0.05, 0.05], 5.0, 0.9)),
        p("Stone", "Marble White", marble([0.92, 0.92, 0.9], [0.35, 0.37, 0.42], 4.0)),
        p("Stone", "Marble Black", marble([0.04, 0.04, 0.05], [0.8, 0.75, 0.55], 5.0)),
        p("Stone", "Marble Green", marble([0.08, 0.3, 0.2], [0.85, 0.92, 0.85], 3.0)),
        p("Ground", "Dirt", natural([[0.1, 0.07, 0.04], [0.2, 0.14, 0.08], [0.32, 0.23, 0.14], [0.42, 0.32, 0.2]], [0.0, 0.35, 0.7, 1.0], 3.0, 0.0, (0.9, 1.0), 0.0, 2.5)),
        p("Ground", "Mud", natural([[0.06, 0.04, 0.025], [0.13, 0.09, 0.05], [0.2, 0.14, 0.08], [0.28, 0.2, 0.12]], [0.0, 0.4, 0.75, 1.0], 2.0, 2.0, (0.3, 0.7), 0.0, 3.0)),
        p("Ground", "Sand", natural(sand, [0.0, 0.35, 0.7, 1.0], 8.0, 0.0, (0.85, 1.0), 0.0, 1.0)),
        p("Ground", "Grass", natural([[0.04, 0.12, 0.03], [0.08, 0.25, 0.05], [0.16, 0.38, 0.08], [0.3, 0.5, 0.14]], [0.0, 0.35, 0.7, 1.0], 9.0, 0.0, (0.8, 1.0), 0.0, 1.2)),
        p("Ground", "Moss", natural([[0.03, 0.1, 0.03], [0.1, 0.25, 0.06], [0.2, 0.4, 0.1], [0.35, 0.55, 0.2]], [0.0, 0.3, 0.65, 1.0], 14.0, 2.0, (0.9, 1.0), 0.0, 2.5)),
        p("Ground", "Snow", natural([[0.7, 0.75, 0.82], [0.85, 0.9, 0.96], [0.95, 0.97, 1.0], [1.0, 1.0, 1.0]], [0.0, 0.3, 0.7, 1.0], 4.0, 0.0, (0.55, 0.8), 0.0, 0.6)),
        p("Ground", "Ice", natural([[0.45, 0.65, 0.8], [0.62, 0.8, 0.92], [0.8, 0.92, 0.98], [0.95, 1.0, 1.0]], [0.0, 0.35, 0.7, 1.0], 3.0, 1.0, (0.05, 0.25), 0.0, 0.5)),
        p("Ground", "Dry Earth Cracks", cracked([0.5, 0.38, 0.24], [0.06, 0.04, 0.02], 4.0, 0.95)),
        // ---- metal
        p("Metal", "Brushed Steel", metal_brushed([0.75, 0.77, 0.8], [0.5, 0.52, 0.55], 0.35, 180.0)),
        p("Metal", "Brushed Aluminium", metal_brushed([0.88, 0.89, 0.9], [0.65, 0.66, 0.68], 0.3, 220.0)),
        p("Metal", "Copper", metal_brushed([0.95, 0.52, 0.38], [0.6, 0.3, 0.2], 0.3, 120.0)),
        p("Metal", "Gold", metal_brushed([1.0, 0.78, 0.34], [0.75, 0.55, 0.2], 0.2, 90.0)),
        p("Metal", "Titanium", metal_brushed([0.55, 0.52, 0.5], [0.38, 0.36, 0.35], 0.38, 160.0)),
        p("Metal", "Rusty Iron", natural([[0.1, 0.1, 0.1], [0.4, 0.17, 0.07], [0.6, 0.28, 0.1], [0.35, 0.33, 0.32]], [0.0, 0.35, 0.65, 1.0], 4.0, 1.0, (0.55, 0.95), 0.6, 3.0)),
        p("Metal", "Hexagon Plate", hex_metal([0.5, 0.52, 0.56], [0.12, 0.12, 0.14], 7.0)),
        p("Metal", "Carbon Fiber", weave([0.02, 0.02, 0.025], [0.07, 0.07, 0.08], 24.0, 0.3, 0.3)),
        p("Metal", "Chain Mail", hex_metal([0.4, 0.42, 0.45], [0.05, 0.05, 0.06], 18.0)),
        // ---- wood and organic
        p("Wood", "Oak", wood([0.55, 0.38, 0.2], [0.3, 0.19, 0.09], false)),
        p("Wood", "Dark Walnut", wood([0.25, 0.14, 0.08], [0.1, 0.055, 0.03], false)),
        p("Wood", "Pine Planks", wood([0.78, 0.6, 0.36], [0.55, 0.38, 0.2], true)),
        p("Wood", "Floor Boards", wood([0.45, 0.3, 0.17], [0.22, 0.13, 0.07], true)),
        p("Organic", "Leather", cracked([0.22, 0.1, 0.05], [0.08, 0.035, 0.02], 22.0, 0.55)),
        p("Organic", "Skin", natural([[0.55, 0.32, 0.25], [0.72, 0.46, 0.36], [0.85, 0.6, 0.48], [0.9, 0.68, 0.56]], [0.0, 0.35, 0.7, 1.0], 18.0, 0.0, (0.45, 0.6), 0.0, 0.4)),
        p("Organic", "Bark", natural([[0.06, 0.045, 0.035], [0.15, 0.11, 0.08], [0.26, 0.19, 0.13], [0.36, 0.28, 0.2]], [0.0, 0.35, 0.7, 1.0], 3.0, 1.0, (0.9, 1.0), 0.0, 5.0)),
        // ---- fabric, tiles, bricks
        p("Fabric", "Linen", weave([0.72, 0.68, 0.58], [0.6, 0.56, 0.47], 40.0, 0.0, 0.95)),
        p("Fabric", "Denim", weave([0.08, 0.14, 0.3], [0.12, 0.2, 0.4], 60.0, 0.0, 0.95)),
        p("Fabric", "Red Cloth", weave([0.5, 0.04, 0.04], [0.65, 0.07, 0.06], 50.0, 0.0, 0.85)),
        p("Building", "Red Bricks", bricks([0.55, 0.2, 0.12], [0.55, 0.53, 0.5])),
        p("Building", "Old Bricks", bricks([0.4, 0.22, 0.16], [0.3, 0.29, 0.28])),
        p("Building", "Checker Tiles", tiles([0.9, 0.9, 0.88], [0.1, 0.1, 0.12], 8.0, 0.04, 0.2, 0.0)),
        p("Building", "Bathroom Tiles", tiles([0.85, 0.9, 0.92], [0.78, 0.84, 0.88], 12.0, 0.05, 0.12, 0.0)),
        p("Building", "Plaster", natural([[0.7, 0.68, 0.64], [0.78, 0.76, 0.72], [0.85, 0.83, 0.79], [0.9, 0.88, 0.85]], [0.0, 0.4, 0.7, 1.0], 7.0, 0.0, (0.85, 1.0), 0.0, 0.5)),
        // ---- emissive and sci-fi
        p("Emissive", "Lava", emissive([[0.04, 0.0, 0.0], [0.6, 0.06, 0.0], [1.0, 0.45, 0.05], [1.0, 0.9, 0.4]], [0.2, 0.45, 0.7, 1.0], 2.5, 1.0, 4.0, [0.04, 0.03, 0.03], 0.9)),
        p("Emissive", "Toxic Slime", emissive([[0.0, 0.05, 0.0], [0.05, 0.4, 0.0], [0.3, 0.9, 0.05], [0.8, 1.0, 0.4]], [0.2, 0.5, 0.75, 1.0], 3.0, 2.0, 2.5, [0.04, 0.12, 0.03], 0.15)),
        p("Emissive", "Plasma", emissive([[0.05, 0.0, 0.2], [0.4, 0.0, 0.9], [0.9, 0.3, 1.0], [1.0, 0.9, 1.0]], [0.1, 0.4, 0.7, 1.0], 4.0, 2.0, 3.0, [0.02, 0.0, 0.05], 0.3)),
        p("Emissive", "Neon Grid Cyan", neon_grid([0.0, 0.9, 1.0], 6.0)),
        p("Emissive", "Neon Grid Magenta", neon_grid([1.0, 0.1, 0.8], 6.0)),
        // ---- plain and translucent
        p("Plain", "Glass Clear", glass([0.9, 0.95, 1.0], 0.12, 0.03)),
        p("Plain", "Glass Green", glass([0.3, 0.7, 0.45], 0.25, 0.05)),
        p("Plain", "Rubber Black", plain([0.02, 0.02, 0.022], 0.85, 0.0)),
        p("Plain", "Plastic White", plain([0.9, 0.9, 0.9], 0.35, 0.0)),
        p("Plain", "Plastic Red", plain([0.8, 0.04, 0.03], 0.3, 0.0)),
        p("Plain", "Plastic Blue", plain([0.05, 0.2, 0.8], 0.3, 0.0)),
        p("Plain", "Chrome", plain([0.9, 0.9, 0.92], 0.04, 1.0)),
        p("Plain", "Clay", plain([0.55, 0.3, 0.2], 0.9, 0.0)),
    ];
    for preset in &mut v {
        preset.graph.next_id = preset.graph.nodes.iter().map(|n| n.id + 1).max().unwrap_or(1);
        for n in &mut preset.graph.nodes {
            n.preview = false;
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_every_preset_round_trips() {
        let all = library();
        let mut seen = std::collections::HashSet::new();
        for p in &all {
            assert!(seen.insert((p.category, p.name)), "duplicate {} / {}", p.category, p.name);
            let again = MaterialGraph::from_json(&p.graph.to_json()).unwrap();
            assert_eq!(again, p.graph, "{}", p.name);
            again.validate().unwrap_or_else(|e| panic!("{}: {e}", p.name));
        }
    }
}
