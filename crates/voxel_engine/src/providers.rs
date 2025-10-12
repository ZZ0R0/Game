//! Voxel data providers - pluggable sources for voxel data
//!
//! Milestone 2: Provider system for different data sources:
//! - GridStoreProvider: In-memory chunk storage with palette
//! - PlanetProvider: Procedural planet generation
//! - AsteroidProvider: Procedural asteroid generation  
//! - DeltaStore: Sparse edit overlay

use crate::chunk::{BlockId, CHUNK_SIZE, CHUNK_VOLUME};
use crate::voxel_schema::{Density, MaterialId};
use glam::{IVec3, Vec3};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Errors that can occur during provider operations
#[derive(Debug, Clone)]
pub enum ProviderError {
    /// Region is outside valid bounds
    OutOfBounds,

    /// Provider is read-only
    ReadOnly,

    /// Invalid LOD level
    InvalidLOD,

    /// IO error during save/load
    IoError(String),

    /// Generic error
    Other(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::OutOfBounds => write!(f, "Region out of bounds"),
            ProviderError::ReadOnly => write!(f, "Provider is read-only"),
            ProviderError::InvalidLOD => write!(f, "Invalid LOD level"),
            ProviderError::IoError(s) => write!(f, "IO error: {}", s),
            ProviderError::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Voxel value (can be block-based or density-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelValue {
    /// Block-based voxel (BlockId)
    Block(BlockId),

    /// Density-based voxel (Density + MaterialId)
    Density(Density, MaterialId),
}

impl VoxelValue {
    /// Check if this voxel is solid
    pub fn is_solid(&self) -> bool {
        match self {
            VoxelValue::Block(id) => crate::chunk::is_solid(*id),
            VoxelValue::Density(d, _) => *d >= 128,
        }
    }
}

/// Voxel data container
#[derive(Debug, Clone)]
pub struct VoxelData {
    /// Dimensions (in voxels)
    pub size: IVec3,

    /// Voxel values (size.x * size.y * size.z)
    pub values: Vec<VoxelValue>,
}

impl VoxelData {
    /// Create new voxel data
    pub fn new(size: IVec3) -> Self {
        let volume = (size.x * size.y * size.z) as usize;
        Self {
            size,
            values: vec![VoxelValue::Block(crate::chunk::AIR); volume],
        }
    }

    /// Get voxel at local coordinates
    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<VoxelValue> {
        if x < 0 || y < 0 || z < 0 || x >= self.size.x || y >= self.size.y || z >= self.size.z {
            return None;
        }
        let idx = (x + y * self.size.x + z * self.size.x * self.size.y) as usize;
        self.values.get(idx).copied()
    }

    /// Set voxel at local coordinates
    pub fn set(&mut self, x: i32, y: i32, z: i32, value: VoxelValue) {
        if x < 0 || y < 0 || z < 0 || x >= self.size.x || y >= self.size.y || z >= self.size.z {
            return;
        }
        let idx = (x + y * self.size.x + z * self.size.x * self.size.y) as usize;
        if idx < self.values.len() {
            self.values[idx] = value;
        }
    }
}

/// Brush pattern for writing multiple voxels
#[derive(Debug, Clone)]
pub struct Brush {
    /// Brush shape
    pub shape: BrushShape,

    /// Brush size
    pub size: f32,

    /// Value to write
    pub value: VoxelValue,
}

/// Brush shapes
#[derive(Debug, Clone, Copy)]
pub enum BrushShape {
    Sphere,
    Cube,
    Cylinder,
}

impl Brush {
    /// Check if a position is inside this brush
    pub fn contains(&self, center: Vec3, pos: Vec3) -> bool {
        let dist = pos.distance(center);
        match self.shape {
            BrushShape::Sphere => dist <= self.size,
            BrushShape::Cube => {
                let diff = (pos - center).abs();
                diff.x <= self.size && diff.y <= self.size && diff.z <= self.size
            }
            BrushShape::Cylinder => {
                let xz_dist = Vec3::new(pos.x - center.x, 0.0, pos.z - center.z).length();
                let y_diff = (pos.y - center.y).abs();
                xz_dist <= self.size && y_diff <= self.size
            }
        }
    }
}

/// Core trait for all voxel providers
pub trait VoxelProvider: Send + Sync {
    /// Read voxel data in a range with LOD sampling
    ///
    /// - `min`, `max`: World coordinates (inclusive)
    /// - `lod`: Level of detail (0 = full resolution, 1+ = lower resolution)
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) -> Result<VoxelData, ProviderError>;

    /// Write a single voxel
    fn write_voxel(&mut self, pos: IVec3, value: VoxelValue) -> Result<(), ProviderError>;

    /// Write a brush pattern
    fn write_brush(&mut self, center: IVec3, brush: &Brush) -> Result<(), ProviderError> {
        // Default implementation: write individual voxels
        let size_i = brush.size.ceil() as i32;
        for x in -size_i..=size_i {
            for y in -size_i..=size_i {
                for z in -size_i..=size_i {
                    let pos = center + IVec3::new(x, y, z);
                    let pos_f = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
                    let center_f = Vec3::new(center.x as f32, center.y as f32, center.z as f32);

                    if brush.contains(center_f, pos_f) {
                        self.write_voxel(pos, brush.value)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Get provider name for debugging
    fn provider_name(&self) -> &str;

    /// Check if provider supports writes
    fn is_writable(&self) -> bool;
}

// ============================================================================
// GridStoreProvider - In-memory chunk storage
// ============================================================================

/// Configuration for grid store
#[derive(Debug, Clone)]
pub struct GridStoreConfig {
    /// Enable palette compression
    pub use_palette: bool,

    /// Max palette entries before compaction
    pub max_palette_size: usize,

    /// Enable dirty tracking
    pub track_dirty: bool,
}

impl Default for GridStoreConfig {
    fn default() -> Self {
        Self {
            use_palette: true,
            max_palette_size: 256,
            track_dirty: true,
        }
    }
}

/// Chunk data with palette
#[derive(Debug, Clone)]
pub struct ChunkData {
    /// Block data
    pub blocks: Box<[BlockId; CHUNK_VOLUME]>,

    /// Palette (for compression)
    pub palette: Vec<BlockId>,

    /// Version (for cache invalidation)
    pub version: u32,
}

impl ChunkData {
    /// Create new empty chunk
    pub fn new() -> Self {
        Self {
            blocks: Box::new([crate::chunk::AIR; CHUNK_VOLUME]),
            palette: vec![crate::chunk::AIR],
            version: 0,
        }
    }

    /// Create filled chunk
    pub fn filled(block: BlockId) -> Self {
        Self {
            blocks: Box::new([block; CHUNK_VOLUME]),
            palette: vec![block],
            version: 0,
        }
    }
}

/// Grid-based chunk storage provider
pub struct GridStoreProvider {
    /// Chunks (chunk_pos -> data)
    chunks: Arc<RwLock<HashMap<IVec3, ChunkData>>>,

    /// Dirty regions
    dirty_regions: Arc<RwLock<HashSet<IVec3>>>,

    /// Configuration
    config: GridStoreConfig,
}

impl GridStoreProvider {
    /// Create new grid store
    pub fn new(config: GridStoreConfig) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            dirty_regions: Arc::new(RwLock::new(HashSet::new())),
            config,
        }
    }

    /// Insert a chunk
    pub fn insert_chunk(&mut self, chunk_pos: IVec3, data: ChunkData) {
        let mut chunks = self.chunks.write().unwrap();
        chunks.insert(chunk_pos, data);

        if self.config.track_dirty {
            let mut dirty = self.dirty_regions.write().unwrap();
            dirty.insert(chunk_pos);
        }
    }

    /// Get chunk (read-only)
    pub fn get_chunk(&self, chunk_pos: IVec3) -> Option<ChunkData> {
        let chunks = self.chunks.read().unwrap();
        chunks.get(&chunk_pos).cloned()
    }

    /// Take dirty chunks
    pub fn take_dirty_chunks(&mut self) -> Vec<IVec3> {
        let mut dirty = self.dirty_regions.write().unwrap();
        dirty.drain().collect()
    }

    /// Compact palettes
    pub fn compact_palettes(&mut self) {
        let mut chunks = self.chunks.write().unwrap();
        for data in chunks.values_mut() {
            if data.palette.len() > self.config.max_palette_size {
                // TODO: Implement palette compaction
                data.version += 1;
            }
        }
    }

    /// Get chunk count
    pub fn chunk_count(&self) -> usize {
        let chunks = self.chunks.read().unwrap();
        chunks.len()
    }
}

impl VoxelProvider for GridStoreProvider {
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) -> Result<VoxelData, ProviderError> {
        if lod > 0 {
            return Err(ProviderError::InvalidLOD);
        }

        let size = max - min + IVec3::ONE;
        let mut data = VoxelData::new(size);

        let chunks = self.chunks.read().unwrap();

        // Iterate through all voxels in range
        for z in 0..size.z {
            for y in 0..size.y {
                for x in 0..size.x {
                    let world_pos = min + IVec3::new(x, y, z);
                    let chunk_pos = IVec3::new(
                        world_pos.x.div_euclid(CHUNK_SIZE as i32),
                        world_pos.y.div_euclid(CHUNK_SIZE as i32),
                        world_pos.z.div_euclid(CHUNK_SIZE as i32),
                    );

                    if let Some(chunk_data) = chunks.get(&chunk_pos) {
                        let local = world_pos - chunk_pos * CHUNK_SIZE as i32;
                        let idx = (local.x
                            + local.y * CHUNK_SIZE as i32
                            + local.z * CHUNK_SIZE as i32 * CHUNK_SIZE as i32)
                            as usize;

                        if idx < CHUNK_VOLUME {
                            let block = chunk_data.blocks[idx];
                            data.set(x, y, z, VoxelValue::Block(block));
                        }
                    }
                }
            }
        }

        Ok(data)
    }

    fn write_voxel(&mut self, pos: IVec3, value: VoxelValue) -> Result<(), ProviderError> {
        let chunk_pos = IVec3::new(
            pos.x.div_euclid(CHUNK_SIZE as i32),
            pos.y.div_euclid(CHUNK_SIZE as i32),
            pos.z.div_euclid(CHUNK_SIZE as i32),
        );

        let local = pos - chunk_pos * CHUNK_SIZE as i32;
        let idx = (local.x
            + local.y * CHUNK_SIZE as i32
            + local.z * CHUNK_SIZE as i32 * CHUNK_SIZE as i32) as usize;

        if idx >= CHUNK_VOLUME {
            return Err(ProviderError::OutOfBounds);
        }

        let block = match value {
            VoxelValue::Block(b) => b,
            VoxelValue::Density(_, _) => {
                return Err(ProviderError::Other(
                    "GridStore only supports blocks".to_string(),
                ))
            }
        };

        let mut chunks = self.chunks.write().unwrap();
        let chunk_data = chunks.entry(chunk_pos).or_insert_with(ChunkData::new);
        chunk_data.blocks[idx] = block;
        chunk_data.version += 1;

        if self.config.track_dirty {
            let mut dirty = self.dirty_regions.write().unwrap();
            dirty.insert(chunk_pos);
        }

        Ok(())
    }

    fn provider_name(&self) -> &str {
        "GridStoreProvider"
    }

    fn is_writable(&self) -> bool {
        true
    }
}

use std::collections::HashSet;

// ============================================================================
// PlanetProvider - Procedural planet generation
// ============================================================================

/// Biome type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    Ice,
    Tundra,
    Temperate,
    Desert,
    Tropical,
    Ocean,
}

impl BiomeType {
    /// Get surface material for this biome
    pub fn surface_material(&self) -> MaterialId {
        match self {
            BiomeType::Ice => crate::voxel_schema::MAT_WATER, // Ice
            BiomeType::Tundra => crate::voxel_schema::MAT_STONE,
            BiomeType::Temperate => crate::voxel_schema::MAT_GRASS,
            BiomeType::Desert => crate::voxel_schema::MAT_DIRT, // Sand
            BiomeType::Tropical => crate::voxel_schema::MAT_GRASS,
            BiomeType::Ocean => crate::voxel_schema::MAT_WATER,
        }
    }
}

/// Noise layer configuration
#[derive(Debug, Clone)]
pub struct NoiseLayer {
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub persistence: f32,
}

impl Default for NoiseLayer {
    fn default() -> Self {
        Self {
            frequency: 0.01,
            amplitude: 50.0,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
}

/// Biome band (latitude-based)
#[derive(Debug, Clone)]
pub struct BiomeBand {
    /// Latitude range (-1.0 to 1.0, where 0 is equator)
    pub lat_min: f32,
    pub lat_max: f32,

    /// Biome type
    pub biome: BiomeType,
}

/// Planet configuration
#[derive(Debug, Clone)]
pub struct PlanetConfig {
    /// Random seed
    pub seed: u64,

    /// Planet radius (in blocks)
    pub radius: f32,

    /// Center position
    pub center: Vec3,

    /// Noise layers (stacked)
    pub noise_stack: Vec<NoiseLayer>,

    /// Biome bands
    pub biome_bands: Vec<BiomeBand>,

    /// Sea level (relative to radius)
    pub sea_level: f32,
}

impl Default for PlanetConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            radius: 1000.0,
            center: Vec3::ZERO,
            noise_stack: vec![NoiseLayer::default()],
            biome_bands: vec![
                BiomeBand {
                    lat_min: -1.0,
                    lat_max: -0.6,
                    biome: BiomeType::Ice,
                },
                BiomeBand {
                    lat_min: -0.6,
                    lat_max: -0.2,
                    biome: BiomeType::Tundra,
                },
                BiomeBand {
                    lat_min: -0.2,
                    lat_max: 0.2,
                    biome: BiomeType::Temperate,
                },
                BiomeBand {
                    lat_min: 0.2,
                    lat_max: 0.6,
                    biome: BiomeType::Tropical,
                },
                BiomeBand {
                    lat_min: 0.6,
                    lat_max: 1.0,
                    biome: BiomeType::Ice,
                },
            ],
            sea_level: 950.0,
        }
    }
}

/// Planet provider - procedural sphere generation
pub struct PlanetProvider {
    config: PlanetConfig,
}

impl PlanetProvider {
    /// Create new planet provider
    pub fn new(config: PlanetConfig) -> Self {
        Self { config }
    }

    /// Calculate signed distance to surface
    /// Negative = inside, Positive = outside
    pub fn signed_distance(&self, pos: Vec3) -> f32 {
        let relative = pos - self.config.center;
        let distance_from_center = relative.length();

        // Basic sphere
        let base_distance = distance_from_center - self.config.radius;

        // Add noise
        let noise = self.sample_noise(pos);

        base_distance - noise
    }

    /// Sample noise at position
    fn sample_noise(&self, pos: Vec3) -> f32 {
        let mut total = 0.0;

        for layer in &self.config.noise_stack {
            let mut amplitude = layer.amplitude;
            let mut frequency = layer.frequency;

            for _ in 0..layer.octaves {
                let sample = Self::noise_3d(
                    pos.x * frequency,
                    pos.y * frequency,
                    pos.z * frequency,
                    self.config.seed,
                );

                total += sample * amplitude;

                amplitude *= layer.persistence;
                frequency *= layer.lacunarity;
            }
        }

        total
    }

    /// Sample material at position
    pub fn sample_material(&self, pos: Vec3, _distance: f32) -> MaterialId {
        // Determine biome based on latitude
        let relative = pos - self.config.center;
        let latitude = (relative.y / relative.length()).clamp(-1.0, 1.0);

        for band in &self.config.biome_bands {
            if latitude >= band.lat_min && latitude < band.lat_max {
                return band.biome.surface_material();
            }
        }

        crate::voxel_schema::MAT_STONE
    }

    /// 3D noise function
    fn noise_3d(x: f32, y: f32, z: f32, seed: u64) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let zi = z.floor() as i32;

        let xf = x - xi as f32;
        let yf = y - yi as f32;
        let zf = z - zi as f32;

        // Smooth interpolation
        let u = Self::smoothstep(xf);
        let v = Self::smoothstep(yf);
        let w = Self::smoothstep(zf);

        // Sample corners of cube
        let c000 = Self::hash_3d(xi, yi, zi, seed);
        let c100 = Self::hash_3d(xi + 1, yi, zi, seed);
        let c010 = Self::hash_3d(xi, yi + 1, zi, seed);
        let c110 = Self::hash_3d(xi + 1, yi + 1, zi, seed);
        let c001 = Self::hash_3d(xi, yi, zi + 1, seed);
        let c101 = Self::hash_3d(xi + 1, yi, zi + 1, seed);
        let c011 = Self::hash_3d(xi, yi + 1, zi + 1, seed);
        let c111 = Self::hash_3d(xi + 1, yi + 1, zi + 1, seed);

        // Trilinear interpolation
        let x00 = Self::lerp(c000, c100, u);
        let x10 = Self::lerp(c010, c110, u);
        let x01 = Self::lerp(c001, c101, u);
        let x11 = Self::lerp(c011, c111, u);

        let y0 = Self::lerp(x00, x10, v);
        let y1 = Self::lerp(x01, x11, v);

        Self::lerp(y0, y1, w)
    }

    /// Hash function for 3D coordinates
    fn hash_3d(x: i32, y: i32, z: i32, seed: u64) -> f32 {
        let mut n = x
            .wrapping_mul(374761393)
            .wrapping_add(y.wrapping_mul(668265263))
            .wrapping_add(z.wrapping_mul(2147483647))
            .wrapping_add(seed as i32);
        n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        n = n ^ (n >> 16);

        (n as f32 / i32::MAX as f32).clamp(-1.0, 1.0)
    }

    /// Smoothstep function
    fn smoothstep(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }

    /// Linear interpolation
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

impl VoxelProvider for PlanetProvider {
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) -> Result<VoxelData, ProviderError> {
        let step = 1 << lod; // LOD0 = 1, LOD1 = 2, LOD2 = 4, etc.

        let size = (max - min + IVec3::ONE) / step;
        let mut data = VoxelData::new(size);

        for z in 0..size.z {
            for y in 0..size.y {
                for x in 0..size.x {
                    let world_pos = min + IVec3::new(x, y, z) * step;
                    let pos_f =
                        Vec3::new(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32);

                    let distance = self.signed_distance(pos_f);

                    let value = if distance < 0.0 {
                        // Inside planet
                        let material = self.sample_material(pos_f, distance);
                        let density =
                            ((1.0 - (distance / 10.0).abs()) * 255.0).clamp(0.0, 255.0) as u8;
                        VoxelValue::Density(density, material)
                    } else if pos_f.distance(self.config.center) < self.config.sea_level {
                        // Below sea level but above surface
                        VoxelValue::Density(200, crate::voxel_schema::MAT_WATER)
                    } else {
                        // Air
                        VoxelValue::Block(crate::chunk::AIR)
                    };

                    data.set(x, y, z, value);
                }
            }
        }

        Ok(data)
    }

    fn write_voxel(&mut self, _pos: IVec3, _value: VoxelValue) -> Result<(), ProviderError> {
        Err(ProviderError::ReadOnly)
    }

    fn provider_name(&self) -> &str {
        "PlanetProvider"
    }

    fn is_writable(&self) -> bool {
        false
    }
}

// ============================================================================
// AsteroidProvider - Procedural asteroid generation
// ============================================================================

/// Noise mode for asteroid generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseMode {
    /// Standard Perlin noise
    Standard,

