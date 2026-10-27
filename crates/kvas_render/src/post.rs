//! Post-processing pass: reads the resolved scene colour and writes the finished picture.

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::wgpu;
use glam::{Mat4, Vec3};
use kvas_engine::fx::PostSettings;

const SHADER: &str = include_str!("shaders/post.wgsl");

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostUniform {
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
    d: [f32; 4],
    e: [f32; 4],
    f: [f32; 4],
    g: [f32; 4],
}

/// Lens settings of the scene camera that need the post pass (depth of field, bars, exposure).
#[derive(Default, Clone, Copy)]
pub(crate) struct Lens {
    pub dof: bool,
    pub focus_distance: f32,
    pub aperture: f32,
    pub focal_mm: f32,
    pub exposure_comp: f32,
    pub letterbox: f32,
}

impl Lens {
    /// Reads the primary camera of the scene.
    pub(crate) fn of(scene: &kvas_engine::scene::Scene, view: &crate::View) -> Self {
        let world = &scene.world;
        let Some((_, camera)) = world.iter::<kvas_engine::components::Camera>().find(|(_, c)| c.primary).or_else(|| world.iter::<kvas_engine::components::Camera>().next()) else { return Self::default() };
        // Focal length in mm for the blur maths: from the field of view when none is set (24 mm sensor height).
        let fov = view.proj.y_axis.y;
        let focal = if camera.focal_length > 1.0 { camera.focal_length } else if fov > 1e-4 { 12.0 * fov } else { 50.0 };
        Self { dof: camera.dof && !camera.orthographic, focus_distance: camera.focus_distance.max(0.1), aperture: camera.aperture.clamp(0.7, 32.0), focal_mm: focal, exposure_comp: camera.exposure_comp, letterbox: camera.letterbox.max(0.0) }
    }

    /// Whether the post pass has work to do for the lens alone.
    pub(crate) fn active(&self) -> bool {
        self.dof || self.letterbox > 0.0 || self.exposure_comp.abs() > 1e-3
    }
}

pub(crate) struct Post {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    /// View-projection of the previous frame, for motion blur.
    prev_vp: Option<Mat4>,
}

