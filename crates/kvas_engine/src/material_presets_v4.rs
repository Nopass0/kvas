//! Game-ready materials built only from general nodes (texture coordinates, noise, Voronoi,
//! bricks, maths, ramps, mixing and bump), so every step can be opened and changed in the node
//! editor. They go to the `Game Ready` folder of the material library; marble is built here too.
//!
//! ```
//! let all = kvas_engine::material_presets_v4::library();
//! assert!(all.len() >= 30);
//! assert!(all.iter().all(|p| kvas_engine::material_graph::to_wgsl(&p.graph).is_ok()));
//! ```

use crate::material_graph::{GraphMode, MaterialGraph};
use crate::material_presets::Preset;

const BASE: usize = 0;
const ROUGH: usize = 1;
const METAL: usize = 2;
const EMIT: usize = 3;
const AO: usize = 5;
const NORMAL: usize = 6;

// Outputs used below.
const WORLD: u8 = 2; // Texture coordinate > World
const UV: u8 = 0;
const FACTOR: u8 = 0;
const COLOUR: u8 = 1; // Noise texture > Colour
const EDGE: u8 = 2; // Voronoi texture > Edge
const CELL: u8 = 3; // Voronoi texture > Cell value
const MASK: u8 = 1; // Brick texture > Mask
const BRICK_VALUE: u8 = 2;
const SLOPE: u8 = 5; // Geometry info > Slope
const NORMAL_UP: u8 = 1; // Normal > Up facing

// Math operations.
const ADD: f32 = 0.0;
const SUB: f32 = 1.0;
const MUL: f32 = 2.0;
const POW: f32 = 4.0;
const ABS: f32 = 7.0;
const SIN: f32 = 17.0;
const MULADD: f32 = 25.0;

// Mix colour modes.
const MIX: f32 = 0.0;
const MULTIPLY: f32 = 1.0;
const OVERLAY: f32 = 3.0;

/// Graph builder: nodes are stacked in columns from left to right.
struct G {
    g: MaterialGraph,
    out: u32,
    col: [f32; 8],
}

impl G {
    fn new() -> Self {
        let mut g = MaterialGraph { nodes: Vec::new(), textures: Default::default(), next_id: 1, mode: GraphMode::Material };
        let out = g.add_node("Output", [1900.0, 140.0]);
        G { g, out, col: [0.0; 8] }
    }

    fn node(&mut self, kind: &str, column: usize, params: &[f32]) -> u32 {
        let c = column.min(7);
        let y = self.col[c];
        let id = self.g.add_node(kind, [40.0 + c as f32 * 260.0, 30.0 + y]);
        let n = self.g.nodes.iter_mut().find(|n| n.id == id).unwrap();
        for (i, v) in params.iter().enumerate() {
            n.params[i] = *v;
        }
        let h = 60.0 + (n.inputs.len() + n.params.len() + crate::material_nodes_v4::outputs(kind).len()) as f32 * 24.0;
        self.col[c] += h.max(120.0);
        id
    }

    fn value(&mut self, id: u32, input: usize, v: [f32; 3]) {
        let n = self.g.nodes.iter_mut().find(|n| n.id == id).unwrap();
        n.inputs[input].value = v;
        n.inputs[input].override_default = true;
    }

    fn num(&mut self, id: u32, input: usize, v: f32) {
        self.value(id, input, [v; 3]);
    }

    /// Links output `out` of `from` to `input` of `to`, converting with `convert` (0 = as is).
    fn link(&mut self, from: u32, out: u8, to: u32, input: usize, convert: u8) {
        assert!(self.g.connect_output(from, out, to, input, convert), "link {from}.{out} -> {to}.{input}");
    }

    fn out(&mut self, from: u32, out: u8, socket: usize) {
        let o = self.out;
        self.link(from, out, o, socket, 0);
    }

    fn out_value(&mut self, socket: usize, v: f32) {
        let o = self.out;
        self.num(o, socket, v);
    }

    /// World coordinates scaled by `scale` (Texture coordinate > World into a Mapping node).
    fn world(&mut self, scale: f32) -> u32 {
        let tc = self.node("Texture coordinate", 0, &[]);
        let m = self.node("Mapping", 0, &[0.0, 0.0, 0.0, 0.0, scale, scale, scale]);
        self.link(tc, WORLD, m, 0, 0);
        m
    }

    fn uv(&mut self, scale: f32) -> u32 {
        let tc = self.node("Texture coordinate", 0, &[]);
        let m = self.node("Mapping", 0, &[0.0, 0.0, 0.0, 0.0, scale, scale, 1.0]);
        self.link(tc, UV, m, 0, 0);
        m
    }