    /// Ridge noise (absolute value for sharp edges)
    Ridge,

    /// Billowy (squared for puffy clouds)
    Billowy,
}

/// Noise parameters
#[derive(Debug, Clone)]
pub struct NoiseParams {
    pub frequency: f32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub persistence: f32,
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            frequency: 0.05,
            octaves: 3,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
}

/// Asteroid configuration
#[derive(Debug, Clone)]
pub struct AsteroidConfig {
    /// Unique seed per asteroid
    pub seed: u64,

    /// Approximate size (radius)
    pub size: f32,

    /// Center position
    pub center: Vec3,

    /// Density threshold (0.0-1.0)
    pub density_threshold: f32,

    /// Noise mode
    pub noise_mode: NoiseMode,

    /// Noise parameters
    pub noise_params: NoiseParams,
}

impl Default for AsteroidConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            size: 20.0,
            center: Vec3::ZERO,
            density_threshold: 0.6,
            noise_mode: NoiseMode::Ridge,
            noise_params: NoiseParams::default(),
        }
    }
}

/// Asteroid provider
pub struct AsteroidProvider {
    config: AsteroidConfig,
}

impl AsteroidProvider {
    /// Create new asteroid provider
    pub fn new(config: AsteroidConfig) -> Self {
        Self { config }
    }

