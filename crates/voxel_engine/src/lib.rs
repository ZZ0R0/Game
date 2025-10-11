#![forbid(unsafe_code)]
#![deny(warnings)]

// ChunkV2: Modern chunk system with BlockId, palette, dirty flags, and world coordinates
pub mod chunk;
pub mod meshing;
pub mod meshing_config;
pub mod atlas;
pub mod raycast;

// New: Chunk loading system with async generation
pub mod chunk_ring;
pub mod job_queue;
pub mod generator;
pub mod storage;

// Milestone 1: Unified voxel model
pub mod voxel_schema;
pub mod volume;

// Milestone 2: Provider system
pub mod providers;

// Re-exports
pub use chunk::{
    Chunk,
    ChunkManager,
    BlockId,
    DirtyFlags,
    CHUNK_SIZE,
    CHUNK_VOLUME,
    AIR, STONE, DIRT, GRASS, WOOD, LEAVES, WATER, GLASS,
    is_transparent, is_solid,
};

pub use meshing::{
    mesh_chunk_v2 as mesh_chunk, 
    mesh_chunk_with_ao,
    MeshPosUv,
    MeshData,
    SeparatedMesh,
    greedy_mesh_chunk,
    greedy_mesh_chunk_separated,
};

pub use atlas::{TextureAtlas, AtlasRect, FaceDir};

pub use raycast::{raycast_dda, RaycastHit};

pub use chunk_ring::{ChunkRing, ChunkRingConfig, world_to_chunk, chunk_to_world};
pub use job_queue::{JobQueue, JobWorker, WorkerHandle, ChunkJob, JobResult};
pub use generator::{TerrainGenerator, TerrainConfig, Biome};
pub use storage::{ChunkPool, MeshPool};
pub use meshing_config::MeshingConfig;

// Milestone 1 exports
pub use voxel_schema::{
    VoxelSchema, BlockSchema, DensitySchema,
    MaterialId, Density,
    MAT_AIR, MAT_STONE, MAT_DIRT, MAT_GRASS, MAT_WOOD, MAT_WATER,
};
pub use volume::{
    Volume, GridVolume, CelestialVolume,
    VolumeTransform, DirtyRegions, ProceduralProvider,
    world_to_chunk_pos,
};

// Milestone 2 exports
pub use providers::{
    VoxelProvider, VoxelValue, VoxelData, Brush, BrushShape,
    ProviderError, GridStoreProvider, GridStoreConfig, ChunkData,
    PlanetProvider, PlanetConfig, NoiseLayer, BiomeBand, BiomeType,
    AsteroidProvider, AsteroidConfig, NoiseMode, NoiseParams,
    DeltaStore, DeltaStats, GCConfig, EvictionPolicy,
    ProviderWithEdits,
};
