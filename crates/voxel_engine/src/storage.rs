//! Memory pools for chunks and meshes to avoid frequent allocations

use crate::chunk::Chunk;
use crate::meshing::MeshData;
use glam::IVec3;
use std::collections::VecDeque;

/// Pool of pre-allocated chunks
pub struct ChunkPool {
    /// Available chunks ready to be reused
    available: VecDeque<Chunk>,

    /// Maximum pool size
    max_size: usize,

    /// Statistics
    pub stats: ChunkPoolStats,
}

#[derive(Debug, Clone, Default)]
pub struct ChunkPoolStats {
    pub available_chunks: usize,
    pub allocations: u64,
    pub reuses: u64,
}

impl ChunkPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            available: VecDeque::with_capacity(max_size),
            max_size,
            stats: ChunkPoolStats::default(),
        }
    }

    /// Acquire a chunk from the pool (or allocate new one)
    pub fn acquire(&mut self, position: IVec3) -> Chunk {
        if let Some(mut chunk) = self.available.pop_front() {
            // Reuse existing chunk
            chunk.position = position;
            self.stats.reuses += 1;
            self.update_stats();
            chunk
        } else {
            // Allocate new chunk
            self.stats.allocations += 1;
            Chunk::new(position)
        }
    }

    /// Return a chunk to the pool for reuse
    pub fn release(&mut self, chunk: Chunk) {
        if self.available.len() < self.max_size {
            self.available.push_back(chunk);
            self.update_stats();
        }
        // If pool is full, chunk is dropped (deallocated)
    }

    /// Clear the entire pool
    pub fn clear(&mut self) {
        self.available.clear();
        self.update_stats();
    }

    /// Pre-allocate chunks to warm up the pool
    pub fn preallocate(&mut self, count: usize) {
        for _ in 0..count.min(self.max_size) {
            let chunk = Chunk::new(IVec3::ZERO);
            self.available.push_back(chunk);
        }
        self.update_stats();
    }

    fn update_stats(&mut self) {
        self.stats.available_chunks = self.available.len();
    }
}

impl Default for ChunkPool {
    fn default() -> Self {
        Self::new(256)
    }
}

/// Pool of mesh data to avoid frequent allocations
pub struct MeshPool {
    /// Available mesh data ready to be reused
    available: VecDeque<MeshData>,

    /// Maximum pool size
    max_size: usize,

    /// Statistics
    pub stats: MeshPoolStats,
}

#[derive(Debug, Clone, Default)]
pub struct MeshPoolStats {
    pub available_meshes: usize,
    pub allocations: u64,
    pub reuses: u64,
}

impl MeshPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            available: VecDeque::with_capacity(max_size),
            max_size,
            stats: MeshPoolStats::default(),
        }
    }

    /// Acquire a mesh from the pool (or allocate new one)
    pub fn acquire(&mut self) -> MeshData {
        if let Some(mut mesh) = self.available.pop_front() {
            // Clear and reuse existing mesh
            mesh.positions.clear();
            mesh.uvs.clear();
            mesh.indices.clear();
            mesh.ao.clear();
            self.stats.reuses += 1;
            self.update_stats();
            mesh
        } else {
            // Allocate new mesh
            self.stats.allocations += 1;
            MeshData {
                positions: Vec::new(),
                uvs: Vec::new(),
                indices: Vec::new(),
                ao: Vec::new(),
            }
        }
    }

    /// Return a mesh to the pool for reuse
    pub fn release(&mut self, mesh: MeshData) {
        if self.available.len() < self.max_size {
            self.available.push_back(mesh);
            self.update_stats();
        }
        // If pool is full, mesh is dropped (deallocated)
    }

    /// Clear the entire pool
    pub fn clear(&mut self) {
        self.available.clear();
        self.update_stats();
    }

    /// Pre-allocate meshes to warm up the pool
    pub fn preallocate(&mut self, count: usize, capacity: usize) {
        for _ in 0..count.min(self.max_size) {
            let mesh = MeshData {
                positions: Vec::with_capacity(capacity),
                uvs: Vec::with_capacity(capacity),
                indices: Vec::with_capacity(capacity),
                ao: Vec::with_capacity(capacity),
            };
            self.available.push_back(mesh);
        }
        self.update_stats();
    }

    fn update_stats(&mut self) {
        self.stats.available_meshes = self.available.len();
    }
}

impl Default for MeshPool {
    fn default() -> Self {
        Self::new(256)
    }
}