    /// Calculate density at position (0.0 = empty, 1.0 = solid)
    pub fn density_at(&self, pos: Vec3) -> f32 {
        let relative = pos - self.config.center;
        let distance = relative.length();

        // Base sphere falloff
        let base_density = 1.0 - (distance / self.config.size).clamp(0.0, 1.0);

        // Add noise
        let mut noise_value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.config.noise_params.frequency;

        for _ in 0..self.config.noise_params.octaves {
            let sample = PlanetProvider::noise_3d(
                relative.x * frequency,
                relative.y * frequency,
                relative.z * frequency,
                self.config.seed,
            );

            let modified = match self.config.noise_mode {
                NoiseMode::Standard => sample,
                NoiseMode::Ridge => 1.0 - sample.abs() * 2.0,
                NoiseMode::Billowy => sample * sample,
            };

            noise_value += modified * amplitude;

            amplitude *= self.config.noise_params.persistence;
            frequency *= self.config.noise_params.lacunarity;
        }

        // Combine base density with noise
        (base_density + noise_value * 0.3).clamp(0.0, 1.0)
    }

    /// Check if position is solid
    pub fn is_solid(&self, pos: Vec3) -> bool {
        self.density_at(pos) > self.config.density_threshold
    }
}

impl VoxelProvider for AsteroidProvider {
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) -> Result<VoxelData, ProviderError> {
        let step = 1 << lod;

        let size = (max - min + IVec3::ONE) / step;
        let mut data = VoxelData::new(size);

        for z in 0..size.z {
            for y in 0..size.y {
                for x in 0..size.x {
                    let world_pos = min + IVec3::new(x, y, z) * step;
                    let pos_f =
                        Vec3::new(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32);

                    let density = self.density_at(pos_f);

                    let value = if density > self.config.density_threshold {
                        let density_u8 = (density * 255.0) as u8;
                        VoxelValue::Density(density_u8, crate::voxel_schema::MAT_STONE)
                    } else {
                        VoxelValue::Block(crate::chunk::AIR)
                    };

                    data.set(x, y, z, value);
                }
            }
        }

        Ok(data)
    }

    fn write_voxel(&mut self, _pos: IVec3, _value: VoxelValue) -> Result<(), ProviderError> {
        Err(ProviderError::ReadOnly)
    }

    fn provider_name(&self) -> &str {
        "AsteroidProvider"
    }

    fn is_writable(&self) -> bool {
        false
    }
}

