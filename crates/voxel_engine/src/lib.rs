#![forbid(unsafe_code)]
#![deny(warnings)]
pub mod chunk;
pub mod meshing;

pub use chunk::{Chunk, Voxel, AIR, SOLID, CHUNK_SIZE};
pub use meshing::{mesh_chunk, MeshPosUv};
