use winit::{
    event::ElementState,
    window::Window,
    keyboard::KeyCode,
};
use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Mat4};
use wgpu::util::DeviceExt;

mod overlay;
mod scene_cache;
use overlay::OverlayRenderer;
use scene_cache::SceneCache;

pub struct OverlayData {
    pub fps: f32,
    pub player_position: Option<Vec3>,
}

// Vertex structure for 3D cubes
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// Camera uniform buffer data
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 3],
    _padding: f32,
}

// Model uniform buffer data
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ModelUniform {
    matrix: [[f32; 4]; 4],
}

// Create cube vertices (Space Engineers block)
const CUBE_VERTICES: &[Vertex] = &[
    // Front face
    Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0], tex_coords: [0.0, 0.0] },
    // Back face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0], tex_coords: [0.0, 1.0] },
    // Top face
    Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0], tex_coords: [1.0, 1.0] },
    // Bottom face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0], tex_coords: [1.0, 0.0] },
    // Right face
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 1.0,  0.0,  0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 1.0,  0.0,  0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 1.0,  0.0,  0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 1.0,  0.0,  0.0], tex_coords: [0.0, 1.0] },
    // Left face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0,  0.0,  0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [-1.0,  0.0,  0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [-1.0,  0.0,  0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0,  0.0,  0.0], tex_coords: [0.0, 0.0] },
];

const CUBE_INDICES: &[u16] = &[
     0,  1,  2,   2,  3,  0, // Front
     4,  5,  6,   6,  7,  4, // Back
     8,  9, 10,  10, 11,  8, // Top
    12, 13, 14,  14, 15, 12, // Bottom
    16, 17, 18,  18, 19, 16, // Right
    20, 21, 22,  22, 23, 20, // Left
];

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: std::sync::Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    model_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    texture_bind_group_layout: wgpu::BindGroupLayout,
    default_texture_bind_group: wgpu::BindGroup,
    blocks_to_render: Vec<BlockInstance>,
    overlay_data: OverlayData,
    overlay_renderer: OverlayRenderer,
    scene_cache: SceneCache,
}

#[derive(Clone)]
pub struct BlockInstance {
    pub id: u32,
    pub version: u64,
    pub position: Vec3,
    pub texture_path: String,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        
        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // Create surface
        let surface = instance.create_surface(window.clone())?;
        
        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| format!("Failed to find adapter: {:?}", e))?;
        
        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    trace: Default::default(),
                },
            )
            .await
            .map_err(|e| format!("Failed to create device: {:?}", e))?;
        
        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        
        // Create basic shader for cube rendering
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        
        // Create camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        // Create model bind group layout
        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("model_bind_group_layout"),
        });

        // Create texture bind group layout (for future texture support)
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        
        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &model_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        
        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = CUBE_INDICES.len() as u32;
        
        // Create camera uniform
        let camera_uniform = CameraUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view_pos: [0.0, 0.0, 5.0],
            _padding: 0.0,
        };
        
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        


        // Create default white texture (1x1 white pixel)
        let texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        // Write white pixel to texture
        queue.write_texture(
            default_texture.as_image_copy(),
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            texture_size,
        );

        let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_sampler),
                },
            ],
            label: Some("default_texture_bind_group"),
        });

        let overlay_data = OverlayData {
            fps: 0.0,
            player_position: None,
        };

        let overlay_renderer = OverlayRenderer::new(&device, config.format);
        let scene_cache = SceneCache::new();
        
        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            model_bind_group_layout,
            texture_bind_group_layout,
            default_texture_bind_group,
            blocks_to_render: Vec::new(),
            overlay_data,
            overlay_renderer,
            scene_cache,
        })
    }
    
    pub fn window(&self) -> &Window {
        &self.window
    }
    
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    fn update_bind_groups_cache(&mut self) {
        // Nettoie le cache des objets obsolètes
        let active_objects: Vec<(u32, u64)> = self.blocks_to_render.iter()
            .map(|block| (block.id, block.version))
            .collect();
        self.scene_cache.cleanup_old_entries(&active_objects);

        // Vérifie et met à jour les bind groups nécessaires
        for block in &self.blocks_to_render {
            if self.scene_cache.is_dirty(block.id, block.version) {
                let model_matrix = Mat4::from_translation(block.position);
                let model_uniform = ModelUniform {
                    matrix: model_matrix.to_cols_array_2d(),
                };
                
                let block_model_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Model Buffer {}", block.id)),
                    contents: bytemuck::cast_slice(&[model_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                let new_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.model_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: block_model_buffer.as_entire_binding(),
                    }],
                    label: Some(&format!("model_bind_group_{}", block.id)),
                });

                self.scene_cache.cache_bind_group(block.id, block.version, new_bind_group);
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Met à jour le cache des bind groups avant le rendu
        self.update_bind_groups_cache();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Render each block instance avec cache
            for block in &self.blocks_to_render {
                // Utilise le bind group depuis le cache (garanti d'exister après update_bind_groups_cache)
                if let Some(model_bind_group) = self.scene_cache.get_bind_group(block.id, block.version) {
                    render_pass.set_bind_group(1, model_bind_group, &[]);
                    render_pass.set_bind_group(2, &self.default_texture_bind_group, &[]);
                    render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
                }
            }
        }

        // Update and render overlay
        self.overlay_renderer.update(
            &self.device, 
            &self.queue, 
            self.overlay_data.fps, 
            self.overlay_data.player_position,
            self.size.width as f32,
            self.size.height as f32
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Overlay Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.overlay_renderer.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }

    pub fn set_blocks_to_render(&mut self, blocks: Vec<BlockInstance>) {
        self.blocks_to_render = blocks;
    }

    pub fn update_overlay_data(&mut self, fps: f32, player_position: Option<Vec3>) {
        self.overlay_data.fps = fps;
        self.overlay_data.player_position = player_position;
    }
    
    pub fn update_camera(&mut self, camera: &Camera) {
        // Update camera uniform with FPS camera data
        self.camera_uniform.view_proj = camera.view_projection_matrix().to_cols_array_2d();
        self.camera_uniform.view_pos = camera.position().to_array();
        
        // Upload to GPU
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn get_cache_stats(&self) -> scene_cache::CacheStats {
        self.scene_cache.stats()
    }
}