// ============================================================================
// DeltaStore - Sparse edit overlay
// ============================================================================

use std::io::{self, Read, Write};
use std::path::Path;

/// Eviction policy for delta store
#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,

    /// Spatial LRU (avoid evicting chunks near player)
    SpatialLRU { player_pos: Vec3, radius: f32 },

    /// Never evict
    Never,
}

/// GC configuration
#[derive(Debug, Clone)]
pub struct GCConfig {
    /// Max delta chunks before GC
    pub max_delta_chunks: usize,

    /// Eviction policy
    pub eviction_policy: EvictionPolicy,

    /// Auto flush to disk
    pub auto_flush: bool,

    /// Flush interval (seconds)
    pub flush_interval: f32,
}

impl Default for GCConfig {
    fn default() -> Self {
        Self {
            max_delta_chunks: 1000,
            eviction_policy: EvictionPolicy::LRU,
            auto_flush: false,
            flush_interval: 60.0,
        }
    }
}

/// Delta chunk (sparse modifications)
#[derive(Debug, Clone)]
pub struct DeltaChunk {
    /// Modifications (local index -> value)
    modifications: HashMap<usize, VoxelValue>,

    /// Last modified time
    last_modified: Instant,

    /// Dirty flag
    dirty: bool,
}

impl DeltaChunk {
    fn new() -> Self {
        Self {
            modifications: HashMap::new(),
            last_modified: Instant::now(),
            dirty: true,
        }
    }

