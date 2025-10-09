use std::time::{Duration, Instant};

use engine_core::Engine;
use glam::Mat4;
use render_wgpu::{wgpu, FrameGraph};
use render_wgpu::gfx::Gfx;
use render_wgpu::winit as rwinit;

use voxel_engine::{Chunk, mesh_chunk};
use render_wgpu::mesh_upload::MeshBuffers;

use rwinit::{
    application::ApplicationHandler,
    event::{ElementState, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod state;
use state::Input;

pub struct App {
    window: Option<&'static Window>,
    gfx: Option<Gfx<'static>>,
    pub fg: FrameGraph,
    engine: Engine,
    dt: Duration,
    next_tick: Instant,
    input: Input,

    // FPS
    frames: u32,
    last_fps_t: Instant,

    // rotation
    rot_on: bool,
    angle: f32,

    // voxel mesh
    voxel_mesh: Option<MeshBuffers>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            gfx: None,
            fg: FrameGraph::new()
                .clear(wgpu::Color { r: 0.07, g: 0.07, b: 0.09, a: 1.0 })
                .scene(),
            engine: Engine::new_fixed_hz(60),
            dt: Duration::from_secs_f64(1.0 / 60.0),
            next_tick: Instant::now(),
            input: Input::default(),
            frames: 0,
            last_fps_t: Instant::now(),
            rot_on: true,
            angle: 0.0,
            voxel_mesh: None,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _el: &ActiveEventLoop, _cause: StartCause) {}

    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes().with_title("Dev Engine");
            let window = el.create_window(attrs).expect("create_window");
            let window_ref: &'static Window = Box::leak(Box::new(window));
            let mut gfx = Gfx::new(window_ref);
            let _ = gfx.enable_shader_hot_reload("crates/render_wgpu/assets/shader.wgsl");
            self.window = Some(window_ref);
            self.gfx = Some(gfx);
            // Build a GPU stress test: combine multiple chunks for 65k+ voxels
            let mut combined_positions = Vec::new();
            let mut combined_uvs = Vec::new();
            let mut combined_indices = Vec::new();
            
            // Create 2x2 grid of chunks (4 chunks = 131,072 voxels max)
            for chunk_x in 0..2 {
                for chunk_z in 0..2 {
                    let mut c = Chunk::new_empty();
                    c.fill_gpu_stress_test();
                    let mesh = mesh_chunk(&c);
                    
                    let offset_x = chunk_x as f32 * 32.0;
                    let offset_z = chunk_z as f32 * 32.0;
                    let base_index = combined_positions.len() as u32;
                    
                    // Add vertices with offset
                    for pos in &mesh.positions {
                        combined_positions.push([pos[0] + offset_x, pos[1], pos[2] + offset_z]);
                    }
                    combined_uvs.extend_from_slice(&mesh.uvs);
                    
                    // Add indices with offset
                    for idx in &mesh.indices {
                        combined_indices.push(idx + base_index);
                    }
                }
            }
            
            println!("Combined voxel mesh: {} vertices, {} indices ({} triangles)", 
                     combined_positions.len(), 
                     combined_indices.len(),
                     combined_indices.len() / 3);
            
            if let Some(g) = self.gfx.as_ref() {
                let mb = g.upload_pos_uv(&combined_positions, &combined_uvs, &combined_indices);
                self.voxel_mesh = Some(mb);
            }

        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(g), Some(w)) = (self.gfx.as_mut(), self.window) {
            g.on_window_event(w, &event);
        }
        self.input.on_event(&event);

        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(sz) => {
                if let Some(g) = self.gfx.as_mut() { g.resize(sz); }
            }
            WindowEvent::DroppedFile(path) => {
                if let Some(g) = self.gfx.as_mut() {
                    if let Err(e) = g.load_texture_path(&path) {
                        eprintln!("image load error: {e}");
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        match code {
                            KeyCode::KeyV => { if let Some(g) = self.gfx.as_mut() { g.toggle_vsync(); } }
                            KeyCode::KeyR => { self.rot_on = !self.rot_on; }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let (Some(g), Some(w)) = (self.gfx.as_mut(), self.window) {
                    let dy = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(p) => (p.y as f32) / 120.0,
                    };
                    g.zoom(-0.5 * dy);
                    w.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let (Some(g), Some(w)) = (self.gfx.as_mut(), self.window) {
                    if self.input.rmb_down {
                        if let Some(prev) = self.input.last_cursor {
                            let sensitivity = 0.003;  // Augmenté de 0.0003 à 0.003
                            let dx = (position.x - prev.x) as f32 * sensitivity;
                            let dy = -(position.y - prev.y) as f32 * sensitivity;  // INVERSÉ pour contrôle naturel
                            g.rotate_camera(dx, dy);
                        }
                    }
                    self.input.last_cursor = Some(position);
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.frames += 1;
                let now = Instant::now();
                let dt = now.duration_since(self.last_fps_t);
                if dt.as_secs_f32() >= 1.0 {
                    let fps = self.frames as f32 / dt.as_secs_f32();
                    if let Some(g) = self.gfx.as_mut() { g.set_fps(fps); }
                    self.frames = 0;
                    self.last_fps_t = now;
                }

                if let (Some(g), Some(w)) = (self.gfx.as_mut(), self.window) {
                    if let Err(e) = g.render_with(w, &self.fg, self.voxel_mesh.as_ref()) {
                        match e {
                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                                let sz = g.size();
                                g.resize(sz);
                            }
                            wgpu::SurfaceError::OutOfMemory => el.exit(),
                            wgpu::SurfaceError::Timeout => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        let now = Instant::now();
        while now >= self.next_tick {
            if let Some(g) = self.gfx.as_mut() {
                // WASD/EQ camera movement in 3D
                let speed = 3.0 * self.dt.as_secs_f32();
                
                let forward = (self.input.held(KeyCode::KeyW) as i32 - self.input.held(KeyCode::KeyS) as i32) as f32;
                let right = (self.input.held(KeyCode::KeyD) as i32 - self.input.held(KeyCode::KeyA) as i32) as f32;
                let up = (self.input.held(KeyCode::KeyE) as i32 - self.input.held(KeyCode::KeyQ) as i32) as f32;
                
                if forward != 0.0 || right != 0.0 || up != 0.0 {
                    g.move_camera(forward * speed, right * speed, up * speed);
                }

                // rotate quad (disabled for voxels - set to identity)
                // if self.rot_on { self.angle += 1.0 * self.dt.as_secs_f32(); }
                let model = Mat4::IDENTITY;  // No rotation for voxels
                g.set_model(model);
            }
            self.engine.tick_once();
            self.next_tick += self.dt;
        }
        if let Some(w) = self.window {
            w.request_redraw(); }
    }
}
