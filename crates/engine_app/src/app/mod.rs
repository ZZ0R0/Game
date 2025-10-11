use std::time::{Duration, Instant};
use std::sync::Arc;
use std::collections::VecDeque;

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

/// Frame profiler for tracking performance of major systems
#[derive(Debug, Clone)]
struct FrameProfiler {
    chunk_loading_ms: f32,
    job_processing_ms: f32,
    rendering_ms: f32,
    total_frame_ms: f32,
    
    // Counters
    chunks_loaded: usize,
    chunks_meshed: usize,
    chunks_rendered: usize,
    draw_calls: usize,
    jobs_pending: usize,
    jobs_completed: usize,
    
    // History for averaging
    frame_times: VecDeque<f32>,
    last_print: Instant,
}

impl FrameProfiler {
    fn new() -> Self {
        Self {
            chunk_loading_ms: 0.0,
            job_processing_ms: 0.0,
            rendering_ms: 0.0,
            total_frame_ms: 0.0,
            
            chunks_loaded: 0,
            chunks_meshed: 0,
            chunks_rendered: 0,
            draw_calls: 0,
            jobs_pending: 0,
            jobs_completed: 0,
            
            frame_times: VecDeque::with_capacity(120),
            last_print: Instant::now(),
        }
    }
    
    fn begin_frame(&mut self) {
        // Reset per-frame counters
        self.chunk_loading_ms = 0.0;
        self.job_processing_ms = 0.0;
        self.rendering_ms = 0.0;
        self.total_frame_ms = 0.0;
        self.chunks_meshed = 0;
        self.draw_calls = 0;
        self.jobs_completed = 0;
    }
    
    fn end_frame(&mut self, total_ms: f32) {
        self.total_frame_ms = total_ms;
        
        // Add to history
        self.frame_times.push_back(total_ms);
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }
        
        // Print if slow frame (> 20ms = < 50 FPS)
        if total_ms > 20.0 {
            println!("🐌 SLOW FRAME: {:.2}ms", total_ms);
            println!("   Chunk Loading: {:.2}ms", self.chunk_loading_ms);
            println!("   Job Processing: {:.2}ms ({} jobs)", self.job_processing_ms, self.jobs_completed);
            println!("   Rendering: {:.2}ms ({} draw calls)", self.rendering_ms, self.draw_calls);
            println!("   Chunks: {} loaded, {} rendered", self.chunks_loaded, self.chunks_rendered);
        }
        