    fn set(&mut self, local_idx: usize, value: VoxelValue) {
        self.modifications.insert(local_idx, value);
        self.last_modified = Instant::now();
        self.dirty = true;
    }
}

/// Delta store statistics
#[derive(Debug, Clone, Default)]
pub struct DeltaStats {
    pub total_deltas: usize,
    pub memory_usage_bytes: usize,
    pub dirty_chunks: usize,
}

/// Delta store - sparse overlay for edits
pub struct DeltaStore {
    /// Sparse delta chunks
    deltas: HashMap<IVec3, DeltaChunk>,

    /// GC configuration
    gc_config: GCConfig,

    /// Last GC time
    last_gc: Instant,

    /// Last flush time
    last_flush: Instant,
}

impl DeltaStore {
    /// Create new delta store
    pub fn new(gc_config: GCConfig) -> Self {
        Self {
            deltas: HashMap::new(),
            gc_config,
            last_gc: Instant::now(),
            last_flush: Instant::now(),
        }
    }

    /// Set voxel
    pub fn set_voxel(&mut self, pos: IVec3, value: VoxelValue) {
        let chunk_pos = IVec3::new(
            pos.x.div_euclid(CHUNK_SIZE as i32),
            pos.y.div_euclid(CHUNK_SIZE as i32),
            pos.z.div_euclid(CHUNK_SIZE as i32),
        );

        let local = pos - chunk_pos * CHUNK_SIZE as i32;
        let local_idx = (local.x
            + local.y * CHUNK_SIZE as i32
            + local.z * CHUNK_SIZE as i32 * CHUNK_SIZE as i32) as usize;

        let delta = self.deltas.entry(chunk_pos).or_insert_with(DeltaChunk::new);
        delta.set(local_idx, value);
    }

