use std::time::{Duration, Instant};
use std::sync::Arc;

use engine_core::Engine;
use glam::{Mat4, IVec3};
use render_wgpu::{wgpu, FrameGraph};
use render_wgpu::gfx::Gfx;
use render_wgpu::winit as rwinit;

use voxel_engine::{
    ChunkManager, greedy_mesh_chunk, TextureAtlas, STONE, AIR,
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

use crate::config::GameConfig;

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
    
    // Configuration
    config: GameConfig,

    // FPS
    frames: u32,
    last_fps_t: Instant,
    
    // FPS tracking for average
    total_fps_samples: f32,
    fps_sample_count: u32,
    start_time: Instant,

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
    texture_atlas: TextureAtlas,
    
    // Control mode
    control_mode: ControlMode,
    fullscreen: bool,
}

impl App {
    pub fn new() -> Self {
        // Load configuration from file
        let config = GameConfig::load().unwrap_or_else(|e| {
            eprintln!("⚠️  Failed to load config: {}", e);
            eprintln!("Using default configuration");
            GameConfig::default()
        });
        
        println!("📋 Configuration loaded:");
        println!("   • Render distance: {:.0} blocks", config.graphics.render_distance);
        println!("   • FOV: {:.0}°", config.graphics.fov_degrees);
        println!("   • Worker threads: {}", config.world.worker_threads);
        println!("   • Camera speed: {:.1} blocks/s", config.camera.move_speed);
        
        // Create job queue and start workers
        let job_queue = Arc::new(JobQueue::new());
        let worker = JobWorker::new(Arc::clone(&job_queue), config.world.worker_threads);
        let worker_handle = worker.start();
        
        // Configure chunk ring based on render distance from config
        let view_radius = config.calculate_view_radius();
        
        let ring_config = ChunkRingConfig {
            view_radius,                         // Based on render distance from config
            generation_radius: view_radius + 2,  // Generate 2 chunks ahead
            unload_radius: view_radius + 4,      // Unload 4 chunks beyond
        };
        
        println!("   • View radius: {} chunks ({} blocks)", view_radius, view_radius * config.world.chunk_size as i32);
        
        let start_time = Instant::now();
        
        Self {
            window: None,
            gfx: None,
            fg: FrameGraph::new()
                .clear(wgpu::Color { r: 0.07, g: 0.07, b: 0.09, a: 1.0 })
                .scene(),
            engine: Engine::new_fixed_hz(config.performance.target_fps),
            dt: Duration::from_secs_f64(1.0 / config.performance.target_fps as f64),
            next_tick: Instant::now(),
            input: Input::default(),
            config,
            frames: 0,
            last_fps_t: start_time,
            total_fps_samples: 0.0,
            fps_sample_count: 0,
            start_time,
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
            texture_atlas: TextureAtlas::new_16x16(),
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
    
    /// Display FPS statistics and exit
    fn display_fps_stats_and_exit(&self, el: &ActiveEventLoop) {
        println!("\n═══════════════════════════════════════");
        println!("          CLOSING APPLICATION          ");
        println!("═══════════════════════════════════════");
        
        let total_duration = self.start_time.elapsed();
        println!("Total runtime: {:.2}s", total_duration.as_secs_f32());
        
        if self.fps_sample_count > 0 {
            let avg_fps = self.total_fps_samples / self.fps_sample_count as f32;
            println!("Average FPS: {:.1} (based on {} samples)", avg_fps, self.fps_sample_count);
        } else {
            println!("No FPS data collected");
        }
        
        println!("═══════════════════════════════════════\n");
        el.exit();
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
                // Generate chunk SYNCHRONOUSLY (main thread)
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
                    // Use greedy mesher with atlas and chunk manager for seamless meshing
                    let mesh = greedy_mesh_chunk(chunk, Some(&self.chunk_manager), &self.texture_atlas);
                    
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
            let mut attrs = Window::default_attributes().with_title("Dev Engine - Voxel World (NEW)");
            
            // Apply window configuration
            attrs = attrs.with_inner_size(rwinit::dpi::LogicalSize::new(
                self.config.graphics.window_width,
                self.config.graphics.window_height,
            ));
            
            if self.config.graphics.fullscreen {
                attrs = attrs.with_fullscreen(Some(rwinit::window::Fullscreen::Borderless(None)));
            }
            
            let window = el.create_window(attrs).expect("create_window");
            let window_ref: &'static Window = Box::leak(Box::new(window));
            
            // Start in UI mode (cursor visible)
            // Player will click to enter Game mode
            
            let mut gfx = Gfx::new_with_config(
                window_ref, 
                Some(self.config.graphics.render_distance),
                Some(self.config.graphics.fov_degrees),
                Some(self.config.graphics.vsync),
            );
            let _ = gfx.enable_shader_hot_reload("crates/render_wgpu/assets/shader.wgsl");
            
            // Apply camera position from config
            gfx.cam_eye = glam::Vec3::from(self.config.camera.start_position);
            
            self.window = Some(window_ref);
            self.gfx = Some(gfx);
            
            println!("=== GAME STARTED ===");
            println!("Worker threads: {}", self.config.world.worker_threads);
            println!("View radius: {} chunks", self.config.calculate_view_radius());
            println!("Render distance: {:.0} blocks", self.config.graphics.render_distance);
            println!("FOV: {:.0}°", self.config.graphics.fov_degrees);
            println!("VSync: {}", if self.config.graphics.vsync { "On" } else { "Off" });
            
            // Initial chunk loading will happen in about_to_wait
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        // Handle raw mouse motion for camera rotation (only in Game mode)
        if self.control_mode == ControlMode::Game {
            if let DeviceEvent::MouseMotion { delta } = event {
                if let (Some(g), Some(w)) = (self.gfx.as_mut(), self.window) {
                    let sensitivity = self.config.camera.mouse_sensitivity;
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
            WindowEvent::CloseRequested => self.display_fps_stats_and_exit(el),
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
                                    ControlMode::UI => self.display_fps_stats_and_exit(el),  // ESC in UI mode = quit with stats
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
                    if let Some(g) = self.gfx.as_mut() { 
                        g.set_fps(fps); 
                        
                        // Update chunk generation performance stats
                        let job_stats = self.job_queue.get_stats();
                        g.set_chunk_perf_stats(
                            job_stats.avg_generation_time_ms,
                            job_stats.avg_meshing_time_ms
                        );
                    }
                    
                    // Track FPS for average calculation
                    self.total_fps_samples += fps;
                    self.fps_sample_count += 1;
                    
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
                // Hold Shift for speed boost
                let mut speed = self.config.camera.move_speed * self.dt.as_secs_f32();
                
                // Speed boost when holding Shift
                if self.input.held(KeyCode::ShiftLeft) || self.input.held(KeyCode::ShiftRight) {
                    speed *= self.config.camera.sprint_multiplier;
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
            
            // Update chunk loading based on camera position
            self.update_chunk_loading();
            
            // Process completed jobs from workers
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