        // Print summary every second
        if self.last_print.elapsed().as_secs_f32() >= 1.0 {
            self.print_summary();
            self.last_print = Instant::now();
        }
    }
    
    fn print_summary(&self) {
        if self.frame_times.is_empty() {
            return;
        }
        
        let avg_frame = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        let fps = 1000.0 / avg_frame;
        let max_frame = self.frame_times.iter().copied().fold(0.0f32, f32::max);
        let min_frame = self.frame_times.iter().copied().fold(f32::MAX, f32::min);
        
        println!("\n📊 Performance Summary:");
        println!("   FPS: {:.1} (avg: {:.2}ms, min: {:.2}ms, max: {:.2}ms)", 
                 fps, avg_frame, min_frame, max_frame);
        println!("   Breakdown: Load={:.1}ms, Jobs={:.1}ms, Render={:.1}ms",
                 self.chunk_loading_ms, self.job_processing_ms, self.rendering_ms);
        println!("   Chunks: {} loaded, {} rendered, {} draw calls",
                 self.chunks_loaded, self.chunks_rendered, self.draw_calls);
        println!("   Jobs: {} pending, {} completed/frame", 
                 self.jobs_pending, self.jobs_completed);
    }
}

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
    
    // Performance profiling
    profiler: FrameProfiler,
    
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
            profiler: FrameProfiler::new(),
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
            let mut unloaded_count = 0;
            for chunk_pos in to_unload {
                if let Some(chunk) = self.chunk_manager.remove(chunk_pos) {
                    // Return chunk to pool for reuse
                    self.chunk_pool.release(chunk);
                    self.chunk_ring.mark_unloaded(chunk_pos);
                    
                    // Also remove from renderer
                    if let Some(gfx) = self.gfx.as_mut() {
                        gfx.chunk_renderer.remove_chunk(chunk_pos);
                    }
                    
                    unloaded_count += 1;
                }
            }
            
            // Log unloading
            if unloaded_count > 0 {
                println!("🗑️  Unloaded {} chunks", unloaded_count);
            }
            
            // Sort chunks by distance from player (Manhattan distance for speed)
            if !to_load.is_empty() {
                let camera_chunk = voxel_engine::world_to_chunk(cam_pos);
                let mut sorted_chunks: Vec<IVec3> = to_load.into_iter().collect();
                
                // Sort by Manhattan distance (closest first)
                sorted_chunks.sort_by_key(|&chunk_pos| {
                    let delta = chunk_pos - camera_chunk;
                    delta.x.abs() + delta.y.abs() + delta.z.abs()
                });
                
                // Submit as batch job for parallel generation
                if !sorted_chunks.is_empty() {
                    self.job_queue.push(ChunkJob::GenerateBatch {
                        positions: sorted_chunks,
                    });
                }
            }
        }
    }
    
    /// Process completed jobs from worker threads
    fn process_completed_jobs(&mut self) {
        let results = self.job_queue.drain_completed();
        
        // FIX: Count jobs before consuming
        let job_count = results.len();
        
        if results.is_empty() {
            return;
        }
        
        // Track this for profiling
        self.profiler.jobs_completed = job_count;
        
        // Get frustum from render context for culling
        let frustum = self.gfx.as_ref().map(|g| g.frustum.clone());
        
        // Collect chunks to mesh in a batch (with frustum culling)
        let mut chunks_to_mesh = Vec::new();
        let mut culled_count = 0;
        
        for result in &results {
            match result {
                JobResult::Generated { position, chunk } => {
                    // Chunk generated, insert into manager
                    self.chunk_manager.insert(chunk.clone());
                    self.chunk_ring.mark_loaded(*position);
                    
                    // FRUSTUM CULLING: Only mesh chunks visible to camera
                    if let Some(ref frustum) = frustum {
                        // For now, use identity transform (static terrain)
                        // TODO: Support multiple grids with transforms
                        let grid_transform = glam::Mat4::IDENTITY;
                        let (min, max) = chunk.world_aabb(grid_transform);
                        
                        if frustum.intersects_aabb(min, max) {
                            // Chunk is visible, add to mesh batch
                            chunks_to_mesh.push((*position, Arc::new((*chunk).clone())));
                        } else {
                            // Chunk is outside frustum, skip meshing
                            culled_count += 1;
                        }
                    } else {
                        // No frustum available (shouldn't happen), mesh everything
                        chunks_to_mesh.push((*position, Arc::new((*chunk).clone())));
                    }
                }
                
                JobResult::Meshed { position, mesh } => {
                    // Mesh generated, upload to GPU
                    self.profiler.chunks_meshed += 1;
                    self.upload_chunk_mesh(*position, mesh.clone());
                }
                
                JobResult::MeshedBatch { meshes } => {
                    // Batch meshing completed, upload all meshes to GPU
                    self.profiler.chunks_meshed += meshes.len();
                    for (position, mesh) in meshes {
                        self.upload_chunk_mesh(*position, mesh.clone());
                    }
                }
                
                JobResult::Uploaded { position: _ } => {
                    // Upload complete (handled by upload_chunk_mesh)
                }
                
                JobResult::PhysicsReady { position: _ } => {
                    // Physics complete (future)
                }
            }
        }
        
        // Log culling statistics
        if culled_count > 0 {
            println!("🔍 Frustum culling: {} chunks outside view, {} chunks to mesh", 
                     culled_count, chunks_to_mesh.len());
        }
        
        // Submit batch mesh job if we have chunks to mesh
        if !chunks_to_mesh.is_empty() {
            self.job_queue.push(ChunkJob::MeshBatch {
                chunks: chunks_to_mesh,
            });
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
                // Profile: Rendering
                let render_start = Instant::now();
                
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
                    
                    // Get actual render stats AFTER rendering completes
                    let stats = &g.chunk_renderer.stats;
                    self.profiler.chunks_rendered = stats.visible_chunks;
                    self.profiler.draw_calls = stats.draw_calls as usize;
                }
                
                self.profiler.rendering_ms = render_start.elapsed().as_secs_f32() * 1000.0;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        // Begin frame profiling
        let frame_start = Instant::now();
        self.profiler.begin_frame();
        
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
            
            // Profile: Chunk Loading
            let t0 = Instant::now();
            self.update_chunk_loading();
            self.profiler.chunk_loading_ms = t0.elapsed().as_secs_f32() * 1000.0;
            
            // Profile: Job Processing
            let t1 = Instant::now();
            self.process_completed_jobs();
            self.profiler.job_processing_ms = t1.elapsed().as_secs_f32() * 1000.0;
            
            // Update job queue stats
            let job_stats = self.job_queue.get_stats();
            self.profiler.jobs_pending = job_stats.pending_count;
            self.profiler.jobs_completed = job_stats.completed_count;
            
            // Update chunk count
            self.profiler.chunks_loaded = self.chunk_ring.loaded_count();
            
            // Update dirty chunks (from block editing)
            self.update_dirty_chunks();
            
            self.engine.tick_once();
            self.next_tick += self.dt;
        }
        
        // End frame profiling
        let total_frame_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        self.profiler.end_frame(total_frame_ms);
        
        if let Some(w) = self.window {
            w.request_redraw(); }
    }
}
