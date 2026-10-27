//! Experimental x2 presentation: block optical flow, midpoint, then the real frame.
//! Keeps simulation at the shell's cadence. Adds one rendered-frame of visual latency.
use eframe::egui_wgpu::wgpu;
use glam::Mat4;

const SHADER: &str = include_str!("shaders/interpolate.wgsl");
const COPY: &str = "@group(0) @binding(0) var src: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
 let p = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0)); return vec4<f32>(p[id],0.0,1.0); }
@fragment fn fs_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> { return textureLoad(src, vec2<i32>(p.xy), 0); }";

struct History {
    size: [u32; 2],
    previous: wgpu::TextureView,
    current: wgpu::TextureView,
    flow: wgpu::TextureView,
    output: wgpu::TextureView,
    valid: bool,
}

pub(crate) struct Interpolator {
    copy_layout: wgpu::BindGroupLayout,
    flow_layout: wgpu::BindGroupLayout,
    synth_layout: wgpu::BindGroupLayout,
    copy: wgpu::RenderPipeline,
    flow: wgpu::RenderPipeline,
    synth: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    history: Option<History>,
    pending_real: bool,
    last_time: f32,
    last_vp: Mat4,
    pub generated: u64,
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None }
}

fn pipeline(device: &wgpu::Device, label: &str, source: &str, entry: &str, layout: &wgpu::BindGroupLayout, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(label), source: wgpu::ShaderSource::Wgsl(source.into()) });
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(label), bind_group_layouts: &[Some(layout)], immediate_size: 0 });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label), layout: Some(&pl),
        vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
        fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(entry), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
        primitive: Default::default(), depth_stencil: None, multisample: Default::default(), multiview_mask: None, cache: None,
    })
}

fn pass(encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView, pipeline: &wgpu::RenderPipeline, bind: &wgpu::BindGroup) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("temporal-presentation"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: output, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
        depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
    });
    pass.set_pipeline(pipeline); pass.set_bind_group(0, bind, &[]); pass.draw(0..3, 0..1);
}

