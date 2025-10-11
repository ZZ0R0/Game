use crate::gfx::{Gfx, VertexTex};
use crate::wgpu;
use wgpu::util::DeviceExt;

pub struct MeshBuffers {
    pub vbuf: wgpu::Buffer,
    pub ibuf: wgpu::Buffer,
    pub index_count: u32,
}

impl<'w> Gfx<'w> {
    pub fn upload_pos_uv(
        &self,
        positions: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) -> MeshBuffers {
        let verts: Vec<VertexTex> = positions
            .iter()
            .zip(uvs.iter())
            .map(|(p, t)| VertexTex { pos: *p, uv: *t })
            .collect();

        let vbuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("voxel_vbuf"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ibuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("voxel_ibuf"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        MeshBuffers {
            vbuf,
            ibuf,
            index_count: indices.len() as u32,
        }
    }

    /// Upload a chunk mesh to the ChunkRenderer
    pub fn upload_chunk(&mut self, position: glam::IVec3, vertices: &[u8], indices: &[u32]) {
        self.chunk_renderer
            .insert_chunk(&self.device, position, vertices, indices);
    }
}