    fn noise(&mut self, coords: u32, scale: f32, detail: f32, roughness: f32, distortion: f32, column: usize) -> u32 {
        let n = self.node("Noise texture", column, &[0.0, 2.0]);
        self.link(coords, 0, n, 0, 0);
        self.num(n, 1, scale);
        self.num(n, 2, detail);
        self.num(n, 3, roughness);
        self.num(n, 4, distortion);
        n
    }

    fn voronoi(&mut self, coords: u32, scale: f32, randomness: f32, column: usize) -> u32 {
        let v = self.node("Voronoi texture", column, &[0.0]);
        self.link(coords, 0, v, 0, 0);
        self.num(v, 1, scale);
        self.num(v, 2, randomness);
        v
    }

    fn math(&mut self, op: f32, column: usize, clamp: bool) -> u32 {
        self.node("Math", column, &[op, if clamp { 1.0 } else { 0.0 }])
    }

    fn ramp(&mut self, source: u32, out: u8, stops: [[f32; 3]; 4], pos: [f32; 4], ease: bool, column: usize) -> u32 {
        let r = self.node("Color ramp", column, &[pos[0], pos[1], pos[2], pos[3], if ease { 2.0 } else { 0.0 }]);
        for (i, s) in stops.iter().enumerate() {
            self.value(r, i + 1, *s);
        }
        self.link(source, out, r, 0, 0);
        r
    }

    /// `A` blended with `B` by `factor` in a mode.
    fn mix(&mut self, mode: f32, column: usize) -> u32 {
        self.node("Mix colour", column, &[mode, 0.0])
    }

    fn map(&mut self, source: u32, out: u8, lo: f32, hi: f32, column: usize) -> u32 {
        let m = self.node("Map range", column, &[0.0, 1.0, lo, hi]);
        self.link(source, out, m, 0, 0);
        m
    }

    fn bump(&mut self, height: u32, out: u8, strength: f32, distance: f32, column: usize) -> u32 {
        let b = self.node("Bump", column, &[0.0]);
        self.link(height, out, b, 0, 4);
        self.num(b, 1, strength);
        self.num(b, 2, distance);
        self.out(b, 0, NORMAL);
        b
    }
}

/// Marble from general nodes: world coordinates projected on a direction, bent by fractal noise,
/// turned into thin veins by |sin|, coloured by a ramp, with cloudy variation and slight relief.
pub fn marble(base: [f32; 3], vein: [f32; 3], warp: f32, frequency: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(3.0);
    // Direction of the veins.
    let dir = b.node("Vector math", 1, &[7.0]);
    b.link(p, 0, dir, 0, 0);
    b.value(dir, 1, [0.82, 0.31, 0.48]);
    // Turbulence that bends the veins.
    let turb = b.noise(p, 0.9, 7.0, 0.58, 0.35, 1);
    let along = b.math(MUL, 2, false);
    b.link(dir, 0, along, 0, 1);
    b.num(along, 1, frequency);
    let phase = b.math(MULADD, 2, false);
    b.link(turb, FACTOR, phase, 0, 0);
    b.num(phase, 1, warp);
    b.link(along, 0, phase, 2, 0);
    let wave = b.math(SIN, 3, false);
    b.link(phase, 0, wave, 0, 0);
    let veins = b.math(ABS, 3, false);
    b.link(wave, 0, veins, 0, 0);
    // A second, finer and fainter vein network.
    let fine_phase = b.math(MULADD, 2, false);
    b.link(turb, FACTOR, fine_phase, 0, 0);
    b.num(fine_phase, 1, warp * 2.3);
    b.link(along, 0, fine_phase, 2, 0);
    let fine_sin = b.math(SIN, 3, false);
    b.link(fine_phase, 0, fine_sin, 0, 0);
    let fine_abs = b.math(ABS, 3, false);
    b.link(fine_sin, 0, fine_abs, 0, 0);
    let fine_soft = b.math(POW, 4, true);
    b.link(fine_abs, 0, fine_soft, 0, 0);
    b.num(fine_soft, 1, 0.35);
    let mid = [(base[0] + vein[0]) * 0.5, (base[1] + vein[1]) * 0.5, (base[2] + vein[2]) * 0.5];
    let colour = b.ramp(veins, 0, [vein, mid, base, [base[0] * 1.03, base[1] * 1.03, base[2] * 1.02]], [0.0, 0.035, 0.16, 1.0], true, 4);
    let faint = b.mix(MULTIPLY, 5);
    b.num(faint, 0, 0.25);
    b.link(colour, 0, faint, 1, 0);
    b.link(fine_soft, 0, faint, 2, 0);
    // Cloudy tone variation.
    let clouds = b.noise(p, 2.4, 4.0, 0.5, 0.0, 4);
    let cloudy = b.mix(OVERLAY, 6);
    b.num(cloudy, 0, 0.3);
    b.link(faint, 0, cloudy, 1, 0);
    b.link(clouds, COLOUR, cloudy, 2, 0);
    b.out(cloudy, 0, BASE);
    // Polished, slightly rougher in the veins.
    let rough = b.map(veins, 0, 0.32, 0.08, 5);
    b.out(rough, 0, ROUGH);
    b.out_value(METAL, 0.0);
    b.bump(veins, 0, 0.08, 0.01, 6);
    b.g
}

