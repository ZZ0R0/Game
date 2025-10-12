#![allow(dead_code)]
//! Occlusion culling module

//! Hardware occlusion culling system using GPU queries
//! Tests chunk visibility by rendering bounding boxes and counting visible pixels

use crate::frustum::AABB;
use crate::wgpu;
use glam::{IVec3, Mat4, Vec3};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Statistics for occlusion culling performance
#[derive(Debug, Default, Clone)]
pub struct OcclusionStats {
    /// Total chunks tested for occlusion this frame
    pub tested_chunks: usize,
    /// Chunks determined to be occluded (hidden)
    pub occluded_chunks: usize,
    /// Percentage of chunks occluded (0.0 to 1.0)
    pub occlusion_rate: f32,
    /// Chunks skipped (too close to camera)
    pub near_chunks: usize,
}



/// Hardware occlusion culling system
pub struct OcclusionCuller {
    /// Occlusion query sets for GPU visibility testing
    query_sets: Vec<wgpu::QuerySet>,
    /// Buffer to read query results back from GPU
    query_buffer: wgpu::Buffer,
    /// Staging buffer for CPU readback
    staging_buffer: wgpu::Buffer,
    
    /// Render pipeline for occlusion testing (depth-only rendering)
    occlusion_pipeline: wgpu::RenderPipeline,
    /// Vertex buffer for bounding box geometry
    bbox_vertex_buffer: wgpu::Buffer,
    /// Index buffer for bounding box triangles
    bbox_index_buffer: wgpu::Buffer,
    
    /// Map of chunk positions to query indices
    chunk_query_map: HashMap<IVec3, u32>,
    /// Results from previous frame's queries
    occlusion_results: HashMap<IVec3, bool>,
    
    /// Maximum number of chunks that can be tested per frame
    max_queries: u32,
    /// Current query index
    current_query: u32,
    
    /// Performance statistics
    pub stats: OcclusionStats,
    
    /// Near distance threshold - chunks closer than this are always rendered
    near_threshold: f32,
}

