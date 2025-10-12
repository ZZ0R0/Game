//! Asynchronous job queue system for chunk operations
//!
//! Pipeline: Generation → Meshing → Upload → Physics
//!
//! Jobs are processed in parallel using rayon threadpool

use crate::chunk::Chunk;
use crate::generator::TerrainGenerator;
use crate::meshing::MeshData;
use crate::meshing_config::MeshingConfig;
use glam::IVec3;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Job types in the pipeline
#[derive(Debug, Clone)]
pub enum ChunkJob {
    /// Generate terrain for a chunk
    Generate { position: IVec3 },

    /// Generate multiple chunks in parallel (uses all CPU cores)
    GenerateBatch { positions: Vec<IVec3> },

    /// Generate mesh from chunk data
    Mesh { position: IVec3, chunk: Arc<Chunk> },

    /// Mesh multiple chunks in parallel (uses all CPU cores)
    MeshBatch { chunks: Vec<(IVec3, Arc<Chunk>)> },

    /// Upload mesh to GPU
    Upload { position: IVec3, mesh: MeshData },

    /// Generate physics collider (optional, future)
    Physics { position: IVec3 },
}

/// Result of a completed job
#[derive(Debug)]
pub enum JobResult {
    /// Chunk generation completed
    Generated { position: IVec3, chunk: Chunk },

    /// Meshing completed
    Meshed { position: IVec3, mesh: MeshData },

    /// Batch meshing completed (multiple chunks meshed in parallel)
    MeshedBatch { meshes: Vec<(IVec3, MeshData)> },

    /// Upload completed
    Uploaded { position: IVec3 },

    /// Physics completed
    PhysicsReady { position: IVec3 },
}

/// Thread-safe job queue
pub struct JobQueue {
    /// Pending jobs (FIFO)
    pending: Arc<Mutex<VecDeque<ChunkJob>>>,

    /// Completed jobs waiting to be consumed
    completed: Arc<Mutex<Vec<JobResult>>>,

    /// Job statistics
    stats: Arc<Mutex<JobStats>>,

    /// Terrain generator for chunk generation
    terrain_generator: Arc<TerrainGenerator>,

    /// Meshing configuration (controls which algorithm to use)
    meshing_config: Arc<MeshingConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct JobStats {
    pub total_generated: u64,
    pub total_meshed: u64,
    pub total_uploaded: u64,
    pub pending_count: usize,
    pub completed_count: usize,

    // Timing stats (in milliseconds)
    pub avg_generation_time_ms: f32,
    pub avg_meshing_time_ms: f32,
    pub total_generation_time_ms: f32,
    pub total_meshing_time_ms: f32,
}

impl JobQueue {
    pub fn new() -> Self {
        Self::with_config(TerrainGenerator::default(), MeshingConfig::default())
    }

    /// Create a new job queue with custom configuration
    pub fn with_config(generator: TerrainGenerator, meshing_config: MeshingConfig) -> Self {
        Self {
            pending: Arc::new(Mutex::new(VecDeque::new())),
            completed: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(JobStats::default())),
            terrain_generator: Arc::new(generator),
            meshing_config: Arc::new(meshing_config),
        }
    }

    /// Create a new job queue with a custom terrain generator (deprecated, use with_config)
    #[deprecated(note = "Use with_config instead to specify both generator and meshing config")]
    pub fn with_generator(generator: TerrainGenerator) -> Self {
        Self::with_config(generator, MeshingConfig::default())
    }

    /// Add a job to the queue
    pub fn push(&self, job: ChunkJob) {
        let mut pending = self.pending.lock().unwrap();
        pending.push_back(job);

        let mut stats = self.stats.lock().unwrap();
        stats.pending_count = pending.len();
    }

    /// Add multiple jobs at once
    pub fn push_batch(&self, jobs: Vec<ChunkJob>) {
        let mut pending = self.pending.lock().unwrap();
        pending.extend(jobs);

        let mut stats = self.stats.lock().unwrap();
        stats.pending_count = pending.len();
    }