/// Stone or clay bricks with mortar, per-brick tint, dirt in the joints and moss on top faces.
fn brick_wall(c1: [f32; 3], c2: [f32; 3], mortar: [f32; 3], scale: f32, moss: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.uv(scale);
    let bricks = b.node("Brick texture", 1, &[2.1, 0.5, 0.045, 0.03, 0.8]);
    b.link(p, 0, bricks, 0, 0);
    b.value(bricks, 1, c1);
    b.value(bricks, 2, c2);
    b.value(bricks, 3, mortar);
    b.num(bricks, 4, 4.0);
    let w = b.world(1.0);
    let grime = b.noise(w, 3.0, 6.0, 0.6, 0.2, 1);
    let dirty = b.mix(MULTIPLY, 2);
    b.num(dirty, 0, 0.55);
    b.link(bricks, 0, dirty, 1, 0);
    b.link(grime, COLOUR, dirty, 2, 0);
    // Moss where the surface faces up, broken by noise.
    let up = b.node("Normal", 2, &[]);
    let moss_noise = b.noise(w, 5.0, 5.0, 0.6, 0.0, 2);
    let moss_mask = b.math(MUL, 3, true);
    b.link(up, NORMAL_UP, moss_mask, 0, 0);
    b.link(moss_noise, FACTOR, moss_mask, 1, 0);
    let moss_k = b.math(MUL, 3, true);
    b.link(moss_mask, 0, moss_k, 0, 0);
    b.num(moss_k, 1, moss * 2.0);
    let mossy = b.mix(MIX, 4);
    b.link(moss_k, 0, mossy, 0, 0);
    b.link(dirty, 0, mossy, 1, 0);
    b.value(mossy, 2, [0.07, 0.16, 0.04]);
    b.out(mossy, 0, BASE);
    let rough = b.map(grime, FACTOR, 0.75, 0.98, 4);
    b.out(rough, 0, ROUGH);
    let ao = b.map(bricks, MASK, 0.45, 1.0, 4);
    b.out(ao, 0, AO);
    b.out_value(METAL, 0.0);
    // Relief: bricks stand out of the mortar, plus fine pitting.
    let pits = b.noise(w, 18.0, 3.0, 0.5, 0.0, 4);
    let height = b.math(MULADD, 5, false);
    b.link(pits, FACTOR, height, 0, 0);
    b.num(height, 1, 0.15);
    b.link(bricks, MASK, height, 2, 0);
    b.bump(height, 0, 0.9, 0.06, 6);
    b.g
}

/// Rounded cobbles from Voronoi cells, with dark gaps, per-stone colour and dirt.
fn cobbles(stone_a: [f32; 3], stone_b: [f32; 3], gap: [f32; 3], scale: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(1.0);
    let jitter = b.node("Jitter coords", 1, &[3.0]);
    b.link(p, 0, jitter, 0, 0);
    b.num(jitter, 1, 0.12);
    let cells = b.voronoi(jitter, scale, 0.9, 1);
    let stone = b.mix(MIX, 2);
    b.link(cells, CELL, stone, 0, 0);
    b.value(stone, 1, stone_a);
    b.value(stone, 2, stone_b);
    let edge = b.node("Smoothstep", 2, &[0.0, 0.12]);
    b.link(cells, EDGE, edge, 0, 0);
    let grain = b.noise(p, 14.0, 4.0, 0.6, 0.0, 2);
    let grainy = b.mix(OVERLAY, 3);
    b.num(grainy, 0, 0.4);
    b.link(stone, 0, grainy, 1, 0);
    b.link(grain, COLOUR, grainy, 2, 0);
    let with_gaps = b.mix(MIX, 4);
    b.link(edge, 0, with_gaps, 0, 0);
    b.value(with_gaps, 1, gap);
    b.link(grainy, 0, with_gaps, 2, 0);
    b.out(with_gaps, 0, BASE);
    let rough = b.map(edge, 0, 1.0, 0.7, 4);
    b.out(rough, 0, ROUGH);
    b.out(edge, 0, AO);
    b.out_value(METAL, 0.0);
    let dome = b.math(POW, 5, false);
    b.link(edge, 0, dome, 0, 0);
    b.num(dome, 1, 0.5);
    b.bump(dome, 0, 1.2, 0.08, 6);
    b.g
}

