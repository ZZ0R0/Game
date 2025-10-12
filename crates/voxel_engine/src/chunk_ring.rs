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
            view_radius: 4,       // 4 chunks = 128 blocks (OPTIMIZED for performance)
            generation_radius: 5, // Generate 1 chunk ahead
            unload_radius: 6,     // Unload 1 chunk beyond generation
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

        // Calculate desired chunks using world-based distance for more precision
        const CHUNK_SIZE: f32 = 32.0;
        let generation_radius_world = self.config.generation_radius as f32 * CHUNK_SIZE;
        self.desired_chunks = self.calculate_chunks_in_world_radius(camera_pos, generation_radius_world);

        // Find chunks to load (desired but not loaded)
        let chunks_to_load: Vec<IVec3> = self
            .desired_chunks
            .difference(&self.loaded_chunks)
            .copied()
            .collect();

        // Find chunks to unload (loaded but beyond unload radius)
        let unload_radius_world = self.config.unload_radius as f32 * CHUNK_SIZE;
        let unload_set = self.calculate_chunks_in_world_radius(camera_pos, unload_radius_world);
        let chunks_to_unload: Vec<IVec3> = self
            .loaded_chunks
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

    /// Calculate all chunks within a radius (spherical, 3D)
    ///
    /// Uses true spherical distance in 3D space for optimal chunk loading.
    ///
    /// Expected chunk counts:
    /// - radius 4: ~268 chunks (4/3 × π × 4³ ≈ 268)
    /// - radius 5: ~523 chunks (4/3 × π × 5³ ≈ 523)
    /// - radius 6: ~904 chunks (4/3 × π × 6³ ≈ 904)
    /// - radius 8: ~2144 chunks (4/3 × π × 8³ ≈ 2144)
    ///
    /// Note: Actual count is slightly less due to discrete grid sampling
    fn calculate_chunks_in_radius(&self, center: IVec3, radius: i32) -> HashSet<IVec3> {
        let mut chunks = HashSet::new();

        let radius_sq = radius * radius;

        // Iterate over 3D space (spherical volume)
        for x in (center.x - radius)..=(center.x + radius) {
            for y in (center.y - radius)..=(center.y + radius) {
                for z in (center.z - radius)..=(center.z + radius) {
                    let dx = x - center.x;
                    let dy = y - center.y;
                    let dz = z - center.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;

                    // Spherical distance check
                    if dist_sq <= radius_sq {
                        chunks.insert(IVec3::new(x, y, z));
                    }
                }
            }
        }

        chunks
    }

    /// Calculate all chunks within a world distance from player position
    /// This is more accurate than chunk-based radius for edge cases
    fn calculate_chunks_in_world_radius(&self, player_pos: Vec3, world_radius: f32) -> HashSet<IVec3> {
        let mut chunks = HashSet::new();
        
        // Convert world radius to chunk radius (with some margin)
        const CHUNK_SIZE: f32 = 32.0;
        let chunk_radius = ((world_radius / CHUNK_SIZE).ceil() as i32) + 1;
        let center_chunk = world_to_chunk(player_pos);
        
        let world_radius_sq = world_radius * world_radius;

        // Iterate over potential chunk area
        for x in (center_chunk.x - chunk_radius)..=(center_chunk.x + chunk_radius) {
            for y in (center_chunk.y - chunk_radius)..=(center_chunk.y + chunk_radius) {
                for z in (center_chunk.z - chunk_radius)..=(center_chunk.z + chunk_radius) {
                    let chunk_pos = IVec3::new(x, y, z);
                    
                    // Check actual distance from player to chunk center
                    if distance_to_chunk_squared(player_pos, chunk_pos) <= world_radius_sq {
                        chunks.insert(chunk_pos);
                    }
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

/// Calculate distance from player position to chunk center
pub fn distance_to_chunk(player_pos: Vec3, chunk_pos: IVec3) -> f32 {
    let chunk_center = chunk_to_world(chunk_pos);
    (player_pos - chunk_center).length()
}

/// Calculate squared distance from player position to chunk center (faster for comparisons)
pub fn distance_to_chunk_squared(player_pos: Vec3, chunk_pos: IVec3) -> f32 {
    let chunk_center = chunk_to_world(chunk_pos);
    (player_pos - chunk_center).length_squared()
}
