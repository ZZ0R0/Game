#![allow(dead_code)]

use std::{
    path::PathBuf,
    time::SystemTime,
};

use bytemuck::{Pod, Zeroable};
use egui::{Context as EguiCtx, ViewportId};
use egui_wgpu::{Renderer as EguiRenderer};
use egui_winit::State as EguiWinit;
use glam::{Mat4, Vec3};
use crate::winit::{dpi::PhysicalSize, window::Window};

use crate::pipeline::create_pipeline_with_shader;
use crate::texture::{create_depth_view, make_checker_texture};

pub use egui_wgpu::ScreenDescriptor;

pub struct Gfx<'w> {
    pub(crate) surface: crate::wgpu::Surface<'w>,
    pub(crate) device: crate::wgpu::Device,
    pub(crate) queue: crate::wgpu::Queue,
    pub(crate) config: crate::wgpu::SurfaceConfiguration,
    pub(crate) present_modes: Vec<crate::wgpu::PresentMode>,
    pub(crate) size: PhysicalSize<u32>,
    pub(crate) depth_view: crate::wgpu::TextureView,

    pub(crate) adapter_name: String,

    pub(crate) cam_layout: crate::wgpu::BindGroupLayout,
    pub(crate) tex_layout: crate::wgpu::BindGroupLayout,
    pub(crate) obj_layout: crate::wgpu::BindGroupLayout,

    pub(crate) pipeline: crate::wgpu::RenderPipeline,

    pub(crate) vbuf: crate::wgpu::Buffer,
    pub(crate) ibuf: crate::wgpu::Buffer,
    pub(crate) index_count: u32,

    pub(crate) cam_eye: Vec3,
    pub(crate) cam_target: Vec3,
    pub cam_yaw: f32,
    pub cam_pitch: f32,
    pub(crate) cam_buf: crate::wgpu::Buffer,
    pub(crate) cam_bg: crate::wgpu::BindGroup,

    pub(crate) obj_buf: crate::wgpu::Buffer,
    pub(crate) obj_bg: crate::wgpu::BindGroup,

    pub(crate) tex_view: crate::wgpu::TextureView,
    pub(crate) tex_sampler: crate::wgpu::Sampler,
    pub(crate) tex_bg: crate::wgpu::BindGroup,

    pub(crate) egui_ctx: EguiCtx,
    pub(crate) egui_state: EguiWinit,
    pub(crate) egui_painter: EguiRenderer,

    pub(crate) hot: Option<HotReload>,
    pub(crate) last_img: Option<String>,
    pub(crate) hud_fps: Option<f32>,

    // object state
    pub(crate) model: Mat4,
    pub(crate) tint: [f32; 4],
}

#[derive(Clone)]
pub(crate) struct HotReload {
    pub(crate) path: PathBuf,
    pub(crate) mtime: SystemTime,
    pub(crate) last_error: Option<String>,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct CameraUBO {
    pub vp: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct ObjectUBO {
    pub model: [[f32; 4]; 4],
    pub tint: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct VertexTex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}
pub(crate) const VERT_TEX_ATTRS: [crate::wgpu::VertexAttribute; 2] =
    crate::wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

impl VertexTex {
    pub(crate) fn layout() -> crate::wgpu::VertexBufferLayout<'static> {
        crate::wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<VertexTex>() as u64,
            step_mode: crate::wgpu::VertexStepMode::Vertex,
            attributes: &VERT_TEX_ATTRS,
        }
    }
}

pub const EMBEDDED_SHADER: &str = r#"
struct Camera { vp: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct Object { model: mat4x4<f32>, tint: vec4<f32> }
@group(2) @binding(0) var<uniform> object: Object;

struct VSIn { @location(0) pos: vec3<f32>, @location(1) uv: vec2<f32> }
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> }

@vertex
fn vs_main(in: VSIn) -> VSOut {
  var out: VSOut;
  out.pos = camera.vp * object.model * vec4<f32>(in.pos, 1.0);
  out.uv = in.uv;
  return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
  return textureSample(tex, samp, in.uv) * object.tint;
}
"#;

impl<'w> Gfx<'w> {
    pub fn new(window: &'w Window) -> Self {
        use crate::wgpu::{Instance, PresentMode};
        use crate::wgpu::util::DeviceExt;

        let size = window.inner_size();

        let instance = Instance::default();
        let surface = instance.create_surface(window).expect("surface");

        let adapter = pollster::block_on(instance.request_adapter(&crate::wgpu::RequestAdapterOptions {
            power_preference: crate::wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("adapter");

        let info = adapter.get_info();
        let adapter_name = info.name.clone();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &crate::wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: crate::wgpu::Features::empty(),
                required_limits: crate::wgpu::Limits::default(),
                memory_hints: crate::wgpu::MemoryHints::default(),
            },
            None,
        ))
        .expect("device");

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let present = if caps.present_modes.contains(&PresentMode::Mailbox) {
            PresentMode::Mailbox
        } else {
            PresentMode::Fifo
        };

        let config = crate::wgpu::SurfaceConfiguration {
            usage: crate::wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: present,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_view = create_depth_view(&device, config.width, config.height);

        // Bind group layouts
        let cam_layout = device.create_bind_group_layout(&crate::wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bgl"),
            entries: &[crate::wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: crate::wgpu::ShaderStages::VERTEX,
                ty: crate::wgpu::BindingType::Buffer {
                    ty: crate::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: core::num::NonZeroU64::new(
                        core::mem::size_of::<CameraUBO>() as u64
                    ),
                },
                count: None,
            }],
        });