/// Wooden planks: stretched noise grain coloured by a ramp, planks from the brick node.
fn planks(light: [f32; 3], dark: [f32; 3], scale: f32, worn: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.uv(scale);
    let boards = b.node("Brick texture", 1, &[6.0, 0.37, 0.012, 0.006, 0.7]);
    b.link(p, 0, boards, 0, 0);
    b.num(boards, 4, 3.0);
    // Grain: noise stretched along the planks.
    let stretch = b.node("Mapping", 1, &[0.0, 0.0, 0.0, 0.0, 1.0, 18.0, 1.0]);
    b.link(p, 0, stretch, 0, 0);
    let grain = b.noise(stretch, 2.5, 6.0, 0.62, 1.2, 2);
    let rings = b.node("Wave texture", 2, &[0.0, 0.0]);
    b.link(stretch, 0, rings, 0, 0);
    b.num(rings, 1, 1.4);
    b.num(rings, 2, 6.0);
    let both = b.math(MUL, 3, true);
    b.link(grain, FACTOR, both, 0, 0);
    b.link(rings, 0, both, 1, 0);
    let wood = b.ramp(both, 0, [[dark[0] * 0.7, dark[1] * 0.7, dark[2] * 0.7], dark, light, [light[0] * 1.08, light[1] * 1.06, light[2] * 1.04]], [0.0, 0.25, 0.6, 1.0], true, 4);
    // Each plank a little different.
    let tone = b.node("Hue saturation value", 5, &[]);
    b.link(wood, 0, tone, 0, 0);
    b.link(boards, BRICK_VALUE, tone, 3, 11);
    let gaps = b.mix(MIX, 6);
    b.link(boards, MASK, gaps, 0, 0);
    b.value(gaps, 1, [0.03, 0.02, 0.012]);
    b.link(tone, 0, gaps, 2, 0);
    b.out(gaps, 0, BASE);
    let rough = b.map(grain, FACTOR, 0.45 + worn * 0.3, 0.8, 5);
    b.out(rough, 0, ROUGH);
    b.out(boards, MASK, AO);
    b.out_value(METAL, 0.0);
    let height = b.math(MULADD, 6, false);
    b.link(grain, FACTOR, height, 0, 0);
    b.num(height, 1, 0.2);
    b.link(boards, MASK, height, 2, 0);
    b.bump(height, 0, 0.7, 0.03, 7);
    b.g
}

/// Painted metal with chipped paint showing rust or bare steel at the edges of noise patches.
fn chipped_paint(paint: [f32; 3], under: [f32; 3], rusty: bool) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(1.0);
    let chips = b.noise(p, 3.5, 8.0, 0.65, 0.3, 1);
    let chip_mask = b.node("Smoothstep", 2, &[0.56, 0.6]);
    b.link(chips, FACTOR, chip_mask, 0, 0);
    let stains = b.noise(p, 8.0, 5.0, 0.6, 0.0, 1);
    let metal_col = b.mix(OVERLAY, 2);
    b.num(metal_col, 0, 0.6);
    b.value(metal_col, 1, under);
    b.link(stains, COLOUR, metal_col, 2, 0);
    let col = b.mix(MIX, 3);
    b.link(chip_mask, 0, col, 0, 0);
    b.value(col, 1, paint);
    b.link(metal_col, 0, col, 2, 0);
    // Dust in crevices (where the surface faces down or sideways).
    let geo = b.node("Geometry info", 3, &[]);
    let dust = b.mix(MULTIPLY, 4);
    b.link(geo, SLOPE, dust, 0, 0);
    b.num(dust, 0, 0.15);
    b.link(col, 0, dust, 1, 0);
    b.value(dust, 2, [0.6, 0.55, 0.5]);
    b.out(dust, 0, BASE);
    let rough = b.map(chip_mask, 0, 0.35, if rusty { 0.95 } else { 0.4 }, 4);
    b.out(rough, 0, ROUGH);
    let metal = b.map(chip_mask, 0, 0.0, if rusty { 0.3 } else { 1.0 }, 4);
    b.out(metal, 0, METAL);
    b.bump(chip_mask, 0, -0.5, 0.01, 5);
    b.g
}

