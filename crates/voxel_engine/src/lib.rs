#![forbid(unsafe_code)]
#![deny(warnings)]

// ChunkV2: Modern chunk system with BlockId, palette, dirty flags, and world coordinates
pub mod chunk;
pub mod meshing;
pub mod atlas;

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
    MeshPosUv,
    MeshData,
    SeparatedMesh,
    greedy_mesh_chunk,
    greedy_mesh_chunk_separated,
};

pub use atlas::{TextureAtlas, AtlasRect, FaceDir};
