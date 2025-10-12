//! Simple meshing configuration - always uses reliable face-by-face meshing

use crate::chunk::{Chunk, ChunkManager};
use crate::atlas::TextureAtlas;
use crate::meshing::{mesh_chunk_with_ao, MeshData};

/// Configuration for mesh generation algorithms (simplified)
#[derive(Debug, Clone)]
pub struct MeshingConfig {
    // Always uses simple face-by-face meshing - no configuration needed
}

impl Default for MeshingConfig {
    fn default() -> Self {
        Self {}
    }
}

impl MeshingConfig {
    /// Create new meshing config (parameter ignored for compatibility)
    pub fn new() -> Self {
        Self {}
    }

    /// High performance config - simple meshing
    pub fn performance() -> Self {
        Self {}
    }

    /// High quality config - simple meshing
    pub fn quality() -> Self {
        Self {}
    }

    /// Fast config - simple meshing
    pub fn fast() -> Self {
        Self {}
    }

    /// Simple meshing - always uses face-by-face algorithm
    pub fn mesh_chunk(
        &self,
        chunk: &Chunk,
        _chunk_manager: Option<&ChunkManager>,
        _atlas: &TextureAtlas,
    ) -> MeshData {
        // Always use simple, reliable meshing
        mesh_chunk_with_ao(chunk)
    }

    /// Mesh a chunk without access to chunk manager (for async workers)
    pub fn mesh_chunk_standalone(&self, chunk: &Chunk, _atlas: &TextureAtlas) -> MeshData {
        mesh_chunk_with_ao(chunk)
    }
}