/// Ground cover: two noise layers mixed by slope and height; pebbles from Voronoi.
fn ground(low: [f32; 3], high: [f32; 3], rocks: [f32; 3], scale: f32, rough: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(1.0);
    let big = b.noise(p, scale * 0.35, 6.0, 0.55, 0.4, 1);
    let mix = b.mix(MIX, 2);
    b.link(big, FACTOR, mix, 0, 0);
    b.value(mix, 1, low);
    b.value(mix, 2, high);
    let pebbles = b.voronoi(p, scale * 2.2, 1.0, 1);
    let peb_mask = b.node("Smoothstep", 2, &[0.32, 0.22]);
    b.link(pebbles, 1, peb_mask, 0, 0);
    let peb_noise = b.noise(p, scale * 3.0, 3.0, 0.5, 0.0, 2);
    let peb_sparse = b.math(MUL, 3, true);
    b.link(peb_mask, 0, peb_sparse, 0, 0);
    b.link(peb_noise, FACTOR, peb_sparse, 1, 0);
    let peb_cut = b.node("Smoothstep", 3, &[0.25, 0.4]);
    b.link(peb_sparse, 0, peb_cut, 0, 0);
    let with_rocks = b.mix(MIX, 4);
    b.link(peb_cut, 0, with_rocks, 0, 0);
    b.link(mix, 0, with_rocks, 1, 0);
    b.value(with_rocks, 2, rocks);
    let fine = b.noise(p, scale * 6.0, 3.0, 0.6, 0.0, 4);
    let detail = b.mix(OVERLAY, 5);
    b.num(detail, 0, 0.35);
    b.link(with_rocks, 0, detail, 1, 0);
    b.link(fine, COLOUR, detail, 2, 0);
    b.out(detail, 0, BASE);
    let r = b.map(big, FACTOR, rough, (rough + 0.15).min(1.0), 5);
    b.out(r, 0, ROUGH);
    b.out_value(METAL, 0.0);
    let height = b.math(MULADD, 5, false);
    b.link(fine, FACTOR, height, 0, 0);
    b.num(height, 1, 0.3);
    b.link(peb_cut, 0, height, 2, 0);
    b.bump(height, 0, 0.8, 0.04, 6);
    b.g
}

/// Glowing cracks: Voronoi edges lit from inside, dark crust in between.
fn glowing_cracks(crust: [f32; 3], glow: [f32; 3], scale: f32, intensity: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(1.0);
    let jitter = b.node("Jitter coords", 1, &[2.0]);
    b.link(p, 0, jitter, 0, 0);
    b.num(jitter, 1, 0.25);
    let cells = b.voronoi(jitter, scale, 1.0, 1);
    let crack = b.node("Smoothstep", 2, &[0.08, 0.0]);
    b.link(cells, EDGE, crack, 0, 0);
    let pulse = b.node("Time", 2, &[]);
    let flicker = b.noise(p, 1.5, 3.0, 0.5, 0.0, 2);
    let live = b.math(MULADD, 3, false);
    b.link(pulse, 1, live, 0, 0);
    b.num(live, 1, 0.35);
    b.link(flicker, FACTOR, live, 2, 0);
    let heat = b.math(MUL, 4, false);
    b.link(crack, 0, heat, 0, 0);
    b.link(live, 0, heat, 1, 0);
    let fire = b.ramp(heat, 0, [[0.0; 3], [glow[0] * 0.4, glow[1] * 0.1, 0.0], glow, [1.0, 0.95, 0.7]], [0.0, 0.3, 0.7, 1.0], true, 5);
    let bright = b.math(MUL, 6, false);
    b.link(fire, 0, bright, 0, 0);
    b.num(bright, 1, intensity);
    b.out(bright, 0, EMIT);
    let rock = b.noise(p, scale * 2.0, 6.0, 0.6, 0.0, 3);
    let base = b.mix(MULTIPLY, 4);
    b.num(base, 0, 0.7);
    b.value(base, 1, crust);
    b.link(rock, COLOUR, base, 2, 0);
    b.out(base, 0, BASE);
    let r = b.map(crack, 0, 0.9, 0.5, 5);
    b.out(r, 0, ROUGH);
    b.out_value(METAL, 0.0);
    b.bump(crack, 0, -1.2, 0.05, 6);
    b.g
}

/// Car paint: base colour with metal flakes and a clear coat sheen at grazing angles.
fn car_paint(colour: [f32; 3], flake: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(1.0);
    let flakes = b.voronoi(p, 260.0, 1.0, 1);
    let sparkle = b.math(POW, 2, false);
    b.link(flakes, CELL, sparkle, 0, 0);
    b.num(sparkle, 1, 8.0);
    let lw = b.node("Layer weight", 1, &[]);
    b.num(lw, 0, 0.35);
    let body = b.mix(MIX, 3);
    b.link(lw, 1, body, 0, 8);
    b.value(body, 1, [colour[0] * 0.35, colour[1] * 0.35, colour[2] * 0.35]);
    b.value(body, 2, colour);
    let flaky = b.mix(ADD, 4);
    b.node_param(flaky, 0, 4.0);
    b.link(sparkle, 0, flaky, 0, 0);
    b.num(flaky, 0, flake);
    b.link(body, 0, flaky, 1, 0);
    b.value(flaky, 2, [0.25; 3]);
    b.out(flaky, 0, BASE);
    let r = b.map(sparkle, 0, 0.18, 0.05, 4);
    b.out(r, 0, ROUGH);
    b.out_value(METAL, 0.6);
    b.g
}

impl G {
    fn node_param(&mut self, id: u32, k: usize, v: f32) {
        self.g.nodes.iter_mut().find(|n| n.id == id).unwrap().params[k] = v;
    }
}

