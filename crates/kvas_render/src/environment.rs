//! HDR equirectangular environments. Manual bilinear reads keep RGBA32F portable.
use crate::cache::AssetCache;
use eframe::egui_wgpu::wgpu;
use kvas_engine::assets::Assets;

pub(crate) struct Environment {
    pub cache: AssetCache<(wgpu::TextureView, [f32; 3])>,
    pub fallback: wgpu::TextureView,
}

fn upload(device: &wgpu::Device, queue: &wgpu::Queue, size: [u32; 2], values: &[f32]) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("HDR-environment"), size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba32Float, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
    });
    queue.write_texture(wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, bytemuck::cast_slice(values), wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size[0] * 16), rows_per_image: Some(size[1]) }, texture.size());
    texture.create_view(&Default::default())
}

impl Environment {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self { Self { cache: Default::default(), fallback: upload(device, queue, [1, 1], &[0.0, 0.0, 0.0, 1.0]) } }

    pub fn select(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, assets: &Assets, errors: &mut Vec<String>, path: &str) -> (wgpu::TextureView, bool, [f32; 3]) {
        if path.is_empty() { return (self.fallback.clone(), false, [0.0; 3]); }
        self.cache.load(path, assets, errors, || {
            let bytes = assets.read(path).map_err(|e| format!("environment {path}: {e}"))?;
            let mut image = image::load_from_memory(&bytes).map_err(|e| format!("environment {path}: {e}"))?;
            if image.width() > 2048 || image.height() > 1024 { image = image.resize(2048, 1024, image::imageops::FilterType::Triangle); }
            let size = [image.width(), image.height()];
            let mut rgba = image.to_rgba32f().into_raw();
            let linear = path.to_ascii_lowercase().ends_with(".hdr");
            let mut average = [0.0f64; 3]; let mut weight = 0.0f64;
            for (i, pixel) in rgba.chunks_exact_mut(4).enumerate() {
                let y = i as u32 / size[0];
                let w = (((y as f64 + 0.5) / size[1] as f64) * std::f64::consts::PI).sin();
                for c in 0..3 {
                    let v = if pixel[c].is_finite() { pixel[c].clamp(0.0, 65504.0) } else { 0.0 };
                    pixel[c] = if linear { v } else if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
                    average[c] += pixel[c] as f64 * w;
                }
                pixel[3] = 1.0; weight += w;
            }
            Ok((upload(device, queue, size, &rgba), average.map(|v| (v / weight.max(1e-10)) as f32)))
        });
        self.cache.get(path).map(|(v, avg)| (v.clone(), true, *avg)).unwrap_or((self.fallback.clone(), false, [0.0; 3]))
    }
}
