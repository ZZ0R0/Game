use std::time::{Duration, Instant};

use engine_core::Engine;
use glam::{Mat4, IVec3};
use render_wgpu::{wgpu, FrameGraph};
use render_wgpu::gfx::Gfx;
use render_wgpu::winit as rwinit;

use voxel_engine::{ChunkManager, Chunk, mesh_chunk, STONE, GRASS, DIRT, WOOD};
use render_wgpu::mesh_upload::MeshBuffers;

use rwinit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod state;
use state::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Game,    // FPS mode: cursor locked, camera rotation
    UI,      // UI mode: cursor visible, can interact with menus
}

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

    // Voxel world
    chunk_manager: ChunkManager,
    voxel_mesh: Option<MeshBuffers>,
    
    // Control mode
    control_mode: ControlMode,
    fullscreen: bool,
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
            chunk_manager: ChunkManager::new(),
            voxel_mesh: None,
            control_mode: ControlMode::UI,  // Start in UI mode
            fullscreen: false,
        }
    }
    
    fn set_control_mode(&mut self, mode: ControlMode) {
        if self.control_mode == mode {
            return;
        }
        self.control_mode = mode;
        
        if let Some(window) = self.window {
            match mode {
                ControlMode::Game => {
                    // Lock cursor and hide it for FPS mode
                    if let Err(e) = window.set_cursor_grab(rwinit::window::CursorGrabMode::Locked) {
                        eprintln!("Failed to lock cursor: {}", e);
                    }
                    window.set_cursor_visible(false);
                    println!("Switched to GAME mode (cursor locked)");
                }
                ControlMode::UI => {
                    // Release cursor and show it for UI mode
                    let _ = window.set_cursor_grab(rwinit::window::CursorGrabMode::None);
                    window.set_cursor_visible(true);
                    println!("Switched to UI mode (cursor visible)");
                }
            }
        }
    }
    
    fn toggle_fullscreen(&mut self) {
        if let Some(window) = self.window {
            self.fullscreen = !self.fullscreen;
            if self.fullscreen {
                window.set_fullscreen(Some(rwinit::window::Fullscreen::Borderless(None)));
                println!("Fullscreen: ON");
            } else {
                window.set_fullscreen(None);
                println!("Fullscreen: OFF");
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Generate a test world with terrain
    fn generate_test_world(&mut self) {
        println!("Generating test world...");
        
        // Create a 8x8 grid of chunks (64 chunks) for better culling demo
        let radius = 4;
        for x in -radius..radius {
            for z in -radius..radius {
                let pos = IVec3::new(x, 0, z);
                let mut chunk = Chunk::new(pos);
                
                // Generate terrain
                let size = 32;
                for cz in 0..size {
                    for cx in 0..size {
                        // World coordinates
                        let wx = pos.x * size as i32 + cx as i32;
                        let wz = pos.z * size as i32 + cz as i32;
                        
                        // More varied height map
                        let height = (10.0 
                            + 5.0 * (wx as f32 * 0.05).sin() 
                            + 4.0 * (wz as f32 * 0.08).cos()
                            + 3.0 * (wx as f32 * 0.12).sin() * (wz as f32 * 0.09).cos()
                        ) as usize;
                        
                        for cy in 0..size {
                            if cy < height.saturating_sub(4) {
                                chunk.set(cx, cy, cz, STONE);
                            } else if cy < height {
                                chunk.set(cx, cy, cz, DIRT);
                            } else if cy == height {
                                chunk.set(cx, cy, cz, GRASS);
                            }
                        }
                    }
                }
                
                // Add some trees randomly
                if (x + z) % 3 == 0 {
                    // Tree at center
                    for y in 12..17 {
                        chunk.set(16, y, 16, WOOD);
                    }
                    // Leaves
                    for dy in 0..3 {
                        for dx in -2..=2 {
                            for dz in -2..=2 {
                                let lx = (16i32 + dx) as usize;
                                let lz = (16i32 + dz) as usize;
                                let ly = 17 + dy;
                                if lx < 32 && lz < 32 && ly < 32 {
                                    chunk.set(lx, ly, lz, GRASS); // Using GRASS as leaves
                                }
                            }
                        }
                    }
                }
                
                self.chunk_manager.insert(chunk);
            }
        }
        
        println!("✓ Generated {} chunks", self.chunk_manager.chunk_count());
        println!("✓ Memory: {:.2} MiB", 
                 self.chunk_manager.total_memory_usage() as f64 / 1024.0 / 1024.0);
    }
    
    /// Generate mesh for all dirty chunks
    fn update_dirty_chunks(&mut self) {
        let dirty_chunks = self.chunk_manager.get_dirty_chunks();
        if dirty_chunks.is_empty() {
            return;
        }
        
        println!("Updating {} dirty chunks...", dirty_chunks.len());
        
        if let Some(g) = self.gfx.as_mut() {
            for chunk_pos in &dirty_chunks {
                if let Some(chunk) = self.chunk_manager.get_chunk(*chunk_pos) {
                    let mesh = mesh_chunk(chunk);
                    
                    // Convert to vertex bytes (VertexTex format)
                    use bytemuck::cast_slice;
                    
                    #[repr(C)]
                    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
                    struct VertexTex {
                        pos: [f32; 3],
                        uv: [f32; 2],
                    }
                    
                    let vertices: Vec<VertexTex> = mesh.positions.iter().zip(mesh.uvs.iter())
                        .map(|(p, u)| {
                            // Offset to world coordinates
                            let offset_x = chunk_pos.x as f32 * 32.0;
                            let offset_y = chunk_pos.y as f32 * 32.0;
                            let offset_z = chunk_pos.z as f32 * 32.0;
                            
                            VertexTex {
                                pos: [p[0] + offset_x, p[1] + offset_y, p[2] + offset_z],
                                uv: *u,
                            }
                        })
                        .collect();
                    
                    let vertex_bytes: &[u8] = cast_slice(&vertices);
                    
                    // Upload chunk to ChunkRenderer
                    g.upload_chunk(*chunk_pos, vertex_bytes, &mesh.indices);
                }
                
                self.chunk_manager.clear_dirty(*chunk_pos);
            }
            
            let stats = &g.chunk_renderer.stats;
            println!("✓ ChunkRenderer: {} chunks, {} triangles total",
                     stats.total_chunks,
                     stats.total_triangles);
        }
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _el: &ActiveEventLoop, _cause: StartCause) {}

    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes().with_title("Dev Engine - Click to play");
            let window = el.create_window(attrs).expect("create_window");
            let window_ref: &'static Window = Box::leak(Box::new(window));
            
            // Start in UI mode (cursor visible)
            // Player will click to enter Game mode
            
            let mut gfx = Gfx::new(window_ref);
            let _ = gfx.enable_shader_hot_reload("crates/render_wgpu/assets/shader.wgsl");
            self.window = Some(window_ref);
            self.gfx = Some(gfx);
            
            // Generate test world
            self.generate_test_world();
            
            // Generate meshes for all chunks
            self.update_dirty_chunks();
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        // Handle raw mouse motion for camera rotation (only in Game mode)
        if self.control_mode == ControlMode::Game {
            if let DeviceEvent::MouseMotion { delta } = event {
                if let (Some(g), Some(w)) = (self.gfx.as_mut(), self.window) {
                    let sensitivity = 0.003;
                    let dx = delta.0 as f32 * sensitivity;
                    let dy = -(delta.1 as f32) * sensitivity;
                    g.rotate_camera(dx, dy);
                    w.request_redraw();
                }
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
                            KeyCode::Escape => {
                                // ESC toggles between Game and UI mode
                                match self.control_mode {
                                    ControlMode::Game => self.set_control_mode(ControlMode::UI),
                                    ControlMode::UI => el.exit(),  // ESC in UI mode = quit
                                }
                            }
                            KeyCode::F11 => {
                                // F11 toggles fullscreen
                                self.toggle_fullscreen();
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Click to enter Game mode from UI mode
                if state == ElementState::Pressed 
                   && button == rwinit::event::MouseButton::Left
                   && self.control_mode == ControlMode::UI {
                    self.set_control_mode(ControlMode::Game);
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
                // WASD camera movement in 3D
                // Space to go up, C to go down
                let speed = 3.0 * self.dt.as_secs_f32();
                
                let forward = (self.input.held(KeyCode::KeyW) as i32 - self.input.held(KeyCode::KeyS) as i32) as f32;
                let right = (self.input.held(KeyCode::KeyD) as i32 - self.input.held(KeyCode::KeyA) as i32) as f32;
                let up = (self.input.held(KeyCode::Space) as i32 - self.input.held(KeyCode::KeyC) as i32) as f32;
                
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
