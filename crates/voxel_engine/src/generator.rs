//! Procedural terrain generator for chunks
//! 
//! Uses noise functions to create varied terrain with hills, valleys, and features

use glam::IVec3;
use rayon::prelude::*;
use crate::chunk::{Chunk, CHUNK_SIZE, BlockId, STONE, DIRT, GRASS, AIR, WATER};

/// Terrain generator configuration
#[derive(Debug, Clone)]
pub struct TerrainConfig {
    /// Base height level
    pub base_height: f32,
    
    /// Height variation amplitude
    pub amplitude: f32,
    
    /// Terrain frequency (smaller = larger features)
    pub frequency: f32,
    
    /// Water level
    pub water_level: i32,
    
    /// Seed for randomness
    pub seed: u32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            base_height: 16.0,
            amplitude: 12.0,
            frequency: 0.03,
            water_level: 10,
            seed: 12345,
        }
    }
}

/// Terrain generator
pub struct TerrainGenerator {
    config: TerrainConfig,
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        Self { config }
    }
    
    /// Generate a chunk at the given position
    /// Uses rayon to parallelize across all voxel positions
    pub fn generate_chunk(&self, position: IVec3) -> Chunk {
        let mut chunk = Chunk::new(position);
        
        // Generate all voxel positions in parallel
        // We'll collect (x, y, z, block) tuples and then apply them
        let blocks: Vec<_> = (0..CHUNK_SIZE)
            .into_par_iter()
            .flat_map(|local_z| {
                (0..CHUNK_SIZE).into_par_iter().flat_map(move |local_y| {
                    (0..CHUNK_SIZE).into_par_iter().map(move |local_x| {
                        // Convert to world coordinates
                        let world_x = position.x * CHUNK_SIZE as i32 + local_x as i32;
                        let world_y = position.y * CHUNK_SIZE as i32 + local_y as i32;
                        let world_z = position.z * CHUNK_SIZE as i32 + local_z as i32;
                        
                        // Calculate height at this XZ position
                        let height = self.calculate_height(world_x, world_z);
                        
                        // Determine block type
                        let block = if world_y < height {
                            // Underground
                            if world_y < height - 4 {
                                STONE
                            } else if world_y < height - 1 {
                                DIRT
                            } else {
                                GRASS
                            }
                        } else if world_y <= self.config.water_level {
                            // Water
                            WATER
                        } else {
                            AIR
                        };
                        
                        (local_x, local_y, local_z, block)
                    })
                })
            })
            .collect();
        
        // Apply results to chunk (sequential, but fast)
        for (x, y, z, block) in blocks {
            chunk.set(x, y, z, block);
        }
        
        chunk
    }
    
    /// Calculate terrain height at world XZ coordinates
    fn calculate_height(&self, x: i32, z: i32) -> i32 {
        let fx = x as f32 * self.config.frequency;
        let fz = z as f32 * self.config.frequency;
        
        // Multi-octave noise for more interesting terrain
        let noise1 = Self::noise_2d(fx, fz, self.config.seed);
        let noise2 = Self::noise_2d(fx * 2.0, fz * 2.0, self.config.seed + 1000);
        let noise3 = Self::noise_2d(fx * 4.0, fz * 4.0, self.config.seed + 2000);
        
        let combined = noise1 * 0.6 + noise2 * 0.25 + noise3 * 0.15;
        
        let height = self.config.base_height + combined * self.config.amplitude;
        height as i32
    }
    
    /// Simple 2D noise function (value noise)
    /// Returns value in range [-1, 1]
    fn noise_2d(x: f32, y: f32, seed: u32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        
        let xf = x - xi as f32;
        let yf = y - yi as f32;
        
        // Smooth interpolation
        let u = Self::smoothstep(xf);
        let v = Self::smoothstep(yf);
        
        // Sample corners
        let a = Self::hash_2d(xi, yi, seed);
        let b = Self::hash_2d(xi + 1, yi, seed);
        let c = Self::hash_2d(xi, yi + 1, seed);
        let d = Self::hash_2d(xi + 1, yi + 1, seed);
        
        // Bilinear interpolation
        let x1 = Self::lerp(a, b, u);
        let x2 = Self::lerp(c, d, u);
        Self::lerp(x1, x2, v)
    }
    
    /// Hash function for 2D coordinates
    /// Returns value in range [-1, 1]
    fn hash_2d(x: i32, y: i32, seed: u32) -> f32 {
        let mut n = x.wrapping_mul(374761393)
            .wrapping_add(y.wrapping_mul(668265263))
            .wrapping_add(seed as i32);
        n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        n = n ^ (n >> 16);
        
        // Convert to [-1, 1]
        (n as f32 / i32::MAX as f32).clamp(-1.0, 1.0)
    }
    
    /// Smoothstep function for smooth interpolation
    fn smoothstep(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
    
    /// Linear interpolation
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self::new(TerrainConfig::default())
    }
}

/// Implementation of ProceduralProvider for TerrainGenerator
/// This allows TerrainGenerator to be used with CelestialVolume
impl crate::volume::ProceduralProvider for TerrainGenerator {
    fn generate_chunk(&self, chunk_pos: IVec3) -> Box<dyn crate::voxel_schema::VoxelSchema> {
        let chunk = TerrainGenerator::generate_chunk(self, chunk_pos);
        
        // Convert Chunk to BlockSchema
        let mut schema = crate::voxel_schema::BlockSchema::new(chunk_pos);
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = chunk.get(x, y, z);
                    schema.set_local(x, y, z, block);
                }
            }
        }
        
        Box::new(schema)
    }
    
    fn provider_name(&self) -> &str {
        "TerrainGenerator"
    }
}

/// Simple biome system (future expansion)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    Mountains,
    Desert,
    Ocean,
}

impl Biome {
    /// Get block type for surface at this biome
    pub fn surface_block(&self) -> BlockId {
        match self {
            Biome::Plains | Biome::Forest => GRASS,
            Biome::Mountains => STONE,
            Biome::Desert => DIRT, // Could be sand
            Biome::Ocean => DIRT,
        }
    }
}