    /// Process pending jobs (should be called from worker threads)
    /// Returns number of jobs processed
    pub fn process_jobs(&self, max_jobs: usize) -> usize {
        let mut processed = 0;

        for _ in 0..max_jobs {
            // Pop a job from the queue
            let job = {
                let mut pending = self.pending.lock().unwrap();
                pending.pop_front()
            };

            let Some(job) = job else {
                break; // No more jobs
            };

            // Process the job
            let result = match job {
                ChunkJob::Generate { position } => {
                    // Generate terrain with panic protection
                    use std::panic::catch_unwind;
                    use std::panic::AssertUnwindSafe;
                    use std::time::Instant;

                    let start = Instant::now();
                    let generator = Arc::clone(&self.terrain_generator);
                    let result =
                        catch_unwind(AssertUnwindSafe(move || generator.generate_chunk(position)));
                    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                    match result {
                        Ok(chunk) => {
                            let mut stats = self.stats.lock().unwrap();
                            stats.total_generated += 1;
                            stats.total_generation_time_ms += elapsed_ms;
                            stats.avg_generation_time_ms =
                                stats.total_generation_time_ms / stats.total_generated as f32;
                            Some(JobResult::Generated { position, chunk })
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ PANIC in generate worker for chunk {:?}: {:?}",
                                position, e
                            );
                            None
                        }
                    }
                }

                ChunkJob::GenerateBatch { positions } => {
                    // Generate multiple chunks in parallel using rayon
                    use std::panic::catch_unwind;
                    use std::panic::AssertUnwindSafe;
                    use std::time::Instant;

                    let start = Instant::now();
                    let num_chunks = positions.len();
                    let generator = Arc::clone(&self.terrain_generator);
                    let result = catch_unwind(AssertUnwindSafe(move || {
                        generator.generate_chunks_parallel(&positions)
                    }));
                    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                    match result {
                        Ok(chunks) => {
                            // Update stats
                            let chunk_count = chunks.len() as u64;
                            let mut stats = self.stats.lock().unwrap();
                            stats.total_generated += chunk_count;
                            stats.total_generation_time_ms += elapsed_ms;
                            stats.avg_generation_time_ms =
                                stats.total_generation_time_ms / stats.total_generated as f32;
                            drop(stats);

                            // Push all generated chunks to completed queue
                            let mut completed = self.completed.lock().unwrap();
                            for (position, chunk) in chunks {
                                completed.push(JobResult::Generated { position, chunk });
                            }

                            None // We already pushed results directly
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ PANIC in batch generate worker for {} chunks: {:?}",
                                num_chunks, e
                            );
                            None
                        }
                    }
                }

                ChunkJob::Mesh { position, chunk } => {
                    // Generate mesh from chunk with panic protection
                    use std::panic::catch_unwind;
                    use std::panic::AssertUnwindSafe;
                    use std::time::Instant;

                    let start = Instant::now();
                    let meshing_config = Arc::clone(&self.meshing_config);
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        // Use configured meshing strategy
                        let atlas = crate::atlas::TextureAtlas::new_16x16();
                        meshing_config.mesh_chunk_standalone(&chunk, &atlas)
                    }));
                    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                    match result {
                        Ok(mesh) => {
                            let mut stats = self.stats.lock().unwrap();
                            stats.total_meshed += 1;
                            stats.total_meshing_time_ms += elapsed_ms;
                            stats.avg_meshing_time_ms =
                                stats.total_meshing_time_ms / stats.total_meshed as f32;
                            Some(JobResult::Meshed { position, mesh })
                        }
                        Err(e) => {
                            eprintln!("❌ PANIC in mesh worker for chunk {:?}: {:?}", position, e);
                            // Return empty mesh or skip
                            None
                        }
                    }
                }

                ChunkJob::MeshBatch { chunks } => {
                    // Mesh multiple chunks in parallel using rayon
                    use std::panic::catch_unwind;
                    use std::panic::AssertUnwindSafe;
                    use std::time::Instant;

                    let start = Instant::now();
                    let num_chunks = chunks.len();

                    // We need ChunkManager to access neighbors
                    // For now, we'll create a temporary ChunkManager from the chunks
                    let mut temp_manager = crate::ChunkManager::new();
                    for (_pos, chunk) in &chunks {
                        temp_manager.insert((**chunk).clone());
                    }

                    let atlas = crate::atlas::TextureAtlas::new_16x16();
                    let meshing_config = Arc::clone(&self.meshing_config);
                    let result = catch_unwind(AssertUnwindSafe(move || {
                        // Simple sequential meshing (replaced parallel meshing)
                        chunks
                            .iter()
                            .map(|(position, chunk)| {
                                let mesh = meshing_config.mesh_chunk(chunk, Some(&temp_manager), &atlas);
                                (*position, mesh)
                            })
                            .collect::<Vec<_>>()
                    }));
                    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                    match result {
                        Ok(meshes) => {
                            // Update stats
                            let chunk_count = meshes.len() as u64;
                            let mut stats = self.stats.lock().unwrap();
                            stats.total_meshed += chunk_count;
                            stats.total_meshing_time_ms += elapsed_ms;
                            stats.avg_meshing_time_ms =
                                stats.total_meshing_time_ms / stats.total_meshed as f32;

                            Some(JobResult::MeshedBatch { meshes })
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ PANIC in batch mesh worker for {} chunks: {:?}",
                                num_chunks, e
                            );
                            None
                        }
                    }
                }

                ChunkJob::Upload { position, mesh: _ } => {
                    // Upload will be handled by main thread (GPU access)
                    // Just mark as completed
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_uploaded += 1;

                    Some(JobResult::Uploaded { position })
                }

                ChunkJob::Physics { position } => {
                    // Physics generation (future implementation)
                    Some(JobResult::PhysicsReady { position })
                }
            };

            // Store result
            if let Some(result) = result {
                let mut completed = self.completed.lock().unwrap();
                completed.push(result);
            }

            processed += 1;
        }

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            let pending = self.pending.lock().unwrap();
            let completed = self.completed.lock().unwrap();
            stats.pending_count = pending.len();
            stats.completed_count = completed.len();
        }

        processed
    }

    /// Get all completed jobs (drains the queue)
    pub fn drain_completed(&self) -> Vec<JobResult> {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        // Wrap entire operation in catch_unwind to handle panics gracefully
        let result = catch_unwind(AssertUnwindSafe(|| {
            // Handle poisoned mutex gracefully
            let mut completed = match self.completed.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    eprintln!("⚠️  WARNING: Completed mutex was poisoned (worker thread panicked), recovering...");
                    poisoned.into_inner()
                }
            };

            // Use mem::take to avoid iterator issues
            let results = std::mem::take(&mut *completed);

            // Explicitly drop the guard BEFORE we try to lock stats
            drop(completed);

            let mut stats = match self.stats.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    eprintln!("⚠️  WARNING: Stats mutex was poisoned, recovering...");
                    poisoned.into_inner()
                }
            };
            stats.completed_count = 0;
            drop(stats);

            results
        }));

        match result {
            Ok(results) => results,
            Err(e) => {
                eprintln!("❌ PANIC in drain_completed: {:?}", e);
                Vec::new() // Return empty vec
            }
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> JobStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clear all pending jobs
    pub fn clear(&self) {
        let mut pending = self.pending.lock().unwrap();
        pending.clear();

        let mut completed = self.completed.lock().unwrap();
        completed.clear();

        let mut stats = self.stats.lock().unwrap();
        stats.pending_count = 0;
        stats.completed_count = 0;
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Worker thread pool for processing jobs
pub struct JobWorker {
    queue: Arc<JobQueue>,
    worker_count: usize,
}

impl JobWorker {
    pub fn new(queue: Arc<JobQueue>, worker_count: usize) -> Self {
        Self {
            queue,
            worker_count,
        }
    }

    /// Start processing jobs in background threads
    /// Returns a handle that stops workers when dropped
    pub fn start(self) -> WorkerHandle {
        let running = Arc::new(Mutex::new(true));
        let mut handles = Vec::new();

        for worker_id in 0..self.worker_count {
            let queue = Arc::clone(&self.queue);
            let running = Arc::clone(&running);

            let handle = std::thread::spawn(move || {
                loop {
                    // Check if we should stop
                    {
                        let should_run = running.lock().unwrap();
                        if !*should_run {
                            break;
                        }
                    }

                    // Process some jobs
                    let processed = queue.process_jobs(10);

                    if processed == 0 {
                        // No jobs, sleep briefly
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }

                println!("Worker {} stopped", worker_id);
            });

            handles.push(handle);
        }

        WorkerHandle { running, handles }
    }
}

/// Handle to stop worker threads
pub struct WorkerHandle {
    running: Arc<Mutex<bool>>,
    handles: Vec<std::thread::JoinHandle<()>>,
}

impl WorkerHandle {
    /// Stop all workers and wait for them to finish
    pub fn stop(mut self) {
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}
