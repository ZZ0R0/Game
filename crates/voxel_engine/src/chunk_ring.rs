//! Chunk ring system: manages which chunks to load/unload based on camera position
//! 
//! Concept:
//! - View radius: chunks visible to player (rendered)
//! - Generation radius: chunks that should be generated (usually larger than view)
//! - Unload radius: chunks beyond this are unloaded

use glam::{IVec3, Vec3};
use std::collections::HashSet;

/// Configuration for chunk loading
#[derive(Debug, Clone)]
pub struct ChunkRingConfig {
    /// View distance in chunks (chunks rendered)
    pub view_radius: i32,
    
    /// Generation distance in chunks (chunks to generate)
    pub generation_radius: i32,
    
    /// Unload distance in chunks (chunks to unload when beyond this)
    pub unload_radius: i32,
}

impl Default for ChunkRingConfig {
    fn default() -> Self {
        Self {
            view_radius: 8,          // 8 chunks = 256 blocks
            generation_radius: 10,   // Generate 2 chunks ahead
            unload_radius: 12,       // Unload 4 chunks beyond generation
        }
    }
}

/// Manages chunk loading/unloading in a ring around the camera
pub struct ChunkRing {
    config: ChunkRingConfig,
    
    /// Current camera chunk position (last update)
    current_chunk: IVec3,
    
    /// Chunks that should be loaded (within generation radius)
    desired_chunks: HashSet<IVec3>,
    
    /// Chunks currently loaded
    loaded_chunks: HashSet<IVec3>,
}

impl ChunkRing {
    pub fn new(config: ChunkRingConfig) -> Self {
        Self {
            config,
            current_chunk: IVec3::ZERO,
            desired_chunks: HashSet::new(),
            loaded_chunks: HashSet::new(),
        }
    }
    
    /// Update the chunk ring based on camera position
    /// Returns (chunks_to_load, chunks_to_unload)
    pub fn update(&mut self, camera_pos: Vec3) -> (Vec<IVec3>, Vec<IVec3>) {
        // Convert camera world position to chunk position
        let chunk_pos = world_to_chunk(camera_pos);
        
        // Only update if camera moved to a different chunk
        if chunk_pos == self.current_chunk {
            return (Vec::new(), Vec::new());
        }
        
        self.current_chunk = chunk_pos;
        
        // Calculate desired chunks (all chunks within generation radius)
        self.desired_chunks = self.calculate_chunks_in_radius(chunk_pos, self.config.generation_radius);
        
        // Find chunks to load (desired but not loaded)
        let chunks_to_load: Vec<IVec3> = self.desired_chunks
            .difference(&self.loaded_chunks)
            .copied()
            .collect();
        
        // Find chunks to unload (loaded but beyond unload radius)
        let unload_set = self.calculate_chunks_in_radius(chunk_pos, self.config.unload_radius);
        let chunks_to_unload: Vec<IVec3> = self.loaded_chunks
            .difference(&unload_set)
            .copied()
            .collect();
        
        (chunks_to_load, chunks_to_unload)
    }
    
    /// Mark a chunk as loaded
    pub fn mark_loaded(&mut self, chunk_pos: IVec3) {
        self.loaded_chunks.insert(chunk_pos);
    }
    
    /// Mark a chunk as unloaded
    pub fn mark_unloaded(&mut self, chunk_pos: IVec3) {
        self.loaded_chunks.remove(&chunk_pos);
    }
    
    /// Get all chunks within view radius (for rendering)
    pub fn get_visible_chunks(&self) -> Vec<IVec3> {
        self.calculate_chunks_in_radius(self.current_chunk, self.config.view_radius)
            .into_iter()
            .filter(|pos| self.loaded_chunks.contains(pos))
            .collect()
    }
    
    /// Calculate all chunks within a radius (cylindrical, on XZ plane, full Y)
    fn calculate_chunks_in_radius(&self, center: IVec3, radius: i32) -> HashSet<IVec3> {
        let mut chunks = HashSet::new();
        
        let radius_sq = radius * radius;
        
        // Iterate over XZ plane (horizontal)
        for x in (center.x - radius)..=(center.x + radius) {
            for z in (center.z - radius)..=(center.z + radius) {
                let dx = x - center.x;
                let dz = z - center.z;
                let dist_sq = dx * dx + dz * dz;
                
                if dist_sq <= radius_sq {
                    // Full Y range (vertical columns)
                    // For now, load chunks at Y=0 only (flat world)
                    // Can be extended to support vertical chunks
                    chunks.insert(IVec3::new(x, 0, z));
                }
            }
        }
        
        chunks
    }
    
    /// Get current camera chunk position
    pub fn current_chunk(&self) -> IVec3 {
        self.current_chunk
    }
    
    /// Get number of loaded chunks
    pub fn loaded_count(&self) -> usize {
        self.loaded_chunks.len()
    }
    
    /// Get number of desired chunks
    pub fn desired_count(&self) -> usize {
        self.desired_chunks.len()
    }
    
    /// Clear all loaded chunks (for reset)
    pub fn clear(&mut self) {
        self.loaded_chunks.clear();
        self.desired_chunks.clear();
    }
}

/// Convert world position to chunk position
pub fn world_to_chunk(world_pos: Vec3) -> IVec3 {
    const CHUNK_SIZE: i32 = 32;
    IVec3::new(
        (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
    )
}

/// Convert chunk position to world position (center of chunk)
pub fn chunk_to_world(chunk_pos: IVec3) -> Vec3 {
    const CHUNK_SIZE: f32 = 32.0;
    Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
        chunk_pos.y as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
        chunk_pos.z as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
    )
}
