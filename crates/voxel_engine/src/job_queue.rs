//! Simple job queue system for chunk operations
//!
//! Pipeline: Generation → Meshing → Upload
//!
//! Jobs are processed directly using Rayon (no worker threads!)

use crate::chunk::Chunk;
use crate::generator::TerrainGenerator;
use crate::meshing::MeshData;
use crate::meshing_config::MeshingConfig;
use glam::IVec3;
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

/// Thread-safe job queue (SIMPLIFIED - no workers!)
pub struct JobQueue {
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
            completed: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(JobStats::default())),
            terrain_generator: Arc::new(generator),
            meshing_config: Arc::new(meshing_config),
        }
    }

    /// Process a single job immediately using Rayon
    pub fn push(&self, job: ChunkJob) {
        // Process job immediately instead of queuing!
        match job {
            ChunkJob::GenerateBatch { positions } => {
                // Use Rayon to generate chunks in parallel
                use rayon::prelude::*;
                use std::time::Instant;

                let start = Instant::now();
                let generator = Arc::clone(&self.terrain_generator);
                
                // Generate all chunks in parallel with Rayon
                let chunks: Vec<_> = positions
                    .par_iter()
                    .map(|&position| {
                        let chunk = generator.generate_chunk(position);
                        (position, chunk)
                    })
                    .collect();

                let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                // Update stats
                let chunk_count = chunks.len() as u64;
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_generated += chunk_count;
                    stats.total_generation_time_ms += elapsed_ms;
                    stats.avg_generation_time_ms =
                        stats.total_generation_time_ms / stats.total_generated as f32;
                }

                // Add results
                {
                    let mut completed = self.completed.lock().unwrap();
                    for (position, chunk) in chunks {
                        completed.push(JobResult::Generated { position, chunk });
                    }
                }
            }

            ChunkJob::MeshBatch { chunks } => {
                // Use Rayon to mesh chunks in parallel
                use rayon::prelude::*;
                use std::time::Instant;

                let start = Instant::now();
                let meshing_config = Arc::clone(&self.meshing_config);
                let atlas = crate::atlas::TextureAtlas::new_16x16();
                
                // Create temporary ChunkManager for neighbors access
                let mut temp_manager = crate::ChunkManager::new();
                for (_pos, chunk) in &chunks {
                    temp_manager.insert((**chunk).clone());
                }

                // Mesh all chunks in parallel with Rayon
                let meshes: Vec<_> = chunks
                    .par_iter()
                    .map(|(position, chunk)| {
                        let mesh = meshing_config.mesh_chunk(chunk, Some(&temp_manager), &atlas);
                        (*position, mesh)
                    })
                    .collect();

                let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                // Update stats
                let chunk_count = meshes.len() as u64;
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_meshed += chunk_count;
                    stats.total_meshing_time_ms += elapsed_ms;
                    stats.avg_meshing_time_ms =
                        stats.total_meshing_time_ms / stats.total_meshed as f32;
                }

                // Add results
                {
                    let mut completed = self.completed.lock().unwrap();
                    completed.push(JobResult::MeshedBatch { meshes });
                }
            }

            ChunkJob::Generate { position } => {
                // Single chunk generation
                use std::time::Instant;

                let start = Instant::now();
                let generator = Arc::clone(&self.terrain_generator);
                let chunk = generator.generate_chunk(position);
                let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_generated += 1;
                    stats.total_generation_time_ms += elapsed_ms;
                    stats.avg_generation_time_ms =
                        stats.total_generation_time_ms / stats.total_generated as f32;
                }

                // Add result
                {
                    let mut completed = self.completed.lock().unwrap();
                    completed.push(JobResult::Generated { position, chunk });
                }
            }

            ChunkJob::Mesh { position, chunk } => {
                // Single chunk meshing
                use std::time::Instant;

                let start = Instant::now();
                let meshing_config = Arc::clone(&self.meshing_config);
                let atlas = crate::atlas::TextureAtlas::new_16x16();
                let mesh = meshing_config.mesh_chunk_standalone(&chunk, &atlas);
                let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;

                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_meshed += 1;
                    stats.total_meshing_time_ms += elapsed_ms;
                    stats.avg_meshing_time_ms =
                        stats.total_meshing_time_ms / stats.total_meshed as f32;
                }

                // Add result
                {
                    let mut completed = self.completed.lock().unwrap();
                    completed.push(JobResult::Meshed { position, mesh });
                }
            }

            ChunkJob::Upload { position, mesh: _ } => {
                // Upload is handled by main thread (GPU access)
                let mut stats = self.stats.lock().unwrap();
                stats.total_uploaded += 1;

                let mut completed = self.completed.lock().unwrap();
                completed.push(JobResult::Uploaded { position });
            }

            ChunkJob::Physics { position } => {
                // Physics generation (future implementation)
                let mut completed = self.completed.lock().unwrap();
                completed.push(JobResult::PhysicsReady { position });
            }
        }
    }

    /// Get all completed jobs (drains the queue)
    pub fn drain_completed(&self) -> Vec<JobResult> {
        let mut completed = self.completed.lock().unwrap();
        let results = completed.drain(..).collect();
        
        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.completed_count = completed.len();
            stats.pending_count = 0; // No pending queue anymore
        }
        
        results
    }

    /// Get current statistics
    pub fn get_stats(&self) -> JobStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}