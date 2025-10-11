//! Meshing configuration and strategy pattern
//!
//! This module defines how chunks should be meshed, following the Open-Close Principle.
//! New meshing strategies can be added without modifying existing code.

use crate::atlas::TextureAtlas;
use crate::chunk::{Chunk, ChunkManager};
use crate::meshing::{greedy_mesh_chunk, mesh_chunk_with_ao, MeshData};
use glam::IVec3;
use rayon::prelude::*;
use std::sync::Arc;

/// Configuration for how chunks should be meshed
#[derive(Debug, Clone)]
pub struct MeshingConfig {
    /// Whether to use greedy meshing algorithm (faster, fewer polygons)
    /// If false, uses legacy face-by-face meshing
    pub use_greedy_meshing: bool,

    /// Whether to calculate ambient occlusion (slower but better visuals)
    pub calculate_ao: bool,
}

impl Default for MeshingConfig {
    fn default() -> Self {
        Self {
            use_greedy_meshing: true, // Default to optimized path
            calculate_ao: false,      // Default to faster path
        }
    }
}

impl MeshingConfig {
    /// Create a new meshing config
    pub fn new(use_greedy_meshing: bool) -> Self {
        Self {
            use_greedy_meshing,
            calculate_ao: false,
        }
    }

    /// Create config for maximum performance
    pub fn fast() -> Self {
        Self {
            use_greedy_meshing: true,
            calculate_ao: false,
        }
    }

    /// Create config for maximum quality
    pub fn quality() -> Self {
        Self {
            use_greedy_meshing: true,
            calculate_ao: true,
        }
    }

    /// Execute meshing based on this configuration
    ///
    /// This is the strategy pattern implementation - the config object
    /// decides which algorithm to use, following Open-Close Principle.
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

    /// Mesh multiple chunks in parallel using rayon
    ///
    /// This respects the meshing configuration for all chunks.
    /// Uses Rayon's parallel iterator for efficient multi-threading.
    pub fn mesh_chunks_parallel(
        &self,
        chunks: &[(IVec3, Arc<Chunk>)],
        chunk_manager: &ChunkManager,
        atlas: &TextureAtlas,
    ) -> Vec<(IVec3, MeshData)> {
        chunks
            .par_iter()
            .map(|(position, chunk)| {
                let mesh = self.mesh_chunk(chunk, Some(chunk_manager), atlas);
                (*position, mesh)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MeshingConfig::default();
        assert!(config.use_greedy_meshing);
        assert!(!config.calculate_ao);
    }

    #[test]
    fn test_fast_config() {
        let config = MeshingConfig::fast();
        assert!(config.use_greedy_meshing);
        assert!(!config.calculate_ao);
    }

    #[test]
    fn test_quality_config() {
        let config = MeshingConfig::quality();
        assert!(config.use_greedy_meshing);
        assert!(config.calculate_ao);
    }
}
