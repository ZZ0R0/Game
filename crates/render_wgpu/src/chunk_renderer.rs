//! Chunk rendering system with per-chunk buffers and frustum culling

use crate::buffer_pool::BufferPool;
use crate::frustum::{Frustum, AABB};
use crate::wgpu;
use glam::{IVec3, Mat4};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Per-chunk GPU mesh data
pub struct ChunkMesh {
    /// Vertex buffer
    pub vbuf: wgpu::Buffer,
    pub vbuf_size: u64,

    /// Index buffer
    pub ibuf: wgpu::Buffer,
    pub ibuf_size: u64,

    /// Number of indices to draw
    pub index_count: u32,

    /// Number of triangles
    pub triangle_count: u32,

    /// AABB for frustum culling
    pub aabb: AABB,

    /// Chunk position
    pub position: IVec3,
}

impl ChunkMesh {
    pub fn new(
        device: &wgpu::Device,
        vertices: &[u8],
        indices: &[u32],
        position: IVec3,
        chunk_size: f32,
    ) -> Self {
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!(
                "chunk_vbuf_{}_{}_{}",
                position.x, position.y, position.z
            )),
            contents: vertices,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!(
                "chunk_ibuf_{}_{}_{}",
                position.x, position.y, position.z
            )),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let aabb = AABB::from_chunk_pos(position.x, position.y, position.z, chunk_size);

        Self {
            vbuf,
            vbuf_size: vertices.len() as u64,
            ibuf,
            ibuf_size: (indices.len() * std::mem::size_of::<u32>()) as u64,
            index_count: indices.len() as u32,
            triangle_count: (indices.len() / 3) as u32,
            aabb,
            position,
        }
    }
}

/// Manages all chunk meshes for rendering
pub struct ChunkRenderer {
    /// Map: chunk position → mesh
    pub meshes: HashMap<IVec3, ChunkMesh>,

    /// Buffer pool for recycling
    pub buffer_pool: BufferPool,

    /// Chunk size (usually 32.0)
    pub chunk_size: f32,

    /// Statistics
    pub stats: RenderStats,
}

#[derive(Debug, Default, Clone)]
pub struct RenderStats {
    /// Total chunks loaded
    pub total_chunks: usize,

    /// Chunks visible this frame (after culling)
    pub visible_chunks: usize,

    /// Total triangles in all chunks
    pub total_triangles: u32,

    /// Triangles rendered this frame
    pub rendered_triangles: u32,

    /// Draw calls this frame
    pub draw_calls: u32,

    /// Culled chunks this frame
    pub culled_chunks: usize,
}

impl ChunkRenderer {
    pub fn new(chunk_size: f32) -> Self {
        Self {
            meshes: HashMap::new(),
            buffer_pool: BufferPool::new(256),
            chunk_size,
            stats: RenderStats::default(),
        }
    }

    /// Add or update a chunk mesh
    pub fn insert_chunk(
        &mut self,
        device: &wgpu::Device,
        position: IVec3,
        vertices: &[u8],
        indices: &[u32],
    ) {
        // Remove old mesh if exists (buffer will be returned to pool later if needed)
        self.meshes.remove(&position);

        // Create new mesh
        let mesh = ChunkMesh::new(device, vertices, indices, position, self.chunk_size);
        self.meshes.insert(position, mesh);

        self.update_stats();
    }

    /// Remove a chunk mesh
    pub fn remove_chunk(&mut self, position: IVec3) {
        if let Some(_mesh) = self.meshes.remove(&position) {
            // TODO: Return buffers to pool when we refactor to use pooled buffers
            self.update_stats();
        }
    }

    /// Perform frustum culling and return visible chunk positions
    pub fn cull_chunks(&mut self, vp_matrix: Mat4) -> Vec<IVec3> {
        let frustum = Frustum::from_matrix(vp_matrix);

        let mut visible = Vec::new();
        let mut rendered_triangles = 0u32;
        let mut culled = 0usize;

        for (pos, mesh) in &self.meshes {
            if frustum.test_aabb(&mesh.aabb) {
                visible.push(*pos);
                rendered_triangles += mesh.triangle_count;
            } else {
                culled += 1;
            }
        }

        self.stats.visible_chunks = visible.len();
        self.stats.culled_chunks = culled;
        self.stats.rendered_triangles = rendered_triangles;
        self.stats.draw_calls = visible.len() as u32;

        visible
    }

    /// Clear all meshes
    pub fn clear(&mut self) {
        self.meshes.clear();
        self.buffer_pool.clear();
        self.update_stats();
    }

    fn update_stats(&mut self) {
        self.stats.total_chunks = self.meshes.len();
        self.stats.total_triangles = self.meshes.values().map(|m| m.triangle_count).sum();
    }

    /// Get mesh for a chunk position
    pub fn get_mesh(&self, position: IVec3) -> Option<&ChunkMesh> {
        self.meshes.get(&position)
    }
}

impl Default for ChunkRenderer {
    fn default() -> Self {
        Self::new(32.0)
    }
}
