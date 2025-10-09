//! GPU buffer pool for efficient vertex/index buffer recycling
//! 
//! Avoids frequent GPU allocations by reusing buffers

use crate::wgpu;
use std::collections::VecDeque;

/// A pooled buffer entry
pub struct PooledBuffer {
    pub buffer: wgpu::Buffer,
    pub capacity: u64,  // Size in bytes
}

/// Pool for recycling GPU buffers
pub struct BufferPool {
    /// Available vertex buffers (sorted by capacity)
    vertex_buffers: VecDeque<PooledBuffer>,
    
    /// Available index buffers (sorted by capacity)
    index_buffers: VecDeque<PooledBuffer>,
    
    /// Maximum pool size per type
    max_pool_size: usize,
    
    /// Statistics
    pub stats: PoolStats,
}

#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    pub vertex_buffers_in_pool: usize,
    pub index_buffers_in_pool: usize,
    pub vertex_allocations: u64,
    pub index_allocations: u64,
    pub vertex_reuses: u64,
    pub index_reuses: u64,
}

impl BufferPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            vertex_buffers: VecDeque::with_capacity(max_pool_size),
            index_buffers: VecDeque::with_capacity(max_pool_size),
            max_pool_size,
            stats: PoolStats::default(),
        }
    }
    
    /// Acquire a vertex buffer (reuse from pool or create new)
    pub fn acquire_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        required_size: u64,
    ) -> wgpu::Buffer {
        // Try to find a buffer with sufficient capacity
        if let Some(pos) = self.vertex_buffers.iter().position(|b| b.capacity >= required_size) {
            let pooled = self.vertex_buffers.remove(pos).unwrap();
            self.stats.vertex_reuses += 1;
            self.update_stats();
            return pooled.buffer;
        }
        
        // No suitable buffer found, create new one
        self.stats.vertex_allocations += 1;
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pooled_vertex_buffer"),
            size: required_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
    
    /// Acquire an index buffer (reuse from pool or create new)
    pub fn acquire_index_buffer(
        &mut self,
        device: &wgpu::Device,
        required_size: u64,
    ) -> wgpu::Buffer {
        // Try to find a buffer with sufficient capacity
        if let Some(pos) = self.index_buffers.iter().position(|b| b.capacity >= required_size) {
            let pooled = self.index_buffers.remove(pos).unwrap();
            self.stats.index_reuses += 1;
            self.update_stats();
            return pooled.buffer;
        }
        
        // No suitable buffer found, create new one
        self.stats.index_allocations += 1;
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pooled_index_buffer"),
            size: required_size,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
    
    /// Return a vertex buffer to the pool
    pub fn return_vertex_buffer(&mut self, buffer: wgpu::Buffer, capacity: u64) {
        if self.vertex_buffers.len() < self.max_pool_size {
            self.vertex_buffers.push_back(PooledBuffer { buffer, capacity });
            self.update_stats();
        }
        // Otherwise, buffer is dropped and freed
    }
    
    /// Return an index buffer to the pool
    pub fn return_index_buffer(&mut self, buffer: wgpu::Buffer, capacity: u64) {
        if self.index_buffers.len() < self.max_pool_size {
            self.index_buffers.push_back(PooledBuffer { buffer, capacity });
            self.update_stats();
        }
        // Otherwise, buffer is dropped and freed
    }
    
    /// Clear all buffers from pool
    pub fn clear(&mut self) {
        self.vertex_buffers.clear();
        self.index_buffers.clear();
        self.update_stats();
    }
    
    fn update_stats(&mut self) {
        self.stats.vertex_buffers_in_pool = self.vertex_buffers.len();
        self.stats.index_buffers_in_pool = self.index_buffers.len();
    }
    
    /// Get reuse rate (0.0 to 1.0)
    pub fn reuse_rate(&self) -> f32 {
        let total_vertex = self.stats.vertex_allocations + self.stats.vertex_reuses;
        let total_index = self.stats.index_allocations + self.stats.index_reuses;
        let total = total_vertex + total_index;
        
        if total == 0 {
            return 0.0;
        }
        
        let reuses = self.stats.vertex_reuses + self.stats.index_reuses;
        reuses as f32 / total as f32
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(128) // Default: 128 buffers per type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pool_stats() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.stats.vertex_buffers_in_pool, 0);
        assert_eq!(pool.stats.index_buffers_in_pool, 0);
    }
    
    #[test]
    fn test_reuse_rate() {
        let mut pool = BufferPool::new(10);
        pool.stats.vertex_allocations = 10;
        pool.stats.vertex_reuses = 40;
        pool.stats.index_allocations = 5;
        pool.stats.index_reuses = 45;
        
        // Total: 100 operations, 85 reuses = 85% reuse rate
        let rate = pool.reuse_rate();
        assert!((rate - 0.85).abs() < 0.01);
    }
}
