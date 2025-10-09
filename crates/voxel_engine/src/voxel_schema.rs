//! Voxel schema definitions for different data representations
//! 
//! Milestone 1: Two schema types:
//! 1. BlockSchema: Solid/Air + BlockId palette (current system)
//! 2. DensitySchema: 8-bit density + 8-bit material (smooth terrain, marching cubes)

use glam::IVec3;

/// Material identifier (8-bit = 256 possible materials)
pub type MaterialId = u8;

/// Density value (8-bit = 256 levels, 0 = empty, 255 = fully solid)
pub type Density = u8;

/// Special material IDs
pub const MAT_AIR: MaterialId = 0;
pub const MAT_STONE: MaterialId = 1;
pub const MAT_DIRT: MaterialId = 2;
pub const MAT_GRASS: MaterialId = 3;
pub const MAT_WOOD: MaterialId = 4;
pub const MAT_WATER: MaterialId = 5;

/// Core trait that all voxel schemas must implement
/// 
/// This allows unified operations on different data representations:
/// - BlockSchema: Discrete blocks (Minecraft-style)
/// - DensitySchema: Smooth density fields (marching cubes)
pub trait VoxelSchema: Send + Sync {
    /// Check if a voxel is solid at the given position
    /// 
    /// Returns true if the voxel should have collision/rendering
    fn is_solid(&self, pos: IVec3) -> bool;
    
    /// Get the material at the given position
    /// 
    /// Returns MaterialId (or 0/AIR if no material)
    fn material_at(&self, pos: IVec3) -> MaterialId;
    
    /// Get the surface sign at a position (for smooth terrain)
    /// 
    /// Returns:
    /// - Positive: Inside solid material
    /// - Negative: Outside/empty space
    /// - Zero: Exactly on the surface
    /// 
    /// For block-based systems, returns discrete values:
    /// - +1.0 if solid
    /// - -1.0 if air
    fn surface_sign(&self, pos: IVec3) -> f32;
    
    /// Get a human-readable name for this schema type
    fn schema_name(&self) -> &str;
}

/// Block-based voxel schema (current system)
/// 
/// Data representation:
/// - Each voxel stores a BlockId (u16)
/// - BlockId 0 = AIR (empty)
/// - BlockId > 0 = Solid blocks with different types
/// 
/// Memory: 32³ × 2 bytes = 64 KiB per chunk
#[derive(Debug, Clone)]
pub struct BlockSchema {
    /// Block data: 32³ BlockIds
    blocks: Box<[crate::chunk::BlockId; crate::chunk::CHUNK_VOLUME]>,
    
    /// World position of this chunk (for coordinate conversions)
    chunk_pos: IVec3,
}

impl BlockSchema {
    /// Create a new empty block schema
    pub fn new(chunk_pos: IVec3) -> Self {
        Self {
            blocks: Box::new([crate::chunk::AIR; crate::chunk::CHUNK_VOLUME]),
            chunk_pos,
        }
    }
    
    /// Create from existing block data
    pub fn from_blocks(chunk_pos: IVec3, blocks: Box<[crate::chunk::BlockId; crate::chunk::CHUNK_VOLUME]>) -> Self {
        Self { blocks, chunk_pos }
    }
    
    /// Set block at local coordinates (0..31)
    pub fn set_local(&mut self, x: usize, y: usize, z: usize, block: crate::chunk::BlockId) {
        let idx = x + y * crate::chunk::CHUNK_SIZE + z * crate::chunk::CHUNK_SIZE * crate::chunk::CHUNK_SIZE;
        self.blocks[idx] = block;
    }
    
    /// Get block at local coordinates
    pub fn get_local(&self, x: usize, y: usize, z: usize) -> crate::chunk::BlockId {
        let idx = x + y * crate::chunk::CHUNK_SIZE + z * crate::chunk::CHUNK_SIZE * crate::chunk::CHUNK_SIZE;
        self.blocks[idx]
    }
    
    /// Convert world pos to local coordinates
    fn world_to_local(&self, pos: IVec3) -> Option<(usize, usize, usize)> {
        let local = pos - self.chunk_pos * crate::chunk::CHUNK_SIZE as i32;
        if local.x >= 0 && local.x < crate::chunk::CHUNK_SIZE as i32
            && local.y >= 0 && local.y < crate::chunk::CHUNK_SIZE as i32
            && local.z >= 0 && local.z < crate::chunk::CHUNK_SIZE as i32 {
            Some((local.x as usize, local.y as usize, local.z as usize))
        } else {
            None
        }
    }
}

impl VoxelSchema for BlockSchema {
    fn is_solid(&self, pos: IVec3) -> bool {
        if let Some((x, y, z)) = self.world_to_local(pos) {
            let block = self.get_local(x, y, z);
            crate::chunk::is_solid(block)
        } else {
            false
        }
    }
    
