//! Advanced chunk system with BlockId palette and world coordinates
//! 
//! Architecture:
//! - BlockId: u16 (65536 possible block types)
//! - Palette: Maps local indices to global BlockIds (compression)
//! - Chunk: 32³ voxels = 32,768 voxels
//! - Storage: ~64 KiB per chunk (with u16 BlockIds)
//! - Dirty flags: Track changes for mesh/physics updates

use glam::IVec3;
use std::collections::HashMap;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE; // 32,768

/// Block identifier (u16 = 65,536 possible block types)
pub type BlockId = u16;

/// Special block IDs
pub const AIR: BlockId = 0;
pub const STONE: BlockId = 1;
pub const DIRT: BlockId = 2;
pub const GRASS: BlockId = 3;
pub const WOOD: BlockId = 4;
pub const LEAVES: BlockId = 5;
pub const WATER: BlockId = 6;
pub const GLASS: BlockId = 7;

/// Check if a block is transparent (needs special rendering)
pub fn is_transparent(block_id: BlockId) -> bool {
    matches!(block_id, WATER | GLASS)
}

/// Check if a block is solid (not air, not transparent)
pub fn is_solid(block_id: BlockId) -> bool {
    block_id != AIR && !is_transparent(block_id)
}

/// Dirty flags for optimizing updates
#[derive(Debug, Clone, Copy, Default)]
pub struct DirtyFlags {
    pub voxels: bool,    // Voxel data changed
    pub mesh: bool,      // Mesh needs regeneration
    pub physics: bool,   // Physics collider needs update
}

impl DirtyFlags {
    pub fn mark_all(&mut self) {
        self.voxels = true;
        self.mesh = true;
        self.physics = true;
    }
    
    pub fn clear(&mut self) {
        self.voxels = false;
        self.mesh = false;
        self.physics = false;
    }
}

/// Chunk with BlockId palette system
/// Memory: 32³ × 2 bytes = 65,536 bytes = 64 KiB
#[derive(Clone)]
pub struct Chunk {
    /// World position of this chunk (in chunk coordinates)
    pub position: IVec3,
    
    /// Block data: 32³ BlockIds (u16)
    /// Index = x + y*32 + z*32²
    blocks: Box<[BlockId; CHUNK_VOLUME]>,
    
    /// Palette: maps local palette index → global BlockId
    /// Useful for compression (not yet implemented)
    palette: Vec<BlockId>,
    
    /// Dirty flags for optimization
    pub dirty: DirtyFlags,
}

impl Chunk {
    /// Create a new empty chunk at the given world position
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            blocks: Box::new([AIR; CHUNK_VOLUME]),
            palette: vec![AIR], // Start with just AIR
            dirty: DirtyFlags::default(),
        }
    }
    
    /// Create a chunk filled with a specific block
    pub fn new_filled(position: IVec3, block: BlockId) -> Self {
        let mut chunk = Self {
            position,
            blocks: Box::new([block; CHUNK_VOLUME]),
            palette: vec![block],
            dirty: DirtyFlags::default(),
        };
        chunk.dirty.mark_all();
        chunk
    }
    
    /// Get block at local coordinates (0..31)
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        self.blocks[Self::index(x, y, z)]
    }
    
    /// Set block at local coordinates (0..31)
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        let idx = Self::index(x, y, z);
        if self.blocks[idx] != block {
            self.blocks[idx] = block;
            self.dirty.mark_all();
            
            // Update palette if needed
            if !self.palette.contains(&block) {
                self.palette.push(block);
            }
        }
    }
    
    /// Calculate flat array index from 3D coordinates
    #[inline]
    const fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
    
    /// Get block at world coordinates
    pub fn get_world(&self, world_pos: IVec3) -> Option<BlockId> {
        let local = Self::world_to_local(world_pos, self.position)?;
        Some(self.get(local.x as usize, local.y as usize, local.z as usize))
    }
    
    /// Set block at world coordinates
    pub fn set_world(&mut self, world_pos: IVec3, block: BlockId) -> bool {
        if let Some(local) = Self::world_to_local(world_pos, self.position) {
            self.set(local.x as usize, local.y as usize, local.z as usize, block);
            true
        } else {
            false
        }
    }
    
    /// Convert world coordinates to local chunk coordinates
    pub fn world_to_local(world_pos: IVec3, chunk_pos: IVec3) -> Option<IVec3> {
        let local = world_pos - chunk_pos * CHUNK_SIZE as i32;
        if local.x >= 0 && local.x < CHUNK_SIZE as i32
            && local.y >= 0 && local.y < CHUNK_SIZE as i32
            && local.z >= 0 && local.z < CHUNK_SIZE as i32
        {
            Some(local)
        } else {
            None
        }
    }
    
    /// Convert local coordinates to world coordinates
    pub fn local_to_world(&self, local: IVec3) -> IVec3 {
        self.position * CHUNK_SIZE as i32 + local
    }
    
    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() 
            + self.blocks.len() * std::mem::size_of::<BlockId>()
            + self.palette.len() * std::mem::size_of::<BlockId>()
    }
    
    /// Fill with test pattern (debug)
    pub fn fill_debug_pattern(&mut self) {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    // Ground layer
                    if y == 0 {
                        self.set(x, y, z, STONE);
                    }
                    // Pillars
                    else if (x % 4 == 0) && (z % 4 == 0) && y < 8 {
                        self.set(x, y, z, WOOD);
                    }
                }
            }
        }
    }
    
    /// Fill with GPU stress test pattern
    pub fn fill_gpu_stress_test(&mut self) {
        let center = CHUNK_SIZE as f32 / 2.0;
        
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let fx = x as f32 - center;
                    let fy = y as f32 - center;
                    let fz = z as f32 - center;
                    
                    let dist = (fx * fx + fy * fy + fz * fz).sqrt();
                    
                    // Multiple spherical shells with different materials
                    if dist < 14.0 && dist > 12.0 {
                        self.set(x, y, z, STONE);
                    } else if dist < 10.0 && dist > 8.0 {
                        self.set(x, y, z, WOOD);
                    } else if dist < 6.0 && dist > 4.0 {
                        self.set(x, y, z, LEAVES);
                    }
                    // Spiral pattern
                    else if dist > 5.0 && dist < 13.0 {
                        let angle = fy.atan2(fx);
                        if ((angle + dist * 0.5).sin() * 2.0).abs() < 1.0 {
                            self.set(x, y, z, GRASS);
                        }
                    }
                    // Checkerboard core
                    else if dist < 3.0 && (x + y + z) % 2 == 0 {
                        self.set(x, y, z, DIRT);
                    }
                    // Floor and ceiling
                    else if y == 0 || y == CHUNK_SIZE - 1 {
                        self.set(x, y, z, STONE);
                    }
                }
            }
        }
    }
}

