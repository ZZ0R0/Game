//! Texture atlas system for voxel blocks
//! Maps BlockId → UV coordinates in a texture atlas

use crate::chunk::BlockId;
use std::collections::HashMap;

/// Rectangle in a texture atlas (normalized 0..1 coordinates)
#[derive(Debug, Clone, Copy)]
pub struct AtlasRect {
    pub u: f32,      // Left U coordinate
    pub v: f32,      // Top V coordinate
    pub w: f32,      // Width in U
    pub h: f32,      // Height in V
}

impl AtlasRect {
    pub fn new(u: f32, v: f32, w: f32, h: f32) -> Self {
        Self { u, v, w, h }
    }
    
    /// Get UV coordinates for a quad's 4 corners
    /// Returns: [(u0,v0), (u1,v0), (u1,v1), (u0,v1)]
    pub fn get_uvs(&self) -> [[f32; 2]; 4] {
        let u0 = self.u;
        let v0 = self.v;
        let u1 = self.u + self.w;
        let v1 = self.v + self.h;
        
        [
            [u0, v0],  // Bottom-left
            [u1, v0],  // Bottom-right
            [u1, v1],  // Top-right
            [u0, v1],  // Top-left
        ]
    }
}

/// Face direction for blocks that have different textures per face
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceDir {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

/// Texture atlas mapping BlockId → texture coordinates
pub struct TextureAtlas {
    /// Map: (BlockId, FaceDir) → AtlasRect
    mapping: HashMap<(BlockId, FaceDir), AtlasRect>,
    
    /// Fallback texture for unmapped blocks
    fallback: AtlasRect,
}

impl TextureAtlas {
    /// Create a new atlas with a 16x16 grid (256 tiles)
    pub fn new_16x16() -> Self {
        let mut atlas = Self {
            mapping: HashMap::new(),
            fallback: AtlasRect::new(0.0, 0.0, 1.0 / 16.0, 1.0 / 16.0),
        };
        
        // Grid size: 16x16 tiles
        let tile_size = 1.0 / 16.0;
        
        // Define block textures (example layout)
        // AIR (0) - no texture needed
        
        // STONE - uniform texture at (1,0)
        atlas.add_uniform_block(crate::chunk::STONE, 1, 0, tile_size);
        
        // DIRT - uniform texture at (2,0)
        atlas.add_uniform_block(crate::chunk::DIRT, 2, 0, tile_size);
        
        // GRASS - custom top/bottom/side
        atlas.add_custom_block(
            crate::chunk::GRASS,
            (0, 0), // Top = grass
            (2, 0), // Bottom = dirt
            (3, 0), // Side = grass_side
            tile_size,
        );
        
        // WOOD - uniform at (4,0)
        atlas.add_uniform_block(crate::chunk::WOOD, 4, 0, tile_size);
        
        // LEAVES - uniform at (5,0)
        atlas.add_uniform_block(crate::chunk::LEAVES, 5, 0, tile_size);
        
        // WATER (6) - semi-transparent at (6,0)
        atlas.add_uniform_block(6, 6, 0, tile_size);
        
        // GLASS (7) - transparent at (7,0)
        atlas.add_uniform_block(7, 7, 0, tile_size);
        
        atlas
    }
    
    /// Add a block with the same texture on all faces
    fn add_uniform_block(&mut self, block_id: BlockId, tile_x: usize, tile_y: usize, tile_size: f32) {
        let rect = AtlasRect::new(
            tile_x as f32 * tile_size,
            tile_y as f32 * tile_size,
            tile_size,
            tile_size,
        );
        
        for face in [FaceDir::Top, FaceDir::Bottom, FaceDir::North, 
                     FaceDir::South, FaceDir::East, FaceDir::West] {
            self.mapping.insert((block_id, face), rect);
        }
    }
    
    /// Add a block with different textures for top/bottom/sides
    fn add_custom_block(
        &mut self,
        block_id: BlockId,
        top: (usize, usize),
        bottom: (usize, usize),
        side: (usize, usize),
        tile_size: f32,
    ) {
        let top_rect = AtlasRect::new(
            top.0 as f32 * tile_size, top.1 as f32 * tile_size,
            tile_size, tile_size
        );
        let bottom_rect = AtlasRect::new(
            bottom.0 as f32 * tile_size, bottom.1 as f32 * tile_size,
            tile_size, tile_size
        );
        let side_rect = AtlasRect::new(
            side.0 as f32 * tile_size, side.1 as f32 * tile_size,
            tile_size, tile_size
        );
        
        self.mapping.insert((block_id, FaceDir::Top), top_rect);
        self.mapping.insert((block_id, FaceDir::Bottom), bottom_rect);
        self.mapping.insert((block_id, FaceDir::North), side_rect);
        self.mapping.insert((block_id, FaceDir::South), side_rect);
        self.mapping.insert((block_id, FaceDir::East), side_rect);
        self.mapping.insert((block_id, FaceDir::West), side_rect);
    }
    
    /// Get texture coordinates for a block face
    pub fn get_uvs(&self, block_id: BlockId, face: FaceDir) -> [[f32; 2]; 4] {
        self.mapping
            .get(&(block_id, face))
            .unwrap_or(&self.fallback)
            .get_uvs()
    }
    
    /// Get atlas rect for a block face
    pub fn get_rect(&self, block_id: BlockId, face: FaceDir) -> AtlasRect {
        self.mapping
            .get(&(block_id, face))
            .copied()
            .unwrap_or(self.fallback)
    }
}

impl Default for TextureAtlas {
    fn default() -> Self {
        Self::new_16x16()
    }
}