impl Post {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                // The scene's (multisampled) depth, for depth of field.
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: true },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("post"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("post"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("post"), size: std::mem::size_of::<PostUniform>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        Self { pipeline, layout, uniform, sampler, prev_vp: None }
    }

    /// Screen-space shift (in uv) of a point ten metres ahead since the last frame.
    pub(crate) fn forget(&mut self) {
        self.prev_vp = None;
    }

    fn motion(&mut self, vp: Mat4, eye: Vec3, forward: Vec3) -> [f32; 2] {
        let anchor = eye + forward * 10.0;
        let uv = |m: Mat4| {
            let p = m * anchor.extend(1.0);
            if p.w.abs() < 1e-4 { [0.0, 0.0] } else { [p.x / p.w * 0.5, -p.y / p.w * 0.5] }
        };
        let shift = match self.prev_vp {
            Some(prev) => {
                let (a, b) = (uv(prev), uv(vp));
                [a[0] - b[0], a[1] - b[1]]
            }
            None => [0.0, 0.0],
        };
        self.prev_vp = Some(vp);
        // A hard cut (teleport) should not smear the whole screen.
        if shift[0].abs() > 0.4 || shift[1].abs() > 0.4 { [0.0, 0.0] } else { shift }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, settings: &PostSettings, lens: &Lens, proj: Mat4, vp: Mat4, eye: Vec3, time: f32, input: &wgpu::TextureView, depth: &wgpu::TextureView, output: &wgpu::TextureView, size: [u32; 2]) {
        let forward = vp.inverse().transform_vector3(Vec3::NEG_Z).normalize_or_zero();
        let forward = if forward == Vec3::ZERO { Vec3::NEG_Z } else { forward };
        let motion = self.motion(vp, eye, forward);
        let (w, h) = (size[0].max(1) as f32, size[1].max(1) as f32);
        // Near and far planes back out of a DirectX-style perspective matrix.
        let (a, b) = (proj.z_axis.z, proj.w_axis.z);
        let (near, far) = if a.abs() > 1e-9 && (a + 1.0).abs() > 1e-9 { (b / a, b / (a + 1.0)) } else { (0.1, 500.0) };
        let u = PostUniform {
            a: [settings.bloom, settings.bloom_threshold.clamp(0.0, 1.5), settings.bloom_radius.clamp(0.2, 6.0), settings.lens_flare],
            b: [settings.blur, settings.motion_blur, settings.vignette, settings.chromatic_aberration],
            c: [settings.film_grain, settings.sharpen, time, w / h],
            d: [settings.tint[0], settings.tint[1], settings.tint[2], 0.0],
            e: [motion[0], motion[1], 1.0 / w, 1.0 / h],
            f: [lens.dof as i32 as f32, lens.focus_distance, lens.aperture, lens.focal_mm],
            g: [near, far, lens.exposure_comp, lens.letterbox],
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(input) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(depth) },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("post") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}

// ------------------------------------------------------------------ custom camera shaders

const CUSTOM_HEAD: &str = "
struct FxU { params: vec4<f32>, misc: vec4<f32> }
@group(0) @binding(0) var fx_src: texture_2d<f32>;
@group(0) @binding(1) var fx_smp: sampler;
@group(0) @binding(2) var<uniform> fx_u: FxU;
struct FxVs { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> FxVs {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    var o: FxVs;
    o.pos = vec4<f32>(p[i], 0.0, 1.0);
    o.uv = vec2<f32>(p[i].x * 0.5 + 0.5, 0.5 - p[i].y * 0.5);
    return o;
}
// Helpers available to every effect.
fn scene_color(uv: vec2<f32>) -> vec3<f32> { return textureSampleLevel(fx_src, fx_smp, uv, 0.0).rgb; }
fn screen_aspect() -> f32 { return fx_u.misc.y; }
fn pixel_size() -> vec2<f32> { return fx_u.misc.zw; }
";

/// Declaration file for editors: everything a camera shader may use (not compiled by the engine).
pub fn camera_api_stub() -> String {
    format!(
        "{}
// Camera shader API (generated by Kvas, do not edit; regenerated when the project opens).
//
// Add this as the first line of a camera shader so your editor knows every name:
//     #include \"shaders/kvas_camera_api.wgsl\"
// The engine ignores the line; it only helps the editor / linter.
//
// A camera shader defines ONE function, called for every pixel of the camera image:
//     fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32>
//   uv      0..1 position on screen (0,0 = top left)
//   color   the pixel as it is now (linear HDR)
//   params  the four numbers set in the Inspector (params.x is the effect strength)
//   time    seconds since the game started
// Return the new colour. scene_color(uv) reads any other pixel (blur, distortion...).
// Libraries: #include \"builtin:lib/color\", \"builtin:lib/noise\", \"builtin:lib/math\".
{CUSTOM_HEAD}
{}{}{}
",
        crate::shader_lib::STUB_MARKER,
        crate::shader_lib::LIB_COLOR,
        crate::shader_lib::LIB_NOISE,
        crate::shader_lib::LIB_MATH
    )
}

const CUSTOM_TAIL: &str = "
@fragment
fn fs_main(in: FxVs) -> @location(0) vec4<f32> {
    let c = scene_color(in.uv);
    return vec4<f32>(max(effect(in.uv, c, fx_u.params, fx_u.misc.x), vec3<f32>(0.0)), 1.0);
}
";

struct CachedEffect {
    stamp: Option<std::time::SystemTime>,
    pipeline: Option<wgpu::RenderPipeline>,
}

/// Runs user-written WGSL effects (`fn effect(uv, color, params, time) -> vec3`) on the camera image.
pub(crate) struct CustomPost {
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    format: wgpu::TextureFormat,
    cache: std::collections::HashMap<String, CachedEffect>,
    tick: u64,
}

/// The full WGSL of an effect (helpers, included libraries, the effect and the entry points).
pub(crate) fn custom_source(assets: &kvas_engine::assets::Assets, path: &str) -> Result<String, String> {
    let mut user = assets.read_string(path).map_err(|e| format!("{path}: {e}"))?;
    if path.ends_with(".kfx") {
        let graph = kvas_engine::material_graph::MaterialGraph::from_json(&user).map_err(|e| format!("{path}: {e}"))?;
        let body = kvas_engine::material_graph::to_wgsl(&graph).map_err(|e| format!("{path}: {e}"))?;
        user = format!("{}{}{}{}{}", crate::shader_lib::LIB_COLOR, crate::shader_lib::LIB_NOISE, crate::shader_lib::LIB_MATH, crate::shader_lib::LIB_NODES, body);
    }
    let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
    let user = crate::shader_lib::expand(assets, &user, dir).map_err(|e| format!("{path}: {e}"))?;
    Ok(format!("{CUSTOM_HEAD}\n{user}\n{CUSTOM_TAIL}"))
}

impl CustomPost {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom-post"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("custom-post"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("custom-post"), size: 32, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        Self { layout, pipeline_layout, uniform, sampler, format, cache: Default::default(), tick: 0 }
    }

    fn compile(&self, device: &wgpu::Device, assets: &kvas_engine::assets::Assets, path: &str) -> Result<wgpu::RenderPipeline, String> {
        let source = custom_source(assets, path)?;
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("custom-post"), source: wgpu::ShaderSource::Wgsl(source.into()) });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("custom-post"),
            layout: Some(&self.pipeline_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: self.format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        match pollster::block_on(scope.pop()) {
            Some(err) => Err(format!("effect {path}: {err}")),
            None => Ok(pipeline),
        }
    }

    /// Applies one effect. `Ok(false)` means the shader is broken (already reported) and was skipped.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &kvas_engine::assets::Assets,
        path: &str,
        params: [f32; 4],
        time: f32,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
        size: [u32; 2],
    ) -> Result<bool, String> {
        self.tick += 1;
        let stale = match self.cache.get(path) {
            None => true,
            // Hot reload: look at the file's timestamp every half second or so.
            Some(c) => self.tick % 30 == 0 && assets.modified(path) != c.stamp,
        };
        let mut error = None;
        if stale {
            let stamp = assets.modified(path);
            let pipeline = match self.compile(device, assets, path) {
                Ok(p) => Some(p),
                Err(e) => {
                    error = Some(e);
                    None
                }
            };
            self.cache.insert(path.to_string(), CachedEffect { stamp, pipeline });
        }
        let Some(pipeline) = self.cache.get(path).and_then(|c| c.pipeline.as_ref()) else {
            return match error {
                Some(e) => Err(e),
                None => Ok(false),
            };
        };
        let (w, h) = (size[0].max(1) as f32, size[1].max(1) as f32);
        let data: [f32; 8] = [params[0], params[1], params[2], params[3], time, w / h, 1.0 / w, 1.0 / h];
        queue.write_buffer(&self.uniform, 0, bytemuck::cast_slice(&data));
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom-post"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(input) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: self.uniform.as_entire_binding() },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("custom-post") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("custom-post"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
        Ok(true)
    }
}
