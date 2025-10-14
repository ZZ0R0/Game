use wgpu::util::DeviceExt;
use glam::Vec3;

/// Simple overlay renderer for displaying text and info on screen
pub struct OverlayRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl OverlayVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OverlayVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl OverlayRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Overlay Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("overlay.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Overlay Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Overlay Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[OverlayVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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

        // Create initial empty buffer
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Overlay Vertex Buffer"),
            size: 1024 * std::mem::size_of::<OverlayVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            num_vertices: 0,
        }
    }

    /// Update overlay with game data
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, fps: f32, position: Option<Vec3>, screen_width: f32, screen_height: f32) {
        let mut vertices = Vec::new();

        // Background panel (top-left corner)
        let panel_width = 300.0;
        let panel_height = 100.0;
        let panel_x = 10.0;
        let panel_y = 10.0;

        self.add_rect(&mut vertices, panel_x, panel_y, panel_width, panel_height, 
                      [0.1, 0.1, 0.1, 0.8], screen_width, screen_height);

        // FPS text simulation with colored bars
        let text_x = panel_x + 10.0;
        let text_y = panel_y + 20.0;
        
        // "FPS:" label
        self.add_text(&mut vertices, text_x, text_y, "FPS:", [1.0, 1.0, 1.0, 1.0], screen_width, screen_height);
        
        // FPS value bar (green if >50, yellow if >30, red otherwise)
        let fps_color = if fps > 50.0 {
            [0.0, 1.0, 0.0, 1.0]
        } else if fps > 30.0 {
            [1.0, 1.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0, 1.0]
        };
        
        let fps_bar_width = (fps / 144.0 * 200.0).min(200.0);
        self.add_rect(&mut vertices, text_x + 60.0, text_y, fps_bar_width, 15.0, 
                      fps_color, screen_width, screen_height);
        
        // FPS number as simple digit blocks
        self.add_number(&mut vertices, text_x + 60.0, text_y + 20.0, fps as i32, 
                       [1.0, 1.0, 1.0, 1.0], screen_width, screen_height);

        // Position
        if let Some(pos) = position {
            let pos_y = text_y + 40.0;
            self.add_text(&mut vertices, text_x, pos_y, "POS:", [1.0, 1.0, 1.0, 1.0], screen_width, screen_height);
            
            // X coordinate (red bar)
            let x_bar_width = ((pos.x.abs() / 100.0).min(1.0) * 50.0).max(2.0);
            self.add_rect(&mut vertices, text_x + 60.0, pos_y, x_bar_width, 12.0, 
                          [1.0, 0.3, 0.3, 1.0], screen_width, screen_height);
            
            // Y coordinate (green bar)
            let y_bar_width = ((pos.y.abs() / 100.0).min(1.0) * 50.0).max(2.0);
            self.add_rect(&mut vertices, text_x + 120.0, pos_y, y_bar_width, 12.0, 
                          [0.3, 1.0, 0.3, 1.0], screen_width, screen_height);
            
            // Z coordinate (blue bar)
            let z_bar_width = ((pos.z.abs() / 100.0).min(1.0) * 50.0).max(2.0);
            self.add_rect(&mut vertices, text_x + 180.0, pos_y, z_bar_width, 12.0, 
                          [0.3, 0.3, 1.0, 1.0], screen_width, screen_height);
            
            // Numbers below
            self.add_number(&mut vertices, text_x + 60.0, pos_y + 15.0, pos.x as i32, 
                           [1.0, 0.3, 0.3, 1.0], screen_width, screen_height);
            self.add_number(&mut vertices, text_x + 120.0, pos_y + 15.0, pos.y as i32, 
                           [0.3, 1.0, 0.3, 1.0], screen_width, screen_height);
            self.add_number(&mut vertices, text_x + 180.0, pos_y + 15.0, pos.z as i32, 
                           [0.3, 0.3, 1.0, 1.0], screen_width, screen_height);
        }

        self.num_vertices = vertices.len() as u32;

        if !vertices.is_empty() {
            // Recreate buffer if needed
            let buffer_size = (vertices.len() * std::mem::size_of::<OverlayVertex>()) as u64;
            if buffer_size > self.vertex_buffer.size() {
                self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Overlay Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
            } else {
                queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            }
        }
    }

    /// Add a rectangle to the vertex list
    fn add_rect(&self, vertices: &mut Vec<OverlayVertex>, x: f32, y: f32, width: f32, height: f32, 
                color: [f32; 4], screen_width: f32, screen_height: f32) {
        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let x1 = (x / screen_width) * 2.0 - 1.0;
        let y1 = -((y / screen_height) * 2.0 - 1.0);
        let x2 = ((x + width) / screen_width) * 2.0 - 1.0;
        let y2 = -(((y + height) / screen_height) * 2.0 - 1.0);

        // Two triangles to make a quad
        vertices.extend_from_slice(&[
            OverlayVertex { position: [x1, y1], color },
            OverlayVertex { position: [x2, y1], color },
            OverlayVertex { position: [x1, y2], color },
            OverlayVertex { position: [x2, y1], color },
            OverlayVertex { position: [x2, y2], color },
            OverlayVertex { position: [x1, y2], color },
        ]);
    }

    /// Add simple text using small rectangles for each character
    fn add_text(&self, vertices: &mut Vec<OverlayVertex>, x: f32, y: f32, text: &str, 
                color: [f32; 4], screen_width: f32, screen_height: f32) {
        let mut cursor_x = x;
        for ch in text.chars() {
            self.add_char(vertices, cursor_x, y, ch, color, screen_width, screen_height);
            cursor_x += 8.0; // Character width + spacing
        }
    }

    /// Render a simple character using small rectangles (bitmap style)
    fn add_char(&self, vertices: &mut Vec<OverlayVertex>, x: f32, y: f32, ch: char, 
                color: [f32; 4], screen_width: f32, screen_height: f32) {
        // Very simple 5x7 bitmap font patterns
        let pattern = match ch {
            'F' => vec![[1,1,1,1], [1,0,0,0], [1,1,1,0], [1,0,0,0], [1,0,0,0]],
            'P' => vec![[1,1,1,1], [1,0,0,1], [1,1,1,1], [1,0,0,0], [1,0,0,0]],
            'S' => vec![[0,1,1,1], [1,0,0,0], [0,1,1,0], [0,0,0,1], [1,1,1,0]],
            'O' => vec![[0,1,1,0], [1,0,0,1], [1,0,0,1], [1,0,0,1], [0,1,1,0]],
            ':' => vec![[0,0,0,0], [0,1,0,0], [0,0,0,0], [0,1,0,0], [0,0,0,0]],
            _ => vec![[0,0,0,0], [0,0,0,0], [0,0,0,0], [0,0,0,0], [0,0,0,0]],
        };

        for (row, line) in pattern.iter().enumerate() {
            for (col, &pixel) in line.iter().enumerate() {
                if pixel == 1 {
                    self.add_rect(vertices, x + col as f32 * 1.5, y + row as f32 * 2.0, 
                                 1.2, 1.8, color, screen_width, screen_height);
                }
            }
        }
    }

    /// Render a number using simple digit blocks
    fn add_number(&self, vertices: &mut Vec<OverlayVertex>, x: f32, y: f32, num: i32, 
                  color: [f32; 4], screen_width: f32, screen_height: f32) {
        let num_str = format!("{}", num);
        let mut cursor_x = x;
        
        for ch in num_str.chars() {
            let pattern = match ch {
                '0' => vec![[1,1,1], [1,0,1], [1,0,1], [1,0,1], [1,1,1]],
                '1' => vec![[0,1,0], [1,1,0], [0,1,0], [0,1,0], [1,1,1]],
                '2' => vec![[1,1,1], [0,0,1], [1,1,1], [1,0,0], [1,1,1]],
                '3' => vec![[1,1,1], [0,0,1], [1,1,1], [0,0,1], [1,1,1]],
                '4' => vec![[1,0,1], [1,0,1], [1,1,1], [0,0,1], [0,0,1]],
                '5' => vec![[1,1,1], [1,0,0], [1,1,1], [0,0,1], [1,1,1]],
                '6' => vec![[1,1,1], [1,0,0], [1,1,1], [1,0,1], [1,1,1]],
                '7' => vec![[1,1,1], [0,0,1], [0,0,1], [0,0,1], [0,0,1]],
                '8' => vec![[1,1,1], [1,0,1], [1,1,1], [1,0,1], [1,1,1]],
                '9' => vec![[1,1,1], [1,0,1], [1,1,1], [0,0,1], [1,1,1]],
                '-' => vec![[0,0,0], [0,0,0], [1,1,1], [0,0,0], [0,0,0]],
                _ => vec![[0,0,0], [0,0,0], [0,0,0], [0,0,0], [0,0,0]],
            };

            for (row, line) in pattern.iter().enumerate() {
                for (col, &pixel) in line.iter().enumerate() {
                    if pixel == 1 {
                        self.add_rect(vertices, cursor_x + col as f32 * 2.0, y + row as f32 * 2.0, 
                                     1.5, 1.5, color, screen_width, screen_height);
                    }
                }
            }
            cursor_x += 10.0;
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.num_vertices > 0 {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }
    }
}