    /// Apply deltas to voxel data
    pub fn apply_to(&self, data: &mut VoxelData, min: IVec3, max: IVec3) {
        // Calculate which chunks overlap with this region
        let min_chunk = IVec3::new(
            min.x.div_euclid(CHUNK_SIZE as i32),
            min.y.div_euclid(CHUNK_SIZE as i32),
            min.z.div_euclid(CHUNK_SIZE as i32),
        );
        let max_chunk = IVec3::new(
            max.x.div_euclid(CHUNK_SIZE as i32),
            max.y.div_euclid(CHUNK_SIZE as i32),
            max.z.div_euclid(CHUNK_SIZE as i32),
        );

        for cz in min_chunk.z..=max_chunk.z {
            for cy in min_chunk.y..=max_chunk.y {
                for cx in min_chunk.x..=max_chunk.x {
                    let chunk_pos = IVec3::new(cx, cy, cz);

                    if let Some(delta) = self.deltas.get(&chunk_pos) {
                        // Apply modifications from this chunk
                        for (&local_idx, &value) in &delta.modifications {
                            let local_x = (local_idx % CHUNK_SIZE) as i32;
                            let local_y = ((local_idx / CHUNK_SIZE) % CHUNK_SIZE) as i32;
                            let local_z = (local_idx / (CHUNK_SIZE * CHUNK_SIZE)) as i32;

                            let world_pos = chunk_pos * CHUNK_SIZE as i32
                                + IVec3::new(local_x, local_y, local_z);

                            // Check if this world pos is in the requested range
                            if world_pos.x >= min.x
                                && world_pos.x <= max.x
                                && world_pos.y >= min.y
                                && world_pos.y <= max.y
                                && world_pos.z >= min.z
                                && world_pos.z <= max.z
                            {
                                let data_pos = world_pos - min;
                                data.set(data_pos.x, data_pos.y, data_pos.z, value);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Run garbage collection
    pub fn gc(&mut self) {
        if self.deltas.len() <= self.gc_config.max_delta_chunks {
            return;
        }

        let to_remove = self.deltas.len() - self.gc_config.max_delta_chunks;

        match &self.gc_config.eviction_policy {
            EvictionPolicy::LRU => {
                // Sort by last modified time
                let mut entries: Vec<_> = self
                    .deltas
                    .iter()
                    .map(|(pos, chunk)| (*pos, chunk.last_modified))
                    .collect();
                entries.sort_by_key(|(_, time)| *time);

                // Remove oldest
                for i in 0..to_remove.min(entries.len()) {
                    self.deltas.remove(&entries[i].0);
                }
            }
            EvictionPolicy::SpatialLRU { player_pos, radius } => {
                // Sort by distance from player, then by time
                let mut entries: Vec<_> = self
                    .deltas
                    .iter()
                    .map(|(pos, chunk)| {
                        let chunk_center = Vec3::new(
                            (pos.x * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f32,
                            (pos.y * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f32,
                            (pos.z * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2) as f32,
                        );
                        let dist = chunk_center.distance(*player_pos);
                        (*pos, dist, chunk.last_modified)
                    })
                    .collect();

                // Only evict chunks outside radius
                entries.retain(|(_, dist, _)| *dist > *radius);
                entries.sort_by(|(_, d1, t1), (_, d2, t2)| {
                    d2.partial_cmp(d1).unwrap().then(t1.cmp(t2))
                });

                for i in 0..to_remove.min(entries.len()) {
                    self.deltas.remove(&entries[i].0);
                }
            }
            EvictionPolicy::Never => {
                // Don't evict anything
            }
        }

        self.last_gc = Instant::now();
    }

    /// Flush to disk
    pub fn flush_to_disk(&mut self, path: &Path) -> Result<(), io::Error> {
        let mut file = std::fs::File::create(path)?;

        // Header
        file.write_all(b"DLTA")?; // Magic
        file.write_all(&1u32.to_le_bytes())?; // Version
        file.write_all(&(self.deltas.len() as u32).to_le_bytes())?; // Chunk count

        // Write each chunk
        for (chunk_pos, delta) in &self.deltas {
            // Chunk position
            file.write_all(&chunk_pos.x.to_le_bytes())?;
            file.write_all(&chunk_pos.y.to_le_bytes())?;
            file.write_all(&chunk_pos.z.to_le_bytes())?;

            // Modification count
            file.write_all(&(delta.modifications.len() as u32).to_le_bytes())?;

            // Write modifications
            for (&idx, &value) in &delta.modifications {
                file.write_all(&(idx as u32).to_le_bytes())?;

                match value {
                    VoxelValue::Block(block) => {
                        file.write_all(&0u8.to_le_bytes())?; // Type: Block
                        file.write_all(&block.to_le_bytes())?;
                    }
                    VoxelValue::Density(density, material) => {
                        file.write_all(&1u8.to_le_bytes())?; // Type: Density
                        file.write_all(&density.to_le_bytes())?;
                        file.write_all(&material.to_le_bytes())?;
                    }
                }
            }
        }

        self.last_flush = Instant::now();

        Ok(())
    }

    /// Load from disk
    pub fn load_from_disk(path: &Path) -> Result<Self, io::Error> {
        let mut file = std::fs::File::open(path)?;

        // Read header
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"DLTA" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)?;
        let _version = u32::from_le_bytes(version_bytes);

        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)?;
        let chunk_count = u32::from_le_bytes(count_bytes);

        let mut store = Self::new(GCConfig::default());

        // Read chunks
        for _ in 0..chunk_count {
            // Read chunk pos
            let mut x_bytes = [0u8; 4];
            let mut y_bytes = [0u8; 4];
            let mut z_bytes = [0u8; 4];
            file.read_exact(&mut x_bytes)?;
            file.read_exact(&mut y_bytes)?;
            file.read_exact(&mut z_bytes)?;

            let chunk_pos = IVec3::new(
                i32::from_le_bytes(x_bytes),
                i32::from_le_bytes(y_bytes),
                i32::from_le_bytes(z_bytes),
            );

            // Read modification count
            let mut mod_count_bytes = [0u8; 4];
            file.read_exact(&mut mod_count_bytes)?;
            let mod_count = u32::from_le_bytes(mod_count_bytes);

            let mut delta = DeltaChunk::new();

            // Read modifications
            for _ in 0..mod_count {
                let mut idx_bytes = [0u8; 4];
                file.read_exact(&mut idx_bytes)?;
                let idx = u32::from_le_bytes(idx_bytes) as usize;

                let mut type_byte = [0u8; 1];
                file.read_exact(&mut type_byte)?;

                let value = if type_byte[0] == 0 {
                    // Block
                    let mut block_bytes = [0u8; 2];
                    file.read_exact(&mut block_bytes)?;
                    VoxelValue::Block(u16::from_le_bytes(block_bytes))
                } else {
                    // Density
                    let mut density_byte = [0u8; 1];
                    let mut material_byte = [0u8; 1];
                    file.read_exact(&mut density_byte)?;
                    file.read_exact(&mut material_byte)?;
                    VoxelValue::Density(density_byte[0], material_byte[0])
                };

                delta.modifications.insert(idx, value);
            }

            store.deltas.insert(chunk_pos, delta);
        }

        Ok(store)
    }

    /// Get statistics
    pub fn stats(&self) -> DeltaStats {
        let mut total_mods = 0;
        let mut dirty_count = 0;

        for delta in self.deltas.values() {
            total_mods += delta.modifications.len();
            if delta.dirty {
                dirty_count += 1;
            }
        }

        DeltaStats {
            total_deltas: total_mods,
            memory_usage_bytes: total_mods
                * (std::mem::size_of::<usize>() + std::mem::size_of::<VoxelValue>()),
            dirty_chunks: dirty_count,
        }
    }

    /// Clear all deltas
    pub fn clear(&mut self) {
        self.deltas.clear();
    }
}

/// Provider with delta overlay
pub struct ProviderWithEdits<P: VoxelProvider> {
    /// Base provider (read-only)
    base: Arc<P>,

    /// Delta overlay
    delta: Arc<RwLock<DeltaStore>>,
}

impl<P: VoxelProvider> ProviderWithEdits<P> {
    /// Create new provider with edits
    pub fn new(base: Arc<P>, gc_config: GCConfig) -> Self {
        Self {
            base,
            delta: Arc::new(RwLock::new(DeltaStore::new(gc_config))),
        }
    }

    /// Get delta store (for save/load)
    pub fn delta(&self) -> Arc<RwLock<DeltaStore>> {
        self.delta.clone()
    }
}

impl<P: VoxelProvider> VoxelProvider for ProviderWithEdits<P> {
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) -> Result<VoxelData, ProviderError> {
        // Read base data
        let mut data = self.base.read_range(min, max, lod)?;

        // Apply deltas
        let delta = self.delta.read().unwrap();
        delta.apply_to(&mut data, min, max);

        Ok(data)
    }

    fn write_voxel(&mut self, pos: IVec3, value: VoxelValue) -> Result<(), ProviderError> {
        let mut delta = self.delta.write().unwrap();
        delta.set_voxel(pos, value);
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "ProviderWithEdits"
    }

    fn is_writable(&self) -> bool {
        true
    }
}
