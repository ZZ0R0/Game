use winit::{
    event::ElementState,
    window::Window,
    keyboard::KeyCode,
};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: std::sync::Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
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
        
        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
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
        
        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
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
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            render_pass.draw(0..3, 0..1);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
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
            mouse_sensitivity: 0.002,
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
        let speed = 5.0 * dt;
        
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
