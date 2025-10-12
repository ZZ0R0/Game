#![forbid(unsafe_code)]
#![deny(warnings)]

// ChunkV2: Modern chunk system with BlockId, palette, dirty flags, and world coordinates
pub mod atlas;
pub mod chunk;
pub mod meshing;
pub mod meshing_config;
pub mod raycast;

// New: Chunk loading system with async generation
pub mod chunk_ring;
pub mod generator;
pub mod job_queue;
pub mod storage;

// Milestone 1: Unified voxel model
pub mod volume;
pub mod voxel_schema;

// Milestone 2: Provider system
pub mod providers;

// Re-exports
pub use chunk::{
    is_solid, is_transparent, BlockId, Chunk, ChunkManager, DirtyFlags, AIR, CHUNK_SIZE,
    CHUNK_VOLUME, DIRT, GLASS, GRASS, LEAVES, STONE, WATER, WOOD,
};

pub use meshing::{
    mesh_chunk_v2 as mesh_chunk, mesh_chunk_with_ao, MeshData, MeshPosUv,
};

pub use atlas::{AtlasRect, FaceDir, TextureAtlas};

pub use raycast::{raycast_dda, RaycastHit};

pub use chunk_ring::{chunk_to_world, world_to_chunk, distance_to_chunk, distance_to_chunk_squared, ChunkRing, ChunkRingConfig};
pub use generator::{Biome, TerrainConfig, TerrainGenerator};
pub use job_queue::{ChunkJob, JobQueue, JobResult, JobWorker, WorkerHandle};
pub use meshing_config::MeshingConfig;
pub use storage::{ChunkPool, MeshPool};

// Milestone 1 exports
pub use volume::{
    world_to_chunk_pos, CelestialVolume, DirtyRegions, GridVolume, ProceduralProvider, Volume,
    VolumeTransform,
};
pub use voxel_schema::{
    BlockSchema, Density, DensitySchema, MaterialId, VoxelSchema, MAT_AIR, MAT_DIRT, MAT_GRASS,
    MAT_STONE, MAT_WATER, MAT_WOOD,
};

// Milestone 2 exports
pub use providers::{
    AsteroidConfig, AsteroidProvider, BiomeBand, BiomeType, Brush, BrushShape, ChunkData,
    DeltaStats, DeltaStore, EvictionPolicy, GCConfig, GridStoreConfig, GridStoreProvider,
    NoiseLayer, NoiseMode, NoiseParams, PlanetConfig, PlanetProvider, ProviderError,
    ProviderWithEdits, VoxelData, VoxelProvider, VoxelValue,
};