/// Fabric weave from two crossing wave textures, with fuzz.
fn fabric(a: [f32; 3], b_col: [f32; 3], scale: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.uv(scale);
    let sep = b.node("Separate XYZ", 1, &[]);
    b.link(p, 0, sep, 0, 0);
    let warp_threads = b.math(SIN, 2, false);
    b.link(sep, 1, warp_threads, 0, 0);
    let weft_threads = b.math(SIN, 2, false);
    b.link(sep, 2, weft_threads, 0, 0);
    let scale_x = b.math(MUL, 1, false);
    b.link(sep, 1, scale_x, 0, 0);
    b.num(scale_x, 1, 28.0);
    let scale_y = b.math(MUL, 1, false);
    b.link(sep, 2, scale_y, 0, 0);
    b.num(scale_y, 1, 28.0);
    b.link(scale_x, 0, warp_threads, 0, 0);
    b.link(scale_y, 0, weft_threads, 0, 0);
    let cross = b.math(MUL, 3, false);
    b.link(warp_threads, 0, cross, 0, 0);
    b.link(weft_threads, 0, cross, 1, 0);
    let weave = b.math(ABS, 3, false);
    b.link(cross, 0, weave, 0, 0);
    let cloth = b.mix(MIX, 4);
    b.link(weave, 0, cloth, 0, 0);
    b.value(cloth, 1, a);
    b.value(cloth, 2, b_col);
    let w = b.world(1.0);
    let fuzz = b.noise(w, 40.0, 3.0, 0.6, 0.0, 4);
    let fuzzy = b.mix(OVERLAY, 5);
    b.num(fuzzy, 0, 0.3);
    b.link(cloth, 0, fuzzy, 1, 0);
    b.link(fuzz, COLOUR, fuzzy, 2, 0);
    b.out(fuzzy, 0, BASE);
    b.out_value(ROUGH, 0.92);
    b.out_value(METAL, 0.0);
    b.bump(weave, 0, 0.4, 0.01, 5);
    b.g
}

/// Ceramic floor tiles with marble-like veins built in, glossy, with darker grout.
fn tiles_marble(base: [f32; 3], vein: [f32; 3], grout: [f32; 3], scale: f32) -> MaterialGraph {
    let g = marble(base, vein, 3.0, 2.2);
    // Put a tile grid over the marble: reuse the builder on the existing graph.
    let out = g.nodes.iter().find(|n| n.kind == "Output").unwrap().id;
    let base_link = g.nodes.iter().find(|n| n.id == out).unwrap().inputs[BASE].link.unwrap();
    let mut b = G { g, out, col: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 900.0] };
    let p = b.uv(scale);
    let grid = b.node("Brick texture", 7, &[1.0, 0.0, 0.02, 0.006, 0.0]);
    b.link(p, 0, grid, 0, 0);
    b.num(grid, 4, 4.0);
    let tiled = b.mix(MIX, 7);
    b.link(grid, MASK, tiled, 0, 0);
    b.value(tiled, 1, grout);
    b.link(base_link, 0, tiled, 2, 0);
    b.out(tiled, 0, BASE);
    let rough = b.map(grid, MASK, 0.8, 0.06, 7);
    b.out(rough, 0, ROUGH);
    b.bump(grid, MASK, 0.5, 0.02, 7);
    b.g
}

/// Stylised hand-painted rock: posterised noise with cool shadows and warm highlights.
fn painted_rock(dark: [f32; 3], light: [f32; 3]) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(1.0);
    let n = b.noise(p, 1.8, 3.0, 0.5, 0.6, 1);
    let ramp = b.node("Color ramp", 2, &[0.0, 0.4, 0.62, 1.0, 1.0]);
    b.link(n, FACTOR, ramp, 0, 0);
    b.value(ramp, 1, dark);
    b.value(ramp, 2, [(dark[0] + light[0]) * 0.45, (dark[1] + light[1]) * 0.45, (dark[2] + light[2]) * 0.5]);
    b.value(ramp, 3, light);
    b.value(ramp, 4, [light[0] * 1.15, light[1] * 1.12, light[2] * 1.0]);
    let up = b.node("Normal", 2, &[]);
    let top = b.mix(MIX, 3);
    b.link(up, NORMAL_UP, top, 0, 0);
    b.link(ramp, 0, top, 1, 0);
    b.value(top, 2, [light[0] * 1.2, light[1] * 1.25, light[2] * 1.1]);
    b.out(top, 0, BASE);
    b.out_value(ROUGH, 0.85);
    b.out_value(METAL, 0.0);
    b.bump(n, FACTOR, 0.6, 0.1, 4);
    b.g
}