impl Interpolator {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler_entry = wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None };
        let copy_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("temporal-copy"), entries: &[texture_entry(0)] });
        let flow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("optical-flow"), entries: &[texture_entry(0), texture_entry(1), sampler_entry] });
        let synth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("interpolation"), entries: &[texture_entry(0), texture_entry(1), sampler_entry, texture_entry(3)] });
        Self {
            copy: pipeline(device, "temporal-copy", COPY, "fs_main", &copy_layout, super::COLOR_FORMAT),
            flow: pipeline(device, "optical-flow", SHADER, "fs_flow", &flow_layout, wgpu::TextureFormat::Rgba16Float),
            synth: pipeline(device, "interpolation", SHADER, "fs_interpolate", &synth_layout, super::COLOR_FORMAT),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() }),
            copy_layout, flow_layout, synth_layout, history: None, pending_real: false, last_time: 0.0, last_vp: Mat4::IDENTITY, generated: 0,
        }
    }

    pub fn reset(&mut self) { self.history = None; self.pending_real = false; }

    pub fn take_real(&mut self, size: [u32; 2], time: f32, vp: Mat4) -> Option<wgpu::TextureView> {
        let valid = time >= self.last_time && time - self.last_time < 0.25 && (vp - self.last_vp).to_cols_array().iter().all(|x| x.abs() < 0.5);
        if self.pending_real && valid && self.history.as_ref().is_some_and(|h| h.size == size) {
            self.pending_real = false;
            return self.history.as_ref().map(|h| h.current.clone());
        }
        self.pending_real = false;
        None
    }

    pub fn apply(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, input: &wgpu::TextureView, size: [u32; 2], time: f32, vp: Mat4) -> wgpu::TextureView {
        if self.history.as_ref().is_none_or(|h| h.size != size) {
            let tex = |label, size: [u32; 2], format| device.create_texture(&wgpu::TextureDescriptor { label: Some(label), size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[] }).create_view(&Default::default());
            self.history = Some(History { size, previous: tex("previous-frame", size, super::COLOR_FORMAT), current: tex("current-frame", size, super::COLOR_FORMAT), flow: tex("block-flow", [size[0].div_ceil(16), size[1].div_ceil(16)], wgpu::TextureFormat::Rgba16Float), output: tex("generated-frame", size, super::COLOR_FORMAT), valid: false });
        }
        let h = self.history.as_mut().unwrap();
        let cut = time < self.last_time || time - self.last_time >= 0.25 || (vp - self.last_vp).to_cols_array().iter().any(|x| x.abs() >= 0.5);
        let synthesize = h.valid && !cut;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame-generation") });
        let copy_bind = |view| device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("frame-copy"), layout: &self.copy_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view) }] });
        if h.valid { pass(&mut encoder, &h.previous, &self.copy, &copy_bind(&h.current)); }
        pass(&mut encoder, &h.current, &self.copy, &copy_bind(input));
        if synthesize {
            let mut entries = vec![wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&h.previous) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&h.current) }, wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) }];
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("flow"), layout: &self.flow_layout, entries: &entries });
            pass(&mut encoder, &h.flow, &self.flow, &bg);
            entries.push(wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&h.flow) });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("synthesis"), layout: &self.synth_layout, entries: &entries });
            pass(&mut encoder, &h.output, &self.synth, &bg);
            self.generated += 1;
        }
        h.valid = true; self.pending_real = synthesize; self.last_time = time; self.last_vp = vp;
        queue.submit([encoder.finish()]);
        if synthesize { h.output.clone() } else { h.current.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn temporal_shaders_validate_without_gpu() {
        for src in [super::SHADER, super::COPY] {
            let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
            naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all()).validate(&module).unwrap();
        }
    }

    #[test]
    #[ignore = "requires a native GPU; bounded 32x32 optical-flow readback"]
    fn midpoint_pixels_pacing_resize_and_camera_cut_work_on_gpu() {
        let renderer = crate::Renderer::new_headless(kvas_engine::assets::Assets::Dir(std::env::temp_dir())).unwrap();
        let (device, queue) = (&renderer.device, &renderer.queue);
        let mut pass = Interpolator::new(device);
        let input = device.create_texture(&wgpu::TextureDescriptor { label: Some("test-input"), size: wgpu::Extent3d { width: 32, height: 32, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: crate::COLOR_FORMAT, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[] });
        let view = input.create_view(&Default::default());
        let upload = |v: u8| {
            let pixels = [v, v, v, 255].repeat(32 * 32);
            queue.write_texture(wgpu::TexelCopyTextureInfo { texture: &input, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, &pixels, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(128), rows_per_image: Some(32) }, input.size());
        };
        upload(100); pass.apply(device, queue, &view, [32, 32], 0.0, Mat4::IDENTITY);
        assert_eq!(pass.generated, 0);
        upload(110); let output = pass.apply(device, queue, &view, [32, 32], 0.033, Mat4::IDENTITY);
        assert_eq!(pass.generated, 1);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 256 * 32, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo { texture: output.texture(), mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, wgpu::TexelCopyBufferInfo { buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(256), rows_per_image: Some(32) } }, input.size());
        let index = queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| { tx.send(r).unwrap(); });
        device.poll(wgpu::PollType::Wait { submission_index: Some(index), timeout: Some(std::time::Duration::from_secs(10)) }).unwrap();
        rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap().unwrap();
        let pixels = buffer.slice(..).get_mapped_range().unwrap();
        assert!((pixels[0] as i32 - 105).abs() <= 1, "midpoint pixel {}", pixels[0]);
        drop(pixels); buffer.unmap();
        assert!(pass.take_real([32, 32], 0.05, Mat4::IDENTITY).is_some());
        assert!(pass.take_real([32, 32], 0.05, Mat4::IDENTITY).is_none());
        pass.apply(device, queue, &view, [32, 32], 1.0, Mat4::IDENTITY);
        assert_eq!(pass.generated, 1, "long pause resets interpolation");
        pass.apply(device, queue, &view, [32, 32], 1.033, Mat4::from_translation(glam::Vec3::X * 10.0));
        assert_eq!(pass.generated, 1, "camera cut resets interpolation");
        pass.reset(); assert!(pass.history.is_none());
    }
}
