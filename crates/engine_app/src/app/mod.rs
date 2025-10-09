use std::time::{Duration, Instant};
use std::sync::Arc;

use engine_core::Engine;
use glam::{Mat4, IVec3};
use render_wgpu::{wgpu, FrameGraph};
use render_wgpu::gfx::Gfx;
use render_wgpu::winit as rwinit;

use voxel_engine::{
    ChunkManager, mesh_chunk, STONE, AIR,
    ChunkRing, ChunkRingConfig, JobQueue, JobWorker, WorkerHandle, ChunkJob, JobResult,
    TerrainGenerator, ChunkPool, MeshPool,
};
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

    // Voxel world - NEW SYSTEM
    chunk_manager: ChunkManager,
    chunk_ring: ChunkRing,
    job_queue: Arc<JobQueue>,
    terrain_generator: TerrainGenerator,
    chunk_pool: ChunkPool,
    mesh_pool: MeshPool,
    worker_handle: Option<WorkerHandle>,
    voxel_mesh: Option<MeshBuffers>,
    
    // Control mode
    control_mode: ControlMode,
    fullscreen: bool,
}

impl App {
    pub fn new() -> Self {
        // Create job queue and start workers
        let job_queue = Arc::new(JobQueue::new());
        let worker = JobWorker::new(Arc::clone(&job_queue), 4); // 4 worker threads
        let worker_handle = worker.start();
        
        // Configure chunk ring
        let ring_config = ChunkRingConfig {
            view_radius: 8,
            generation_radius: 10,
            unload_radius: 12,
        };
        
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
            chunk_ring: ChunkRing::new(ring_config),
            job_queue,
            terrain_generator: TerrainGenerator::default(),
            chunk_pool: ChunkPool::new(512),
            mesh_pool: MeshPool::new(512),
            worker_handle: Some(worker_handle),
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
    /// Update chunk loading based on camera position (NEW SYSTEM)
    fn update_chunk_loading(&mut self) {
        if let Some(g) = self.gfx.as_ref() {
            let cam_pos = g.cam_eye;
            
            // Update chunk ring to find which chunks to load/unload
            let (to_load, to_unload) = self.chunk_ring.update(cam_pos);
            
            // Unload chunks that are too far
            for chunk_pos in to_unload {
                if let Some(chunk) = self.chunk_manager.remove(chunk_pos) {
                    // Return chunk to pool for reuse
                    self.chunk_pool.release(chunk);
                    self.chunk_ring.mark_unloaded(chunk_pos);
                    
                    // Also remove from renderer
                    if let Some(gfx) = self.gfx.as_mut() {
                        gfx.chunk_renderer.remove_chunk(chunk_pos);
                    }
                }
            }
            
            // Queue generation jobs for new chunks
            for chunk_pos in to_load {
                // Create generation job
                let chunk = self.terrain_generator.generate_chunk(chunk_pos);
                
                // Insert into chunk manager
                self.chunk_manager.insert(chunk.clone());
                self.chunk_ring.mark_loaded(chunk_pos);
                
                // Queue meshing job
                self.job_queue.push(ChunkJob::Mesh {
                    position: chunk_pos,
                    chunk: Arc::new(chunk),
                });
            }
        }
    }
    
    /// Process completed jobs from worker threads
    fn process_completed_jobs(&mut self) {
        let results = self.job_queue.drain_completed();
        if !results.is_empty() {
        
        for result in &results {
            match result {
                JobResult::Generated { position, chunk } => {
                    // Chunk generated, insert into manager
                    self.chunk_manager.insert(chunk.clone());
                    self.chunk_ring.mark_loaded(*position);
                    
                    // Queue meshing
                    self.job_queue.push(ChunkJob::Mesh {
                        position: *position,
                        chunk: Arc::new((*chunk).clone()),
                    });
                }
                
                JobResult::Meshed { position, mesh } => {
                    // Mesh generated, upload to GPU
                    self.upload_chunk_mesh(*position, mesh.clone());
                }
                
                JobResult::Uploaded { position: _ } => {
                    // Upload complete (handled by upload_chunk_mesh)
                }
                
                JobResult::PhysicsReady { position: _ } => {
                    // Physics complete (future)
                }
            }
        }
        }
    }
    
    /// Upload a chunk mesh to the GPU
    fn upload_chunk_mesh(&mut self, position: IVec3, mesh: voxel_engine::MeshData) {
        if let Some(g) = self.gfx.as_mut() {
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
                    let offset_x = position.x as f32 * 32.0;
                    let offset_y = position.y as f32 * 32.0;
                    let offset_z = position.z as f32 * 32.0;
                    
                    VertexTex {
                        pos: [p[0] + offset_x, p[1] + offset_y, p[2] + offset_z],
                        uv: *u,
                    }
                })
                .collect();
            
            let vertex_bytes: &[u8] = cast_slice(&vertices);
            
            // Upload to ChunkRenderer
            g.upload_chunk(position, vertex_bytes, &mesh.indices);
        }
    }
    
    /// Handle voxel picking with mouse clicks
    fn handle_voxel_picking(&mut self, button: rwinit::event::MouseButton) {
        if let Some(g) = self.gfx.as_ref() {
            // Get camera position and forward direction
            let cam_pos = g.cam_eye;
            let cam_target = g.cam_target;
            let cam_forward = (cam_target - cam_pos).normalize();
            
            // Perform raycast
            let max_distance = 100.0;
            if let Some(hit) = self.chunk_manager.raycast(cam_pos, cam_forward, max_distance) {
                match button {
                    rwinit::event::MouseButton::Left => {
                        // Left click: REMOVE block (mine)
                        println!("⛏️ Mining block at {:?} (was {:?})", hit.position, hit.block_id);
                        self.chunk_manager.set_block(hit.position, AIR);
                        
                        // Update dirty chunks
                        self.update_dirty_chunks();
                    }
                    rwinit::event::MouseButton::Right => {
                        // Right click: PLACE block
                        // Place at the adjacent position (in front of hit face)
                        println!("🧱 Placing block at {:?} (adjacent to {:?})", hit.adjacent_position, hit.position);
                        self.chunk_manager.set_block(hit.adjacent_position, STONE);
                        
                        // Update dirty chunks
                        self.update_dirty_chunks();
                    }
                    _ => {}
                }
            } else {
                println!("❌ No block in range (max: {:.1}m)", max_distance);
            }
        }
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
            let attrs = Window::default_attributes().with_title("Dev Engine - Voxel World (NEW)");
            let window = el.create_window(attrs).expect("create_window");
            let window_ref: &'static Window = Box::leak(Box::new(window));
            
            // Start in UI mode (cursor visible)
            // Player will click to enter Game mode
            
            let mut gfx = Gfx::new(window_ref);
            let _ = gfx.enable_shader_hot_reload("crates/render_wgpu/assets/shader.wgsl");
            self.window = Some(window_ref);
            self.gfx = Some(gfx);
            
            println!("=== NEW CHUNK SYSTEM INITIALIZED ===");
            println!("✓ Worker threads: 4");
            println!("✓ View radius: 8 chunks");
            println!("✓ Generation radius: 10 chunks");
            println!("✓ Chunk pool capacity: 512");
            
            // Initial chunk loading will happen in about_to_wait
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
                if self.control_mode == ControlMode::UI {
                    if state == ElementState::Pressed && button == rwinit::event::MouseButton::Left {
                        self.set_control_mode(ControlMode::Game);
                    }
                } else if self.control_mode == ControlMode::Game {
                    // Voxel picking in Game mode
                    if state == ElementState::Pressed {
                        self.handle_voxel_picking(button);
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
                // Hold Shift for 10x speed boost
                let mut speed = 3.0 * self.dt.as_secs_f32();
                
                // Speed boost when holding Shift
                if self.input.held(KeyCode::ShiftLeft) || self.input.held(KeyCode::ShiftRight) {
                    speed *= 10.0;
                }
                
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
            
            // NEW: Update chunk loading based on camera position
            self.update_chunk_loading();
            
            // NEW: Process completed jobs from workers
            self.process_completed_jobs();
            
            // Update dirty chunks (from block editing)
            self.update_dirty_chunks();
            
            self.engine.tick_once();
            self.next_tick += self.dt;
        }
        if let Some(w) = self.window {
            w.request_redraw(); }
    }
}