/// Ice with deep blue cracks and frosty, rougher patches.
fn ice(clear: [f32; 3], deep: [f32; 3]) -> MaterialGraph {
    let mut b = G::new();
    let p = b.world(1.0);
    let cracks = b.voronoi(p, 3.0, 1.0, 1);
    let crack = b.node("Smoothstep", 2, &[0.04, 0.0]);
    b.link(cracks, EDGE, crack, 0, 0);
    let frost = b.noise(p, 6.0, 6.0, 0.6, 0.2, 1);
    let lw = b.node("Layer weight", 1, &[]);
    b.num(lw, 0, 0.4);
    let body = b.mix(MIX, 3);
    b.link(lw, 0, body, 0, 0);
    b.value(body, 1, deep);
    b.value(body, 2, clear);
    let cracked = b.mix(MIX, 4);
    b.link(crack, 0, cracked, 0, 0);
    b.link(body, 0, cracked, 1, 0);
    b.value(cracked, 2, [0.9, 0.97, 1.0]);
    b.out(cracked, 0, BASE);
    let r = b.map(frost, FACTOR, 0.02, 0.45, 4);
    b.out(r, 0, ROUGH);
    b.out_value(METAL, 0.0);
    let glow = b.math(MUL, 4, false);
    b.link(lw, 0, glow, 0, 0);
    b.value(glow, 1, [deep[0] * 0.3, deep[1] * 0.4, deep[2] * 0.6]);
    b.out(glow, 0, EMIT);
    b.bump(crack, 0, 0.6, 0.02, 5);
    b.g
}

/// Sci-fi panels: a grid of plates, glowing seams and small vents.
fn scifi(plate: [f32; 3], seam: [f32; 3], glow: f32) -> MaterialGraph {
    let mut b = G::new();
    let p = b.uv(3.0);
    let panels = b.node("Brick texture", 1, &[1.6, 0.0, 0.02, 0.004, 0.4]);
    b.link(p, 0, panels, 0, 0);
    b.num(panels, 4, 3.0);
    let small = b.node("Brick texture", 1, &[1.0, 0.0, 0.035, 0.004, 0.0]);
    b.link(p, 0, small, 0, 0);
    b.num(small, 4, 12.0);
    let tone = b.mix(MIX, 2);
    b.link(panels, BRICK_VALUE, tone, 0, 11);
    b.value(tone, 1, plate);
    b.value(tone, 2, [plate[0] * 0.75, plate[1] * 0.75, plate[2] * 0.8]);
    let col = b.mix(MULTIPLY, 3);
    b.link(small, MASK, col, 0, 8);
    b.num(col, 0, 0.25);
    b.link(tone, 0, col, 1, 0);
    b.value(col, 2, [0.4; 3]);
    let seams = b.math(SUB, 3, true);
    b.num(seams, 0, 1.0);
    b.link(panels, MASK, seams, 1, 0);
    b.out(col, 0, BASE);
    let time = b.node("Time", 2, &[]);
    let pulse = b.math(MULADD, 4, false);
    b.link(time, 1, pulse, 0, 0);
    b.num(pulse, 1, 0.5);
    b.num(pulse, 2, 0.6);
    let light = b.math(MUL, 5, false);
    b.link(seams, 0, light, 0, 0);
    b.link(pulse, 0, light, 1, 0);
    let emit = b.mix(MULTIPLY, 6);
    b.num(emit, 0, 1.0);
    b.link(light, 0, emit, 1, 0);
    b.value(emit, 2, [seam[0] * glow, seam[1] * glow, seam[2] * glow]);
    b.out(emit, 0, EMIT);
    let r = b.map(panels, BRICK_VALUE, 0.25, 0.5, 4);
    b.out(r, 0, ROUGH);
    b.out_value(METAL, 0.85);
    let height = b.math(ADD, 5, false);
    b.link(panels, MASK, height, 0, 0);
    b.link(small, MASK, height, 1, 8);
    b.bump(height, 0, 0.6, 0.02, 6);
    b.g
}