pub struct Camera {
    pub position: glam::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: glam::Vec3::new(0.0, 0.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 45.0_f32.to_radians(),
            aspect: width / height,
            near: 0.1,
            far: 100.0,
        }
    }
    
    pub fn update_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }
    
    pub fn view_matrix(&self) -> glam::Mat4 {
        let forward = glam::Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();
        
        let up = glam::Vec3::Y;
        
        glam::Mat4::look_at_rh(
            self.position,
            self.position + forward,
            up,
        )
    }
    
    pub fn projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
    
    pub fn view_projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
    
    pub fn position(&self) -> glam::Vec3 {
        self.position
    }
}

pub struct InputHandler {
    pub keys_pressed: std::collections::HashSet<KeyCode>,
    pub mouse_delta: (f32, f32),
    pub mouse_sensitivity: f32,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: std::collections::HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_sensitivity: 0.02,
        }
    }
    
    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.keys_pressed.insert(key);
            }
            ElementState::Released => {
                self.keys_pressed.remove(&key);
            }
        }
    }
    
    pub fn process_mouse(&mut self, delta: (f64, f64)) {
        self.mouse_delta = (delta.0 as f32, delta.1 as f32);
    }
    
    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        let base_speed = 5.0;
        let sprint_multiplier = if self.keys_pressed.contains(&KeyCode::ShiftLeft) || 
                                   self.keys_pressed.contains(&KeyCode::ShiftRight) { 2.5 } else { 1.0 };
        let speed = base_speed * sprint_multiplier * dt;
        
        // Mouse look
        camera.yaw += self.mouse_delta.0 * self.mouse_sensitivity;
        camera.pitch -= self.mouse_delta.1 * self.mouse_sensitivity;
        camera.pitch = camera.pitch.clamp(-1.5, 1.5);
        self.mouse_delta = (0.0, 0.0);
        
        // Movement
        let forward = glam::Vec3::new(
            camera.yaw.cos() * camera.pitch.cos(),
            camera.pitch.sin(),
            camera.yaw.sin() * camera.pitch.cos(),
        ).normalize();
        
        let right = forward.cross(glam::Vec3::Y).normalize();
        let up = glam::Vec3::Y;
        
        if self.keys_pressed.contains(&KeyCode::KeyW) {
            camera.position += forward * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyS) {
            camera.position -= forward * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyA) {
            camera.position -= right * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyD) {
            camera.position += right * speed;
        }
        if self.keys_pressed.contains(&KeyCode::Space) {
            camera.position += up * speed;
        }
        if self.keys_pressed.contains(&KeyCode::ControlLeft) {
            camera.position -= up * speed;
        }
    }
}
