//! Meshing configuration with greedy meshing support

use crate::chunk::{Chunk, ChunkManager};
use crate::atlas::TextureAtlas;
use crate::meshing::{greedy_mesh_chunk, mesh_chunk_with_ao, MeshData};

/// Configuration for mesh generation algorithms
#[derive(Debug, Clone)]
pub struct MeshingConfig {
    /// Whether to use greedy meshing (true) or face-by-face (false)
    pub use_greedy_meshing: bool,
    
    /// Whether to calculate ambient occlusion (slower but better visuals)
    pub calculate_ao: bool,
}

impl Default for MeshingConfig {
    fn default() -> Self {
        Self {
            use_greedy_meshing: true,  // Default to optimized greedy meshing
            calculate_ao: false,       // Default to faster path without AO
        }
    }
}

impl MeshingConfig {
    /// Create new meshing config
    pub fn new(use_greedy_meshing: bool) -> Self {
        Self {
            use_greedy_meshing,
            calculate_ao: false,
        }
    }

    /// High performance config - greedy meshing without AO
    pub fn performance() -> Self {
        Self {
            use_greedy_meshing: true,
            calculate_ao: false,
        }
    }

    /// High quality config - greedy meshing with AO
    pub fn quality() -> Self {
        Self {
            use_greedy_meshing: true,
            calculate_ao: true,
        }
    }

    /// Fast config - face-by-face meshing (legacy compatibility)
    pub fn fast() -> Self {
        Self {
            use_greedy_meshing: false,
            calculate_ao: false,
        }
    }

    /// Execute meshing based on this configuration
    pub fn mesh_chunk(
        &self,
        chunk: &Chunk,
        chunk_manager: Option<&ChunkManager>,
        atlas: &TextureAtlas,
    ) -> MeshData {
        if self.use_greedy_meshing {
            // Use optimized greedy meshing with neighbor support
            greedy_mesh_chunk(chunk, chunk_manager, atlas)
        } else {
            // Use legacy face-by-face meshing (no neighbor support needed)
            mesh_chunk_with_ao(chunk)
        }
    }

    /// Mesh a chunk without access to chunk manager (for async workers)
    pub fn mesh_chunk_standalone(&self, chunk: &Chunk, atlas: &TextureAtlas) -> MeshData {
        self.mesh_chunk(chunk, None, atlas)
    }

    /// Create config optimized for maximum performance (greedy meshing, no AO)
    pub fn maximum_performance() -> Self {
        Self {
            use_greedy_meshing: true,
            calculate_ao: false,
        }
    }

    /// Create config balanced between performance and quality
    pub fn balanced() -> Self {
        Self {
            use_greedy_meshing: true,
            calculate_ao: false, // AO still disabled until implementation
        }
    }

    /// Check if this config will produce cross-chunk seamless meshes
    pub fn supports_cross_chunk_meshing(&self) -> bool {
        self.use_greedy_meshing
    }

    /// Get expected memory usage reduction compared to face-by-face meshing
    pub fn expected_memory_reduction(&self) -> f32 {
        if self.use_greedy_meshing {
            0.7 // Expect ~70% memory reduction on average
        } else {
            0.0
        }
    }
}