    fn material_at(&self, pos: IVec3) -> MaterialId {
        if let Some((x, y, z)) = self.world_to_local(pos) {
            let block = self.get_local(x, y, z);
            // Map BlockId to MaterialId (simplified for now)
            match block {
                crate::chunk::AIR => MAT_AIR,
                crate::chunk::STONE => MAT_STONE,
                crate::chunk::DIRT => MAT_DIRT,
                crate::chunk::GRASS => MAT_GRASS,
                crate::chunk::WOOD => MAT_WOOD,
                crate::chunk::WATER => MAT_WATER,
                _ => MAT_STONE, // Default to stone
            }
        } else {
            MAT_AIR
        }
    }
    
    fn surface_sign(&self, pos: IVec3) -> f32 {
        if self.is_solid(pos) {
            1.0  // Inside solid
        } else {
            -1.0 // Outside/air
        }
    }
    
    fn schema_name(&self) -> &str {
        "BlockSchema"
    }
}

/// Density-based voxel schema (for smooth terrain)
/// 
/// Data representation:
/// - Each voxel stores: (Density: u8, MaterialId: u8)
/// - Density: 0 = empty, 255 = fully solid
/// - MaterialId: Which material (stone, dirt, etc.)
/// 
/// Memory: 32³ × 2 bytes = 64 KiB per chunk
/// 
/// Use cases:
/// - Smooth terrain (marching cubes)
/// - Caves and overhangs
/// - Destructible terrain with smooth edges
#[derive(Debug, Clone)]
pub struct DensitySchema {
    /// Density + Material data: 32³ × 2 bytes
    /// Stored as interleaved pairs: [density0, material0, density1, material1, ...]
    data: Box<[u8; crate::chunk::CHUNK_VOLUME * 2]>,
    
    /// World position of this chunk
    chunk_pos: IVec3,
    
    /// Threshold for considering a voxel "solid"
    /// Default: 128 (50% density)
    solid_threshold: Density,
}

impl DensitySchema {
    /// Create a new empty density schema
    pub fn new(chunk_pos: IVec3) -> Self {
        Self {
            data: Box::new([0u8; crate::chunk::CHUNK_VOLUME * 2]),
            chunk_pos,
            solid_threshold: 128,
        }
    }
    
    /// Set density and material at local coordinates
    pub fn set_local(&mut self, x: usize, y: usize, z: usize, density: Density, material: MaterialId) {
        let idx = (x + y * crate::chunk::CHUNK_SIZE + z * crate::chunk::CHUNK_SIZE * crate::chunk::CHUNK_SIZE) * 2;
        self.data[idx] = density;
        self.data[idx + 1] = material;
    }
    
    /// Get density at local coordinates
    pub fn get_density_local(&self, x: usize, y: usize, z: usize) -> Density {
        let idx = (x + y * crate::chunk::CHUNK_SIZE + z * crate::chunk::CHUNK_SIZE * crate::chunk::CHUNK_SIZE) * 2;
        self.data[idx]
    }
    
    /// Get material at local coordinates
    pub fn get_material_local(&self, x: usize, y: usize, z: usize) -> MaterialId {
        let idx = (x + y * crate::chunk::CHUNK_SIZE + z * crate::chunk::CHUNK_SIZE * crate::chunk::CHUNK_SIZE) * 2;
        self.data[idx + 1]
    }
    
    /// Convert world pos to local coordinates
    fn world_to_local(&self, pos: IVec3) -> Option<(usize, usize, usize)> {
        let local = pos - self.chunk_pos * crate::chunk::CHUNK_SIZE as i32;
        if local.x >= 0 && local.x < crate::chunk::CHUNK_SIZE as i32
            && local.y >= 0 && local.y < crate::chunk::CHUNK_SIZE as i32
            && local.z >= 0 && local.z < crate::chunk::CHUNK_SIZE as i32 {
            Some((local.x as usize, local.y as usize, local.z as usize))
        } else {
            None
        }
    }
    
    /// Set the threshold for solid determination
    pub fn set_solid_threshold(&mut self, threshold: Density) {
        self.solid_threshold = threshold;
    }
}

impl VoxelSchema for DensitySchema {
    fn is_solid(&self, pos: IVec3) -> bool {
        if let Some((x, y, z)) = self.world_to_local(pos) {
            self.get_density_local(x, y, z) >= self.solid_threshold
        } else {
            false
        }
    }
    
    fn material_at(&self, pos: IVec3) -> MaterialId {
        if let Some((x, y, z)) = self.world_to_local(pos) {
            self.get_material_local(x, y, z)
        } else {
            MAT_AIR
        }
    }
    
    fn surface_sign(&self, pos: IVec3) -> f32 {
        if let Some((x, y, z)) = self.world_to_local(pos) {
            let density = self.get_density_local(x, y, z);
            // Map density [0..255] to sign [-1.0..+1.0]
            // 0 = -1.0 (empty)
            // 128 = 0.0 (surface)
            // 255 = +1.0 (solid)
            (density as f32 - 128.0) / 128.0
        } else {
            -1.0
        }
    }
    
    fn schema_name(&self) -> &str {
        "DensitySchema"
    }
}
