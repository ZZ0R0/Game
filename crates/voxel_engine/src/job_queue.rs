//! Asynchronous job queue system for chunk operations
//! 
//! Pipeline: Generation → Meshing → Upload → Physics
//! 
//! Jobs are processed in parallel using rayon threadpool

use glam::IVec3;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::chunk::Chunk;
use crate::meshing::MeshData;

/// Job types in the pipeline
#[derive(Debug, Clone)]
pub enum ChunkJob {
    /// Generate terrain for a chunk
    Generate { position: IVec3 },
    
    /// Generate mesh from chunk data
    Mesh { position: IVec3, chunk: Arc<Chunk> },
    
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
}

#[derive(Debug, Clone, Default)]
pub struct JobStats {
    pub total_generated: u64,
    pub total_meshed: u64,
    pub total_uploaded: u64,
    pub pending_count: usize,
    pub completed_count: usize,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(VecDeque::new())),
            completed: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(JobStats::default())),
        }
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
                    
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        self.generate_chunk(position)
                    }));
                    
                    match result {
                        Ok(chunk) => {
                            let mut stats = self.stats.lock().unwrap();
                            stats.total_generated += 1;
                            Some(JobResult::Generated { position, chunk })
                        }
                        Err(e) => {
                            eprintln!("❌ PANIC in generate worker for chunk {:?}: {:?}", position, e);
                            None
                        }
                    }
                }
                
                ChunkJob::Mesh { position, chunk } => {
                    // Generate mesh from chunk with panic protection
                    use std::panic::catch_unwind;
                    use std::panic::AssertUnwindSafe;
                    
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        // Use greedy mesher without neighbors for async meshing
                        // We pass None for chunk_manager since we don't have access to it here
                        let atlas = crate::atlas::TextureAtlas::new_16x16();
                        crate::meshing::greedy_mesh_chunk(&chunk, None, &atlas)
                    }));
                    
                    match result {
                        Ok(mesh) => {
                            let mut stats = self.stats.lock().unwrap();
                            stats.total_meshed += 1;
                            Some(JobResult::Meshed { position, mesh })
                        }
                        Err(e) => {
                            eprintln!("❌ PANIC in mesh worker for chunk {:?}: {:?}", position, e);
                            // Return empty mesh or skip
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
    
    /// Placeholder: Generate a chunk (will use TerrainGenerator later)
    fn generate_chunk(&self, position: IVec3) -> Chunk {
        // For now, create an empty chunk
        // Will be replaced by terrain generator
        Chunk::new(position)
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
        
        WorkerHandle {
            running,
            handles,
        }
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