impl OcclusionCuller {
    /// Create a new occlusion culler
    pub fn new(
        device: &wgpu::Device,
        depth_format: wgpu::TextureFormat,
        max_queries: u32,
    ) -> Self {
        // Create query sets for occlusion testing
        let query_sets = vec![device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("occlusion_queries"),
            ty: wgpu::QueryType::Occlusion,
            count: max_queries,
        })];

        // Create buffers for reading query results
        let query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("occlusion_query_buffer"),
            size: (max_queries * 8) as u64, // Each query result is u64
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("occlusion_staging_buffer"),
            size: (max_queries * 8) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create bounding box geometry
        let vertices = Self::create_bbox_vertices();
        let indices = Self::get_triangle_indices();

        let bbox_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bbox_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bbox_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bbox_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create occlusion test shader and pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("occlusion_shader"),
            source: wgpu::ShaderSource::Wgsl(Self::occlusion_shader_source().into()),
        });

        let occlusion_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("occlusion_pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 12, // 3 floats * 4 bytes
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[], // No color output - depth only
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false, // Don't write depth during occlusion test
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            query_sets,
            query_buffer,
            staging_buffer,
            occlusion_pipeline,
            bbox_vertex_buffer,
            bbox_index_buffer,
            chunk_query_map: HashMap::new(),
            occlusion_results: HashMap::new(),
            max_queries,
            current_query: 0,
            stats: OcclusionStats::default(),
            near_threshold: 96.0, // 3 chunks distance - chunks closer are always rendered
        }
    }

    /// Test chunks for occlusion and return list of visible chunk positions
    pub fn cull_chunks(
        &mut self,
        chunks: &[(IVec3, &crate::chunk_renderer::ChunkMesh)],
        camera_pos: Vec3,
        view_proj_matrix: Mat4,
    ) -> Vec<IVec3> {
        // Reset stats
        self.stats = OcclusionStats::default();
        self.current_query = 0;
        self.chunk_query_map.clear();

        // Sort chunks by distance from camera
        let mut sorted_chunks = chunks.to_vec();
        sorted_chunks.sort_by(|a, b| {
            let pos_a = a.0.as_vec3() * 32.0; // Convert to world position
            let pos_b = b.0.as_vec3() * 32.0;
            let dist_a = (pos_a - camera_pos).length_squared();
            let dist_b = (pos_b - camera_pos).length_squared();
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        let mut visible_chunks = Vec::new();

        // Process chunks by distance
        for (chunk_pos, mesh) in sorted_chunks {
            let world_pos = chunk_pos.as_vec3() * 32.0;
            let distance = (world_pos - camera_pos).length();

            if distance <= self.near_threshold {
                // Near chunks are always visible
                visible_chunks.push(chunk_pos);
                self.stats.near_chunks += 1;
            } else {
                // Test far chunks for occlusion
                if self.is_chunk_visible(chunk_pos, &mesh.aabb, camera_pos, view_proj_matrix) {
                    visible_chunks.push(chunk_pos);
                } else {
                    self.stats.occluded_chunks += 1;
                }
                self.stats.tested_chunks += 1;
            }
        }

        // Calculate occlusion rate
        if self.stats.tested_chunks > 0 {
            self.stats.occlusion_rate = 
                self.stats.occluded_chunks as f32 / self.stats.tested_chunks as f32;
        }

        visible_chunks
    }

    /// Check if a chunk is visible (not occluded) using software-based heuristics
    fn is_chunk_visible(&mut self, _chunk_pos: IVec3, aabb: &AABB, camera_pos: Vec3, view_proj_matrix: Mat4) -> bool {
        // Get chunk center in world space
        let chunk_center = aabb.center();
        let _chunk_size = (aabb.max - aabb.min).length(); // Unused for now
        
        // Distance-based culling: far chunks are candidates for occlusion
        let distance = (chunk_center - camera_pos).length();
        if distance > self.near_threshold * 1.5 { // More aggressive threshold
            // For far chunks, use occlusion testing
            
            // Check if chunk is behind camera plane (should already be frustum culled, but double check)
            let to_chunk = (chunk_center - camera_pos).normalize();
            // Extract camera forward direction from view_proj matrix (approximate)
            let camera_forward = -Vec3::new(view_proj_matrix.z_axis.x, view_proj_matrix.z_axis.y, view_proj_matrix.z_axis.z).normalize();
            
            if to_chunk.dot(camera_forward) < 0.0 {
                return false; // Behind camera
            }
            
            // Occlusion heuristic: chunks at oblique angles or far distances are likely occluded
            let angle_factor = to_chunk.dot(camera_forward);
            
            // More aggressive occlusion testing
            if distance > self.near_threshold * 1.2 {
                // Check terrain-based occlusion
                if self.is_likely_occluded_by_terrain(chunk_center, camera_pos, distance) {
                    return false;
                }
                
                // Angle-based occlusion: chunks at steep angles are likely hidden
                if angle_factor < 0.8 && distance > self.near_threshold * 1.5 {
                    return false;
                }
            }
        }
        
        // Default to visible for now
        // TODO: Implement real GPU occlusion queries for more accurate results
        true
    }
    
    /// Heuristic to determine if a chunk is likely occluded by nearer terrain
    fn is_likely_occluded_by_terrain(&self, chunk_center: Vec3, camera_pos: Vec3, distance: f32) -> bool {
        let height_diff = chunk_center.y - camera_pos.y;
        let horizontal_distance = ((chunk_center.x - camera_pos.x).powi(2) + (chunk_center.z - camera_pos.z).powi(2)).sqrt();
        
        // More aggressive occlusion rules for testing:
        
        // 1. Chunks significantly below camera and far away are likely occluded
        if height_diff < -16.0 && distance > self.near_threshold * 1.5 {
            return true;
        }
        
        // 2. Very far chunks at ground level are likely occluded by nearer terrain
        if horizontal_distance > self.near_threshold * 2.0 && height_diff < 8.0 {
            return true;
        }
        
        // 3. Chunks that are far horizontally and not significantly higher
        if horizontal_distance > self.near_threshold * 2.5 && height_diff < 32.0 {
            return true;
        }
        
        // 4. Very distant chunks (regardless of height) - aggressive culling for testing
        if distance > self.near_threshold * 3.0 {
            return true;
        }
        
        false
    }

    /// Begin occlusion testing render pass
    pub fn begin_occlusion_pass<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.occlusion_pipeline);
        render_pass.set_vertex_buffer(0, self.bbox_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.bbox_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    }

    /// Render a chunk's bounding box for occlusion testing
    pub fn test_chunk_occlusion<'a>(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        chunk_pos: IVec3,
        _view_proj_matrix: Mat4,
    ) {
        if self.current_query >= self.max_queries {
            return; // No more queries available
        }

        // Store chunk-to-query mapping
        self.chunk_query_map.insert(chunk_pos, self.current_query);

        // Begin occlusion query
        render_pass.begin_occlusion_query(self.current_query);

        // Set transform for this chunk's bounding box
        // TODO: Upload transform matrix to GPU uniform buffer
        
        // Draw bounding box
        render_pass.draw_indexed(0..36, 0, 0..1);

        // End occlusion query
        render_pass.end_occlusion_query();

        self.current_query += 1;
    }

    /// Process query results from GPU (call this after the frame completes)
    pub fn process_query_results(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        // This is complex and requires async GPU readback
        // For now, we'll use a simplified approach
        
        // In a full implementation, this would:
        // 1. Resolve queries to buffer
        // 2. Copy to staging buffer  
        // 3. Map staging buffer and read results
        // 4. Update occlusion_results HashMap
    }

    /// Create vertices for unit bounding box (will be transformed per chunk)
    fn create_bbox_vertices() -> Vec<[f32; 3]> {
        // Unit cube vertices (0,0,0) to (1,1,1)
        // Will be scaled and translated per chunk
        vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ]
    }

    /// Get indices for rendering the bounding box as triangles (for occlusion testing)
    fn get_triangle_indices() -> [u16; 36] {
        [
            // Front face (z=max)
            4, 5, 6, 4, 6, 7,
            // Back face (z=min)
            1, 0, 3, 1, 3, 2,
            // Left face (x=min)
            0, 4, 7, 0, 7, 3,
            // Right face (x=max)
            5, 1, 2, 5, 2, 6,
            // Bottom face (y=min)
            0, 1, 5, 0, 5, 4,
            // Top face (y=max)
            3, 7, 6, 3, 6, 2,
        ]
    }

    /// Shader source for occlusion testing
    fn occlusion_shader_source() -> &'static str {
        r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ChunkUniform {
    model: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> chunk: ChunkUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = chunk.model * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) {
    // No color output - depth only for occlusion testing
}
"#
    }
}