        let tex_layout = device.create_bind_group_layout(&crate::wgpu::BindGroupLayoutDescriptor {
            label: Some("tex_bgl"),
            entries: &[
                crate::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: crate::wgpu::ShaderStages::FRAGMENT,
                    ty: crate::wgpu::BindingType::Texture {
                        sample_type: crate::wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: crate::wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                crate::wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: crate::wgpu::ShaderStages::FRAGMENT,
                    ty: crate::wgpu::BindingType::Sampler(crate::wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let obj_layout = device.create_bind_group_layout(&crate::wgpu::BindGroupLayoutDescriptor {
            label: Some("obj_bgl"),
            entries: &[crate::wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: crate::wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: crate::wgpu::BindingType::Buffer {
                    ty: crate::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: core::num::NonZeroU64::new(
                        core::mem::size_of::<ObjectUBO>() as u64
                    ),
                },
                count: None,
            }],
        });

        // Camera + object buffers
        let cam_eye = Vec3::new(32.0, 20.0, 32.0);  // Higher and further to see 2x2 chunk grid
        // cam_target will be calculated from yaw/pitch
        let cam_yaw = -std::f32::consts::PI * 0.75f32;  // Regarde vers le centre
        let cam_pitch = -0.6f32;  // Regarde vers le bas
        
        // Calculate initial target from angles
        let forward = Vec3::new(
            cam_yaw.cos() * cam_pitch.cos(),
            cam_pitch.sin(),
            cam_yaw.sin() * cam_pitch.cos(),
        );
        let cam_target = cam_eye + forward;
        let cam_buf = device.create_buffer(&crate::wgpu::BufferDescriptor {
            label: Some("camera_buf"),
            size: core::mem::size_of::<CameraUBO>() as u64,
            usage: crate::wgpu::BufferUsages::UNIFORM | crate::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cam_bg = device.create_bind_group(&crate::wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &cam_layout,
            entries: &[crate::wgpu::BindGroupEntry {
                binding: 0,
                resource: cam_buf.as_entire_binding(),
            }],
        });

        let model = Mat4::IDENTITY;
        let tint = [1.0, 1.0, 1.0, 1.0];
        let obj_buf = device.create_buffer(&crate::wgpu::BufferDescriptor {
            label: Some("obj_buf"),
            size: core::mem::size_of::<ObjectUBO>() as u64,
            usage: crate::wgpu::BufferUsages::UNIFORM | crate::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let obj_bg = device.create_bind_group(&crate::wgpu::BindGroupDescriptor {
            label: Some("obj_bg"),
            layout: &obj_layout,
            entries: &[crate::wgpu::BindGroupEntry {
                binding: 0,
                resource: obj_buf.as_entire_binding(),
            }],
        });

        // Texture
        let (tex_view, tex_sampler) = make_checker_texture(&device, &queue, 128, 128);
        let tex_bg = device.create_bind_group(&crate::wgpu::BindGroupDescriptor {
            label: Some("tex_bg"),
            layout: &tex_layout,
            entries: &[
                crate::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: crate::wgpu::BindingResource::TextureView(&tex_view),
                },
                crate::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: crate::wgpu::BindingResource::Sampler(&tex_sampler),
                },
            ],
        });

        // Pipeline
        let shader = device.create_shader_module(crate::wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: crate::wgpu::ShaderSource::Wgsl(EMBEDDED_SHADER.into()),
        });
        let pipeline = create_pipeline_with_shader(
            &device,
            &[&cam_layout, &tex_layout, &obj_layout],
            &shader,
            surface_format,
        );

        // Geometry
        let verts = [
            VertexTex { pos: [-0.8, -0.6, 0.0], uv: [0.0, 1.0] },
            VertexTex { pos: [ 0.8, -0.6, 0.0], uv: [1.0, 1.0] },
            VertexTex { pos: [ 0.8,  0.6, 0.0], uv: [1.0, 0.0] },
            VertexTex { pos: [-0.8,  0.6, 0.0], uv: [0.0, 0.0] },
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vbuf = device.create_buffer_init(&crate::wgpu::util::BufferInitDescriptor {
            label: Some("quad_vbuf"),
            contents: bytemuck::cast_slice(&verts),
            usage: crate::wgpu::BufferUsages::VERTEX,
        });
        let ibuf = device.create_buffer_init(&crate::wgpu::util::BufferInitDescriptor {
            label: Some("quad_ibuf"),
            contents: bytemuck::cast_slice(&indices),
            usage: crate::wgpu::BufferUsages::INDEX,
        });

        // egui
        let egui_ctx = EguiCtx::default();
        let egui_state = EguiWinit::new(egui_ctx.clone(), ViewportId::ROOT, window, None, None, None);
        let egui_painter = EguiRenderer::new(&device, surface_format, None, 1, false);

        let mut gfx = Self {
            surface,
            device,
            queue,
            config,
            present_modes: caps.present_modes.clone(),
            size,
            depth_view,
            adapter_name,
            cam_layout,
            tex_layout,
            obj_layout,
            pipeline,
            vbuf,
            ibuf,
            index_count: indices.len() as u32,
            cam_eye,
            cam_target,
            cam_yaw,
            cam_pitch,
            cam_buf,
            cam_bg,
            obj_buf,
            obj_bg,
            tex_view,
            tex_sampler,
            tex_bg,
            egui_ctx,
            egui_state,
            egui_painter,
            hot: None,
            last_img: None,
            hud_fps: None,
            model,
            tint,
        };
        gfx.write_camera();
        gfx.write_object();
        gfx
    }
}
