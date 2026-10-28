//! Small layered raster document. CPU-only, bounded to 1024² and eight layers.
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageDocument {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
}

impl ImageDocument {
    pub fn new(width: u32, height: u32) -> Self {
        let (width, height) = (width.clamp(1, 1024), height.clamp(1, 1024));
        Self { width, height, layers: vec![Layer { name: "Layer 1".into(), visible: true, opacity: 1.0, pixels: vec![0; width as usize * height as usize * 4] }] }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 || self.width > 1024 || self.height > 1024 || self.layers.is_empty() || self.layers.len() > 8 { return Err("document limit: 1..1024 pixels per edge, 1..8 layers".into()); }
        for layer in &self.layers {
            if layer.pixels.len() != self.width as usize * self.height as usize * 4 || !layer.opacity.is_finite() || !(0.0..=1.0).contains(&layer.opacity) { return Err("invalid image layer data".into()); }
        }
        Ok(())
    }
    pub fn composite(&self) -> Vec<u8> {
        let mut output = vec![0; self.width as usize * self.height as usize * 4];
        for layer in self.layers.iter().filter(|l| l.visible) {
            for (destination, source) in output.chunks_exact_mut(4).zip(layer.pixels.chunks_exact(4)) { over(destination, source, layer.opacity); }
        }
        output
    }
    pub fn add_layer(&mut self) -> bool {
        if self.layers.len() >= 8 { return false; }
        self.layers.push(Layer { name: format!("Layer {}", self.layers.len() + 1), visible: true, opacity: 1.0, pixels: vec![0; self.width as usize * self.height as usize * 4] }); true
    }
    // Selection coordinates are in image pixel space; an empty polygon means the whole image.
    pub fn dab(&mut self, layer: usize, center: [f32; 2], radius: f32, color: [u8; 4], erase: bool, hardness: f32, selection: &[[f32; 2]]) {
        if !radius.is_finite() || radius <= 0.0 || center.iter().any(|v| !v.is_finite()) { return; }
        let Some(layer) = self.layers.get_mut(layer) else { return };
        let x0 = (center[0] - radius).floor().max(0.0) as u32;
        let y0 = (center[1] - radius).floor().max(0.0) as u32;
        let x1 = (center[0] + radius).ceil().min(self.width as f32) as u32;
        let y1 = (center[1] + radius).ceil().min(self.height as f32) as u32;
        for y in y0..y1 { for x in x0..x1 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            if selection.len() >= 3 && !inside_polygon(p, selection) { continue; }
            let d = ((p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2)).sqrt() / radius;
            if d >= 1.0 { continue; }
            let falloff = if d < hardness { 1.0 } else { ((1.0 - d) / (1.0 - hardness).max(0.001)).clamp(0.0, 1.0) };
            let i = ((y * self.width + x) * 4) as usize; let pixel = &mut layer.pixels[i..i + 4];
            if erase { pixel[3] = (pixel[3] as f32 * (1.0 - falloff * color[3] as f32 / 255.0)).round() as u8; }
            else { over(pixel, &color, falloff); }
        } }
    }
    pub fn fill(&mut self, layer: usize, color: [u8; 4], selection: &[[f32; 2]]) {
        let Some(layer) = self.layers.get_mut(layer) else { return };
        for (i, pixel) in layer.pixels.chunks_exact_mut(4).enumerate() {
            let p = [(i as u32 % self.width) as f32 + 0.5, (i as u32 / self.width) as f32 + 0.5];
            if selection.len() < 3 || inside_polygon(p, selection) { pixel.copy_from_slice(&color); }
        }
    }
}

fn over(destination: &mut [u8], source: &[u8], opacity: f32) {
    let sa = source[3] as f32 / 255.0 * opacity.clamp(0.0, 1.0); let da = destination[3] as f32 / 255.0;
    let alpha = sa + da * (1.0 - sa);
    for c in 0..3 { destination[c] = if alpha > 0.0 { ((source[c] as f32 * sa + destination[c] as f32 * da * (1.0 - sa)) / alpha).round() as u8 } else { 0 }; }
    destination[3] = (alpha * 255.0).round() as u8;
}

pub fn inside_polygon(point: [f32; 2], polygon: &[[f32; 2]]) -> bool {
    let mut inside = false;
    if polygon.len() < 3 { return true; }
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current[1] > point[1]) != (previous[1] > point[1]) && point[0] < (previous[0] - current[0]) * (point[1] - current[1]) / (previous[1] - current[1]) + current[0] { inside = !inside; }
        previous = current;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layers_composite_erase_and_selection_work() {
        let mut doc = ImageDocument::new(8, 8); doc.fill(0, [255, 0, 0, 255], &[]); doc.add_layer();
        let selection = [[0.0, 0.0], [4.0, 0.0], [4.0, 8.0], [0.0, 8.0]];
        doc.fill(1, [0, 0, 255, 255], &selection); doc.layers[1].opacity = 0.5;
        let pixels = doc.composite(); assert_eq!(&pixels[..4], &[128, 0, 128, 255]); assert_eq!(&pixels[28..32], &[255, 0, 0, 255]);
        doc.dab(1, [1.5, 1.5], 1.0, [0, 0, 0, 255], true, 1.0, &[]);
        assert_eq!(doc.layers[1].pixels[(8 + 1) * 4 + 3], 0);
        let loaded: ImageDocument = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap(); loaded.validate().unwrap();
        assert_eq!(loaded.composite(), doc.composite());
    }
}