/// ChunkManager: handles multiple chunks and world↔chunk coordinate mapping
pub struct ChunkManager {
    /// Map of chunk position → chunk data
    chunks: HashMap<IVec3, Chunk>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    
    /// Add or replace a chunk
    pub fn insert(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }
    
    /// Get chunk at chunk coordinates
    pub fn get_chunk(&self, chunk_pos: IVec3) -> Option<&Chunk> {
        self.chunks.get(&chunk_pos)
    }
    
    /// Get mutable chunk at chunk coordinates
    pub fn get_chunk_mut(&mut self, chunk_pos: IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(&chunk_pos)
    }
    
    /// Remove a chunk
    pub fn remove(&mut self, chunk_pos: IVec3) -> Option<Chunk> {
        self.chunks.remove(&chunk_pos)
    }
    
    /// Get block at world coordinates
    pub fn get_block(&self, world_pos: IVec3) -> BlockId {
        let chunk_pos = Self::world_to_chunk(world_pos);
        self.chunks
            .get(&chunk_pos)
            .and_then(|chunk| chunk.get_world(world_pos))
            .unwrap_or(AIR)
    }
    
    /// Set block at world coordinates
    pub fn set_block(&mut self, world_pos: IVec3, block: BlockId) -> bool {
        let chunk_pos = Self::world_to_chunk(world_pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_world(world_pos, block)
        } else {
            false
        }
    }
    
    /// Convert world coordinates to chunk coordinates
    pub fn world_to_chunk(world_pos: IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.div_euclid(CHUNK_SIZE as i32),
            world_pos.y.div_euclid(CHUNK_SIZE as i32),
            world_pos.z.div_euclid(CHUNK_SIZE as i32),
        )
    }
    
    /// Get all neighbor chunk positions (26 neighbors + self = 27 total)
    pub fn get_neighbors(chunk_pos: IVec3) -> [IVec3; 27] {
        let mut neighbors = [IVec3::ZERO; 27];
        let mut idx = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    neighbors[idx] = chunk_pos + IVec3::new(dx, dy, dz);
                    idx += 1;
                }
            }
        }
        neighbors
    }
    
    /// Get 6 adjacent neighbors (face-adjacent only)
    pub fn get_adjacent_neighbors(chunk_pos: IVec3) -> [IVec3; 6] {
        [
            chunk_pos + IVec3::new(1, 0, 0),   // +X
            chunk_pos + IVec3::new(-1, 0, 0),  // -X
            chunk_pos + IVec3::new(0, 1, 0),   // +Y
            chunk_pos + IVec3::new(0, -1, 0),  // -Y
            chunk_pos + IVec3::new(0, 0, 1),   // +Z
            chunk_pos + IVec3::new(0, 0, -1),  // -Z
        ]
    }
    
    /// Get all chunks that need mesh updates
    pub fn get_dirty_chunks(&self) -> Vec<IVec3> {
        self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.dirty.mesh)
            .map(|(pos, _)| *pos)
            .collect()
    }
    
    /// Clear dirty flags for a chunk
    pub fn clear_dirty(&mut self, chunk_pos: IVec3) {
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.dirty.clear();
        }
    }
    
    /// Get total number of chunks loaded
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
    
    /// Get total memory usage
    pub fn total_memory_usage(&self) -> usize {
        self.chunks.values().map(|c| c.memory_usage()).sum()
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chunk_coordinates() {
        let chunk = Chunk::new(IVec3::new(1, 0, 2));
        
        // Local (0,0,0) in chunk (1,0,2) = world (32, 0, 64)
        let world = chunk.local_to_world(IVec3::ZERO);
        assert_eq!(world, IVec3::new(32, 0, 64));
        
        // World (32, 0, 64) should map to chunk (1, 0, 2)
        assert_eq!(ChunkManager::world_to_chunk(world), IVec3::new(1, 0, 2));
    }
    
    #[test]
    fn test_dirty_flags() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        assert!(!chunk.dirty.voxels);
        
        chunk.set(0, 0, 0, STONE);
        assert!(chunk.dirty.voxels);
        assert!(chunk.dirty.mesh);
        
        chunk.dirty.clear();
        assert!(!chunk.dirty.voxels);
    }
    
    #[test]
    fn test_memory_size() {
        let chunk = Chunk::new(IVec3::ZERO);
        let mem = chunk.memory_usage();
        println!("Chunk memory usage: {} bytes ({} KiB)", mem, mem / 1024);
        // Should be around 64 KiB for the blocks array
        assert!(mem >= 65536); // At least 64 KiB
    }
}
