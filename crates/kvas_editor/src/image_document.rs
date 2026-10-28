//! Layered raster document. CPU-only, bounded to 2048² and sixteen layers.
use serde::{Deserialize, Serialize};

/// Max edge in pixels and max layer count of a document.
pub const MAX_SIZE: u32 = 2048;
pub const MAX_LAYERS: usize = 16;

/// How a layer is combined with the layers below it.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub enum Blend {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Darken,
    Lighten,
}

impl Blend {
    pub const ALL: [Blend; 7] = [Blend::Normal, Blend::Multiply, Blend::Screen, Blend::Overlay, Blend::Add, Blend::Darken, Blend::Lighten];

    pub fn name(self) -> &'static str {
        match self {
            Blend::Normal => "Обычный",
            Blend::Multiply => "Умножение",
            Blend::Screen => "Экран",
            Blend::Overlay => "Перекрытие",
            Blend::Add => "Сложение",
            Blend::Darken => "Затемнение",
            Blend::Lighten => "Осветление",
        }
    }

    fn mix(self, s: f32, d: f32) -> f32 {
        match self {
            Blend::Normal => s,
            Blend::Multiply => s * d,
            Blend::Screen => 1.0 - (1.0 - s) * (1.0 - d),
            Blend::Overlay => if d < 0.5 { 2.0 * s * d } else { 1.0 - 2.0 * (1.0 - s) * (1.0 - d) },
            Blend::Add => (s + d).min(1.0),
            Blend::Darken => s.min(d),
            Blend::Lighten => s.max(d),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub pixels: Vec<u8>,
    #[serde(default)]
    pub blend: Blend,
    /// Painting keeps the transparency of the pixels (only colours change).
    #[serde(default)]
    pub lock_alpha: bool,
}

impl Layer {
    fn empty(name: String, w: u32, h: u32) -> Layer {
        Layer { name, visible: true, opacity: 1.0, pixels: vec![0; w as usize * h as usize * 4], blend: Blend::Normal, lock_alpha: false }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageDocument {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
}

/// One brush dab: where, how big, which colour and how strongly.
#[derive(Clone, Copy)]
pub struct Dab {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [u8; 4],
    /// 0..1 strength of this dab (opacity x flow x pressure).
    pub strength: f32,
    pub hardness: f32,
    pub erase: bool,
}

impl ImageDocument {
    pub fn new(width: u32, height: u32) -> Self {
        let (width, height) = (width.clamp(1, MAX_SIZE), height.clamp(1, MAX_SIZE));
        Self { width, height, layers: vec![Layer::empty("Layer 1".into(), width, height)] }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 || self.width > MAX_SIZE || self.height > MAX_SIZE || self.layers.is_empty() || self.layers.len() > MAX_LAYERS {
            return Err(format!("document limit: 1..{MAX_SIZE} pixels per edge, 1..{MAX_LAYERS} layers"));
        }
        for layer in &self.layers {
            if layer.pixels.len() != self.width as usize * self.height as usize * 4 || !layer.opacity.is_finite() || !(0.0..=1.0).contains(&layer.opacity) {
                return Err("invalid image layer data".into());
            }
        }
        Ok(())
    }
    /// All visible layers combined, bottom to top, with their blend modes and opacities.
    pub fn composite(&self) -> Vec<u8> {
        let mut output = vec![0; self.width as usize * self.height as usize * 4];
        for layer in self.layers.iter().filter(|l| l.visible) {
            for (destination, source) in output.chunks_exact_mut(4).zip(layer.pixels.chunks_exact(4)) {
                blend_over(destination, source, layer.opacity, layer.blend);
            }
        }
        output
    }
    pub fn add_layer(&mut self) -> bool {
        if self.layers.len() >= MAX_LAYERS {
            return false;
        }
        self.layers.push(Layer::empty(format!("Layer {}", self.layers.len() + 1), self.width, self.height));
        true
    }
    /// A copy of layer `i` right above it.
    pub fn duplicate_layer(&mut self, i: usize) -> bool {
        if self.layers.len() >= MAX_LAYERS || i >= self.layers.len() {
            return false;
        }
        let mut copy = self.layers[i].clone();
        copy.name = format!("{} copy", copy.name);
        self.layers.insert(i + 1, copy);
        true
    }
    /// Combines layer `i` into the one below it (with its blend mode and opacity).
    pub fn merge_down(&mut self, i: usize) -> bool {
        if i == 0 || i >= self.layers.len() {
            return false;
        }
        let top = self.layers.remove(i);
        let below = &mut self.layers[i - 1];
        for (d, s) in below.pixels.chunks_exact_mut(4).zip(top.pixels.chunks_exact(4)) {
            if top.visible {
                blend_over(d, s, top.opacity, top.blend);
            }
        }
        true
    }
    /// The composite colour at a pixel (eyedropper).
    pub fn sample(&self, p: [f32; 2]) -> Option<[u8; 4]> {
        let (x, y) = (p[0].floor(), p[1].floor());
        if x < 0.0 || y < 0.0 || x >= self.width as f32 || y >= self.height as f32 {
            return None;
        }
        let mut out = [0u8; 4];
        let i = ((y as u32 * self.width + x as u32) * 4) as usize;
        for layer in self.layers.iter().filter(|l| l.visible) {
            blend_over(&mut out, &layer.pixels[i..i + 4], layer.opacity, layer.blend);
        }
        Some(out)
    }
    pub fn dab(&mut self, layer: usize, dab: Dab, selection: &[[f32; 2]]) {
        let Dab { center, radius, color, strength, hardness, erase } = dab;
        if !radius.is_finite() || radius <= 0.0 || center.iter().any(|v| !v.is_finite()) {
            return;
        }
        let Some(layer) = self.layers.get_mut(layer) else { return };
        let x0 = (center[0] - radius).floor().max(0.0) as u32;
        let y0 = (center[1] - radius).floor().max(0.0) as u32;
        let x1 = (center[0] + radius).ceil().min(self.width as f32) as u32;
        let y1 = (center[1] + radius).ceil().min(self.height as f32) as u32;
        for y in y0..y1 {
            for x in x0..x1 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                if selection.len() >= 3 && !inside_polygon(p, selection) {
                    continue;
                }
                let d = ((p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2)).sqrt() / radius;
                if d >= 1.0 {
                    continue;
                }
                let falloff = if d < hardness { 1.0 } else { ((1.0 - d) / (1.0 - hardness).max(0.001)).clamp(0.0, 1.0) };
                let k = falloff * strength.clamp(0.0, 1.0);
                let i = ((y * self.width + x) * 4) as usize;
                let pixel = &mut layer.pixels[i..i + 4];
                if erase {
                    pixel[3] = (pixel[3] as f32 * (1.0 - k * color[3] as f32 / 255.0)).round() as u8;
                } else if layer.lock_alpha {
                    let a = pixel[3];
                    let mut tmp = [pixel[0], pixel[1], pixel[2], 255];
                    blend_over(&mut tmp, &color, k, Blend::Normal);
                    pixel.copy_from_slice(&[tmp[0], tmp[1], tmp[2], a]);
                } else {
                    blend_over(pixel, &color, k, Blend::Normal);
                }
            }
        }
    }
    pub fn fill(&mut self, layer: usize, color: [u8; 4], selection: &[[f32; 2]]) {
        let Some(layer) = self.layers.get_mut(layer) else { return };
        for (i, pixel) in layer.pixels.chunks_exact_mut(4).enumerate() {
            let p = [(i as u32 % self.width) as f32 + 0.5, (i as u32 / self.width) as f32 + 0.5];
            if selection.len() < 3 || inside_polygon(p, selection) {
                pixel.copy_from_slice(&color);
            }
        }
    }
    /// Paint bucket: fills the connected area of similar colour (on the layer) around `start`.
    pub fn flood_fill(&mut self, layer: usize, start: [f32; 2], color: [u8; 4], tolerance: f32, selection: &[[f32; 2]]) {
        let (w, h) = (self.width as i64, self.height as i64);
        let (sx, sy) = (start[0].floor() as i64, start[1].floor() as i64);
        if sx < 0 || sy < 0 || sx >= w || sy >= h {
            return;
        }
        let Some(layer) = self.layers.get_mut(layer) else { return };
        let at = |x: i64, y: i64| ((y * w + x) * 4) as usize;
        let target: [u8; 4] = layer.pixels[at(sx, sy)..at(sx, sy) + 4].try_into().unwrap();
        let limit = (tolerance.clamp(0.0, 1.0) * 255.0 * 2.0) as i32;
        let close = |p: &[u8]| (0..4).map(|c| (p[c] as i32 - target[c] as i32).abs()).sum::<i32>() <= limit;
        let mut seen = vec![false; (w * h) as usize];
        let mut stack = vec![(sx, sy)];
        while let Some((x, y)) = stack.pop() {
            if x < 0 || y < 0 || x >= w || y >= h || seen[(y * w + x) as usize] {
                continue;
            }
            seen[(y * w + x) as usize] = true;
            let i = at(x, y);
            if !close(&layer.pixels[i..i + 4]) {
                continue;
            }
            if selection.len() >= 3 && !inside_polygon([x as f32 + 0.5, y as f32 + 0.5], selection) {
                continue;
            }
            layer.pixels[i..i + 4].copy_from_slice(&color);
            stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
        }
    }
    /// Linear gradient from `a` (colour `ca`) to `b` (colour `cb`) over the layer or selection.
    pub fn gradient(&mut self, layer: usize, a: [f32; 2], b: [f32; 2], ca: [u8; 4], cb: [u8; 4], selection: &[[f32; 2]]) {
        let Some(layer) = self.layers.get_mut(layer) else { return };
        let d = [b[0] - a[0], b[1] - a[1]];
        let len2 = (d[0] * d[0] + d[1] * d[1]).max(1e-4);
        for (i, pixel) in layer.pixels.chunks_exact_mut(4).enumerate() {
            let p = [(i as u32 % self.width) as f32 + 0.5, (i as u32 / self.width) as f32 + 0.5];
            if selection.len() >= 3 && !inside_polygon(p, selection) {
                continue;
            }
            let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2).clamp(0.0, 1.0);
            let c: [u8; 4] = std::array::from_fn(|k| (ca[k] as f32 + (cb[k] as f32 - ca[k] as f32) * t).round() as u8);
            blend_over(pixel, &c, 1.0, Blend::Normal);
        }
    }
    /// Clears the selection (or the whole layer) to transparent.
    pub fn clear(&mut self, layer: usize, selection: &[[f32; 2]]) {
        self.fill(layer, [0, 0, 0, 0], selection);
    }
}

fn blend_over(destination: &mut [u8], source: &[u8], opacity: f32, mode: Blend) {
    let sa = source[3] as f32 / 255.0 * opacity.clamp(0.0, 1.0);
    let da = destination[3] as f32 / 255.0;
    let alpha = sa + da * (1.0 - sa);
    for c in 0..3 {
        let (s, d) = (source[c] as f32 / 255.0, destination[c] as f32 / 255.0);
        // Where there is nothing below, the layer shows as it is.
        let mixed = if da > 0.0 { mode.mix(s, d) * da + s * (1.0 - da) } else { s };
        destination[c] = if alpha > 0.0 { ((mixed * sa + d * da * (1.0 - sa)) / alpha * 255.0).round().clamp(0.0, 255.0) as u8 } else { 0 };
    }
    destination[3] = (alpha * 255.0).round() as u8;
}

pub fn inside_polygon(point: [f32; 2], polygon: &[[f32; 2]]) -> bool {
    let mut inside = false;
    if polygon.len() < 3 {
        return true;
    }
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current[1] > point[1]) != (previous[1] > point[1]) && point[0] < (previous[0] - current[0]) * (point[1] - current[1]) / (previous[1] - current[1]) + current[0] {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dab(center: [f32; 2], radius: f32, color: [u8; 4], erase: bool) -> Dab {
        Dab { center, radius, color, strength: 1.0, hardness: 1.0, erase }
    }
    #[test]
    fn layers_composite_erase_and_selection_work() {
        let mut doc = ImageDocument::new(8, 8);
        doc.fill(0, [255, 0, 0, 255], &[]);
        doc.add_layer();
        let selection = [[0.0, 0.0], [4.0, 0.0], [4.0, 8.0], [0.0, 8.0]];
        doc.fill(1, [0, 0, 255, 255], &selection);
        doc.layers[1].opacity = 0.5;
        let pixels = doc.composite();
        assert_eq!(&pixels[..4], &[128, 0, 128, 255]);
        assert_eq!(&pixels[28..32], &[255, 0, 0, 255]);
        doc.dab(1, dab([1.5, 1.5], 1.0, [0, 0, 0, 255], true), &[]);
        assert_eq!(doc.layers[1].pixels[(8 + 1) * 4 + 3], 0);
        let loaded: ImageDocument = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
        loaded.validate().unwrap();
        assert_eq!(loaded.composite(), doc.composite());
    }
    #[test]
    fn blend_modes_flood_fill_merge_and_sample() {
        let mut doc = ImageDocument::new(4, 4);
        doc.fill(0, [128, 128, 128, 255], &[]);
        doc.add_layer();
        doc.fill(1, [128, 128, 128, 255], &[]);
        doc.layers[1].blend = Blend::Multiply;
        assert_eq!(doc.sample([1.0, 1.0]).unwrap()[0], 64);
        assert!(doc.merge_down(1));
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.layers[0].pixels[0], 64);
        // A wall splits the layer; the bucket stops at it.
        for y in 0..4 {
            doc.layers[0].pixels[((y * 4 + 2) * 4) as usize..((y * 4 + 2) * 4 + 4) as usize].copy_from_slice(&[0, 0, 0, 255]);
        }
        doc.flood_fill(0, [0.5, 0.5], [255, 0, 0, 255], 0.05, &[]);
        assert_eq!(&doc.layers[0].pixels[..4], &[255, 0, 0, 255]);
        assert_eq!(doc.layers[0].pixels[3 * 4], 64, "right of the wall untouched");
        doc.gradient(0, [0.0, 0.0], [4.0, 0.0], [0, 0, 0, 255], [255, 255, 255, 255], &[]);
        assert!(doc.layers[0].pixels[0] < doc.layers[0].pixels[3 * 4]);
    }
}