/// Every material of this file.
pub fn library() -> Vec<Preset> {
    let p = |name: &'static str, graph: MaterialGraph| Preset { category: "Game Ready", name, graph };
    vec![
        p("Marble Carrara", marble([0.9, 0.9, 0.88], [0.28, 0.3, 0.34], 3.0, 1.6)),
        p("Marble Nero", marble([0.025, 0.025, 0.03], [0.75, 0.72, 0.6], 3.5, 1.4)),
        p("Marble Verde", marble([0.04, 0.2, 0.13], [0.75, 0.85, 0.78], 4.0, 1.8)),
        p("Marble Rosso", marble([0.42, 0.12, 0.1], [0.9, 0.8, 0.75], 3.0, 2.0)),
        p("Marble Floor Tiles", tiles_marble([0.86, 0.85, 0.82], [0.32, 0.33, 0.36], [0.12, 0.12, 0.12], 2.0)),
        p("Red Brick Wall", brick_wall([0.42, 0.13, 0.07], [0.3, 0.09, 0.05], [0.55, 0.52, 0.47], 3.0, 0.15)),
        p("Mossy Stone Wall", brick_wall([0.32, 0.31, 0.29], [0.22, 0.22, 0.21], [0.35, 0.34, 0.3], 2.0, 0.9)),
        p("Sandstone Blocks", brick_wall([0.66, 0.5, 0.31], [0.55, 0.4, 0.24], [0.6, 0.52, 0.4], 1.5, 0.05)),
        p("Castle Wall", brick_wall([0.38, 0.37, 0.35], [0.27, 0.27, 0.27], [0.18, 0.17, 0.16], 1.2, 0.4)),
        p("Cobblestone Path", cobbles([0.3, 0.29, 0.27], [0.45, 0.42, 0.38], [0.05, 0.05, 0.04], 3.5)),
        p("River Pebbles", cobbles([0.42, 0.4, 0.37], [0.2, 0.18, 0.16], [0.08, 0.07, 0.05], 9.0)),
        p("Mossy Cobbles", cobbles([0.25, 0.27, 0.22], [0.36, 0.38, 0.3], [0.06, 0.12, 0.03], 4.0)),
        p("Oak Floor", planks([0.5, 0.33, 0.17], [0.25, 0.14, 0.06], 1.0, 0.2)),
        p("Weathered Planks", planks([0.42, 0.38, 0.32], [0.2, 0.18, 0.15], 1.0, 0.9)),
        p("Dark Walnut Floor", planks([0.22, 0.12, 0.06], [0.08, 0.04, 0.02], 1.2, 0.1)),
        p("Ship Deck", planks([0.6, 0.45, 0.28], [0.32, 0.22, 0.12], 0.8, 0.6)),
        p("Chipped Red Paint", chipped_paint([0.5, 0.06, 0.04], [0.32, 0.14, 0.06], true)),
        p("Chipped Blue Paint", chipped_paint([0.06, 0.18, 0.42], [0.55, 0.56, 0.58], false)),
        p("Military Green", chipped_paint([0.16, 0.2, 0.1], [0.4, 0.4, 0.42], false)),
        p("Industrial Yellow", chipped_paint([0.75, 0.52, 0.04], [0.35, 0.16, 0.07], true)),
        p("Forest Floor", ground([0.08, 0.05, 0.03], [0.14, 0.12, 0.05], [0.3, 0.28, 0.25], 2.0, 0.85)),
        p("Beach Sand", ground([0.62, 0.52, 0.36], [0.78, 0.68, 0.5], [0.5, 0.46, 0.4], 3.0, 0.9)),
        p("Gravel Road", ground([0.25, 0.24, 0.22], [0.35, 0.33, 0.3], [0.5, 0.48, 0.45], 4.0, 0.9)),
        p("Muddy Ground", ground([0.07, 0.05, 0.03], [0.16, 0.12, 0.07], [0.25, 0.22, 0.18], 2.5, 0.35)),
        p("Snowy Ground", ground([0.75, 0.8, 0.88], [0.95, 0.97, 1.0], [0.3, 0.3, 0.32], 2.0, 0.6)),
        p("Lava Rock", glowing_cracks([0.05, 0.04, 0.04], [1.0, 0.35, 0.05], 2.5, 6.0)),
        p("Magic Crystal Cracks", glowing_cracks([0.05, 0.04, 0.08], [0.45, 0.2, 1.0], 3.0, 5.0)),
        p("Toxic Cracks", glowing_cracks([0.04, 0.05, 0.03], [0.35, 1.0, 0.1], 3.0, 4.0)),
        p("Car Paint Red", car_paint([0.5, 0.02, 0.02], 0.35)),
        p("Car Paint Midnight", car_paint([0.02, 0.04, 0.12], 0.5)),
        p("Car Paint Pearl", car_paint([0.8, 0.8, 0.78], 0.25)),
        p("Denim", fabric([0.05, 0.09, 0.2], [0.18, 0.25, 0.4], 6.0)),
        p("Tweed", fabric([0.25, 0.18, 0.1], [0.4, 0.33, 0.22], 5.0)),
        p("Painted Rock", painted_rock([0.18, 0.2, 0.28], [0.62, 0.55, 0.45])),
        p("Painted Rock Warm", painted_rock([0.25, 0.15, 0.12], [0.85, 0.65, 0.42])),
        p("Glacier Ice", ice([0.75, 0.9, 1.0], [0.05, 0.3, 0.6])),
        p("Sci-Fi Panels", scifi([0.35, 0.37, 0.4], [0.2, 0.7, 1.0], 4.0)),
        p("Sci-Fi Panels Orange", scifi([0.22, 0.22, 0.24], [1.0, 0.45, 0.1], 4.0)),
    ]
}
