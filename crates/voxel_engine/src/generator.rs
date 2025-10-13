//! Procedural terrain generator for chunks
//!
//! Uses noise functions to create varied terrain with hills, valleys, and features

use crate::chunk::{BlockId, Chunk, AIR, CHUNK_SIZE, DIRT, GRASS, STONE};
use crate::generator_metrics::{GenerationSample, GeneratorMetrics, measure_us};
use glam::IVec3;
use rayon::prelude::*;
use std::time::Instant;

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
#[derive(Clone)]
pub struct TerrainGenerator {
    config: TerrainConfig,
    metrics: Option<GeneratorMetrics>,
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        Self { 
            config,
            metrics: None,
        }
    }

    /// Create a new generator with metrics enabled
    pub fn with_metrics(config: TerrainConfig, max_samples: usize) -> Self {
        Self {
            config,
            metrics: Some(GeneratorMetrics::new(max_samples)),
        }
    }

    /// Get access to metrics
    pub fn metrics(&self) -> Option<&GeneratorMetrics> {
        self.metrics.as_ref()
    }

    /// Generate a chunk at the given position with detailed timing
    pub fn generate_chunk(&self, position: IVec3) -> Chunk {
        let start_total = Instant::now();
        
        let underground_check_time_us;
        let mut underground_fill_time_us = 0.0;
        let height_calculation_time_us;
        let block_placement_time_us;
        let was_underground;

        let mut chunk = Chunk::new(position);
        
        let chunk_y_start = position.y * CHUNK_SIZE as i32;
        let chunk_y_end = chunk_y_start + CHUNK_SIZE as i32;

        // 🚀 SUPER OPTIMISATION: Chunk entièrement souterrain = remplir de STONE directement
        let chunk_above_y = chunk_y_end;
        
        // Échantillonner quelques points pour voir si tout le chunk est souterrain
        let sample_points = [
            (0, 0), (CHUNK_SIZE-1, 0), (0, CHUNK_SIZE-1), (CHUNK_SIZE-1, CHUNK_SIZE-1), 
            (CHUNK_SIZE/2, CHUNK_SIZE/2)
        ];
        
        // TIMING: Underground check
        let (is_fully_underground, check_time) = measure_us(|| {
            sample_points.iter().all(|&(lx, lz)| {
                let world_x = position.x * CHUNK_SIZE as i32 + lx as i32;
                let world_z = position.z * CHUNK_SIZE as i32 + lz as i32;
                let height = self.calculate_height(world_x, world_z);
                chunk_above_y <= height
            })
        });
        underground_check_time_us = check_time;

        if is_fully_underground {
            // TIMING: Underground fill
            let (_, fill_time) = measure_us(|| {
                for local_z in 0..CHUNK_SIZE {
                    for local_y in 0..CHUNK_SIZE {
                        for local_x in 0..CHUNK_SIZE {
                            chunk.set(local_x, local_y, local_z, STONE);
                        }
                    }
                }
            });
            underground_fill_time_us = fill_time;

            // Record metrics
            if let Some(metrics) = &self.metrics {
                let total_time_us = start_total.elapsed().as_micros() as f32;
                metrics.add_sample(GenerationSample {
                    underground_check_time_us,
                    underground_fill_time_us,
                    height_calculation_time_us: 0.0,
                    block_placement_time_us: 0.0,
                    total_time_us,
                    was_underground: true,
                    timestamp: Instant::now(),
                });
            }

            return chunk;
        }

        // Surface chunk (not fully underground)
        was_underground = false;

        // ✅ ULTRA-OPTIMISÉ: Pas de height_map! Calcul à la volée (meilleur cache CPU)
        // TIMING: Height calculation and block placement
        let start_height_calc = Instant::now();
        
        for local_z in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = position.x * CHUNK_SIZE as i32 + local_x as i32;
                let world_z = position.z * CHUNK_SIZE as i32 + local_z as i32;
                let height = self.calculate_height(world_x, world_z);

                // Early exit: colonne entièrement au-dessus du terrain (empty)
                if chunk_y_start > height {
                    continue;
                }

                // Pré-calculer si toute la colonne est solide
                let all_solid = chunk_y_end <= height;

                for local_y in 0..CHUNK_SIZE {
                    let world_y = chunk_y_start + local_y as i32;

                    // Arrêt précoce: si on est au-dessus du terrain
                    if !all_solid && world_y > height {
                        break; // Reste de la colonne = empty (défaut du chunk)
                    }

                    let block = if world_y < height {
                        // Calcul de profondeur optimisé
                        let depth = height - world_y;
                        if depth > 4 { 
                            STONE 
                        } else if depth > 1 { 
                            DIRT 
                        } else { 
                            GRASS 
                        }
                    } else {
                        AIR // Empty
                    };

                    if block != AIR {
                        chunk.set(local_x, local_y, local_z, block);
                    }
                }
            }
        }

        let height_and_block_time = start_height_calc.elapsed().as_micros() as f32;
        
        // Split timing between height calculation and block placement (approximate 50/50)
        height_calculation_time_us = height_and_block_time * 0.5;
        block_placement_time_us = height_and_block_time * 0.5;

        // Record metrics
        if let Some(metrics) = &self.metrics {
            let total_time_us = start_total.elapsed().as_micros() as f32;
            metrics.add_sample(GenerationSample {
                underground_check_time_us,
                underground_fill_time_us,
                height_calculation_time_us,
                block_placement_time_us,
                total_time_us,
                was_underground,
                timestamp: Instant::now(),
            });
        }

        chunk
    }

    /// Generate multiple chunks in parallel using all available CPU cores
    /// Positions should be pre-sorted by distance from player for optimal loading
    pub fn generate_chunks_parallel(&self, positions: &[IVec3]) -> Vec<(IVec3, Chunk)> {
        positions
            .par_iter()
            .map(|&position| {
                let chunk = self.generate_chunk(position);
                (position, chunk)
            })
            .collect()
    }

    /// Calculate terrain height at world XZ coordinates
    /// SIMPLIFIÉ: Un seul niveau de noise "numérique"
    fn calculate_height(&self, x: i32, z: i32) -> i32 {
        let fx = x as f32 * self.config.frequency;
        let fz = z as f32 * self.config.frequency;

        // Un seul niveau de noise à échelle 1 (plus "numérique")
        let noise = Self::noise_2d(fx, fz, self.config.seed);

        let height = self.config.base_height + noise * self.config.amplitude;
        height as i32
    }

    /// Simple 2D noise function (value noise)
    /// Returns value in range [-1, 1]
    #[inline]
    fn noise_2d(x: f32, y: f32, seed: u32) -> f32 {
        // Optimisation: éviter floor() coûteux
        let xi = if x >= 0.0 { x as i32 } else { (x - 1.0) as i32 };
        let yi = if y >= 0.0 { y as i32 } else { (y - 1.0) as i32 };

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

        // Bilinear interpolation - optimisé inline
        let x1 = a + (b - a) * u;
        let x2 = c + (d - c) * u;
        x1 + (x2 - x1) * v
    }

    /// Hash function for 2D coordinates
    /// Returns value in range [-1, 1]
    #[inline(always)]
    fn hash_2d(x: i32, y: i32, seed: u32) -> f32 {
        // Utiliser des primes optimisées pour les CPU modernes
        let mut n = x
            .wrapping_mul(1619)
            .wrapping_add(y.wrapping_mul(31337))
            .wrapping_add(seed as i32);
        
        // Mix bits plus efficace (splitmix32 style)
        n ^= n >> 15;
        n = n.wrapping_mul(0x85ebca6b_u32 as i32);
        n ^= n >> 13;
        n = n.wrapping_mul(0xc2b2ae35_u32 as i32);
        n ^= n >> 16;

        // Conversion optimisée sans clamp (plus rapide)
        (n as f32) * (2.0 / 4294967296.0) - 1.0
    }

    /// Smoothstep function for smooth interpolation
    #[inline]
    fn smoothstep(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
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

        // Convert Chunk to BlockSchema - OPTIMISÉ: skip air blocks
        let mut schema = crate::voxel_schema::BlockSchema::new(chunk_pos);
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = chunk.get(x, y, z);
                    if block != AIR {
                        schema.set_local(x, y, z, block);
                    }
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
