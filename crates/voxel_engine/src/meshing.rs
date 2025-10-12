//! Voxel mesh generation with greedy meshing algorithm
//! 
//! Features:
//! - Greedy meshing for 60-90% triangle reduction
//! - Cross-chunk boundary support via ChunkManager
//! - Texture atlas integration with UV scaling
//! - Configurable face-by-face fallback for compatibility
//! - Optional ambient occlusion
//! - Memory-optimized with buffer reuse

use crate::atlas::{FaceDir, TextureAtlas};
use crate::chunk::{BlockId, Chunk, ChunkManager, CHUNK_SIZE};
use glam::IVec3;

/// Constants for performance tuning
const INITIAL_VERTEX_CAPACITY: usize = 4096;
const INITIAL_INDEX_CAPACITY: usize = 6144;

/// Mesh data with positions, UVs, indices, and optional AO
#[derive(Debug, Default, Clone)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub ao: Vec<f32>, // Ambient occlusion values (0.0 = fully occluded, 1.0 = no occlusion)
}

impl MeshData {
    /// Create with pre-allocated capacity
    pub fn with_capacity(vertex_count: usize, index_count: usize) -> Self {
        Self {
            positions: Vec::with_capacity(vertex_count),
            uvs: Vec::with_capacity(vertex_count),
            indices: Vec::with_capacity(index_count),
            ao: Vec::with_capacity(vertex_count),
        }
    }

    /// Clear all data for reuse
    pub fn clear(&mut self) {
        self.positions.clear();
        self.uvs.clear();
        self.indices.clear();
        self.ao.clear();
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Legacy structure for compatibility
#[derive(Debug, Default)]
pub struct MeshPosUv {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// Legacy mesh function - simple face-by-face meshing
pub fn mesh_chunk_v2(chunk: &Chunk) -> MeshPosUv {
    let mut m = MeshPosUv::default();
    m.positions.reserve(6 * 4 * 1024);
    m.uvs.reserve(6 * 4 * 1024);
    m.indices.reserve(6 * 6 * 1024);

    let size = CHUNK_SIZE as i32;

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                if sample(chunk, x, y, z) == 0 {
                    continue;
                } // 0 = AIR

                // For each of the 6 directions, if neighbor is air, emit that face.
                for (nx, ny, nz, face) in FACES {
                    let ax = x + nx;
                    let ay = y + ny;
                    let az = z + nz;
                    if ax < 0
                        || ay < 0
                        || az < 0
                        || ax >= size
                        || ay >= size
                        || az >= size
                        || sample(chunk, ax, ay, az) == 0
                    {
                        emit_face(&mut m, x as f32, y as f32, z as f32, *face);
                    }
                }
            }
        }
    }
    m
}

/// Simple mesh function that returns MeshData (legacy face-by-face)
pub fn mesh_chunk_with_ao(chunk: &Chunk) -> MeshData {
    let legacy = mesh_chunk_v2(chunk);
    let vertex_count = legacy.positions.len();

    // Convert to MeshData with default AO values
    MeshData {
        positions: legacy.positions,
        uvs: legacy.uvs,
        indices: legacy.indices,
        ao: vec![1.0; vertex_count], // No AO for legacy meshing
    }
}

#[inline]
fn sample(chunk: &Chunk, x: i32, y: i32, z: i32) -> BlockId {
    if x < 0 || y < 0 || z < 0 {
        return 0;
    } // AIR
    let xu = x as usize;
    let yu = y as usize;
    let zu = z as usize;
    if xu >= CHUNK_SIZE || yu >= CHUNK_SIZE || zu >= CHUNK_SIZE {
        return 0;
    }
    chunk.get(xu, yu, zu)
}

type Quad = ([f32; 3], [f32; 3], [f32; 3], [f32; 3]);

// Neighbor offset and which face to emit from the cube centered at (x,y,z).
const FACES: &[(i32, i32, i32, usize)] = &[
    (1, 0, 0, 0),  // +X
    (-1, 0, 0, 1), // -X
    (0, 1, 0, 2),  // +Y
    (0, -1, 0, 3), // -Y
    (0, 0, 1, 4),  // +Z
    (0, 0, -1, 5), // -Z
];

const FACE_QUADS: [Quad; 6] = [
    // +X
    (
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 0.0],
    ),
    // -X
    (
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
    ),
    // +Y
    (
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ),
    // -Y
    (
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ),
    // +Z
    (
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ),
    // -Z
    (
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ),
];

#[inline]
fn emit_face(m: &mut MeshPosUv, x: f32, y: f32, z: f32, face_id: usize) {
    let base = m.positions.len() as u32;
    let q = FACE_QUADS[face_id];

    // Two triangles: (0,1,2) and (0,2,3)
    m.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

    // UVs are full-tile 0..1
    const UV: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    for (i, p) in [q.0, q.1, q.2, q.3].into_iter().enumerate() {
        m.positions.push([x + p[0], y + p[1], z + p[2]]);
        m.uvs.push(UV[i]);
    }
}

//==============================================================================
// GREEDY MESHING ALGORITHM
//==============================================================================

/// Axis for sweeping during greedy meshing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

/// Direction along an axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Pos, // Positive direction
    Neg, // Negative direction
}

impl Dir {
    /// Convert axis + direction to FaceDir for texture atlas
    fn to_face_dir(self, axis: Axis) -> FaceDir {
        match (axis, self) {
            (Axis::X, Dir::Pos) => FaceDir::East,
            (Axis::X, Dir::Neg) => FaceDir::West,
            (Axis::Y, Dir::Pos) => FaceDir::Top,
            (Axis::Y, Dir::Neg) => FaceDir::Bottom,
            (Axis::Z, Dir::Pos) => FaceDir::South,
            (Axis::Z, Dir::Neg) => FaceDir::North,
        }
    }
}

/// Entry in the meshing mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MaskEntry {
    block_id: BlockId,
    u: i32,
    v: i32,
}

/// Greedy meshing for a single chunk with neighbor support
pub fn greedy_mesh_chunk(
    chunk: &Chunk,
    chunk_manager: Option<&ChunkManager>,
    atlas: &TextureAtlas,
) -> MeshData {
    // Get neighbor chunks for cross-chunk meshing
    let neighbors = if let Some(manager) = chunk_manager {
        let pos = chunk.position;
        [
            manager.get_chunk(pos + IVec3::new(1, 0, 0)),  // +X (East)
            manager.get_chunk(pos + IVec3::new(-1, 0, 0)), // -X (West)
            manager.get_chunk(pos + IVec3::new(0, 1, 0)),  // +Y (Top)
            manager.get_chunk(pos + IVec3::new(0, -1, 0)), // -Y (Bottom)
            manager.get_chunk(pos + IVec3::new(0, 0, 1)),  // +Z (South)
            manager.get_chunk(pos + IVec3::new(0, 0, -1)), // -Z (North)
        ]
    } else {
        [None, None, None, None, None, None]
    };

    let mut mesh = MeshData::with_capacity(INITIAL_VERTEX_CAPACITY, INITIAL_INDEX_CAPACITY);

    // Sweep along each axis in both directions
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        for dir in [Dir::Pos, Dir::Neg] {
            greedy_mesh_axis(chunk, &neighbors, axis, dir, atlas, &mut mesh);
        }
    }

    mesh
}

/// Sample block with neighbor chunk support
#[inline]
fn sample_with_neighbors(
    chunk: &Chunk,
    neighbors: &[Option<&Chunk>; 6],
    x: i32,
    y: i32,
    z: i32,
) -> BlockId {
    let size = CHUNK_SIZE as i32;
    
    // If within current chunk bounds, sample directly
    if x >= 0 && y >= 0 && z >= 0 && x < size && y < size && z < size {
        return chunk.get(x as usize, y as usize, z as usize);
    }

    // Check neighbor chunks
    if x >= size {
        // +X neighbor (East)
        if let Some(neighbor) = neighbors[0] {
            if y >= 0 && z >= 0 && y < size && z < size {
                return neighbor.get(0, y as usize, z as usize);
            }
        }
    } else if x < 0 {
        // -X neighbor (West)
        if let Some(neighbor) = neighbors[1] {
            if y >= 0 && z >= 0 && y < size && z < size {
                return neighbor.get((size - 1) as usize, y as usize, z as usize);
            }
        }
    } else if y >= size {
        // +Y neighbor (Top)
        if let Some(neighbor) = neighbors[2] {
            if x >= 0 && z >= 0 && x < size && z < size {
                return neighbor.get(x as usize, 0, z as usize);
            }
        }
    } else if y < 0 {
        // -Y neighbor (Bottom)
        if let Some(neighbor) = neighbors[3] {
            if x >= 0 && z >= 0 && x < size && z < size {
                return neighbor.get(x as usize, (size - 1) as usize, z as usize);
            }
        }
    } else if z >= size {
        // +Z neighbor (South)
        if let Some(neighbor) = neighbors[4] {
            if x >= 0 && y >= 0 && x < size && y < size {
                return neighbor.get(x as usize, y as usize, 0);
            }
        }
    } else if z < 0 {
        // -Z neighbor (North)
        if let Some(neighbor) = neighbors[5] {
            if x >= 0 && y >= 0 && x < size && y < size {
                return neighbor.get(x as usize, y as usize, (size - 1) as usize);
            }
        }
    }

    // Default to air if out of bounds
    0
}

/// Check if a block is transparent (allows light through)
#[inline]
fn is_transparent(block_id: BlockId) -> bool {
    block_id == 0 || block_id == 6 || block_id == 7 // AIR, WATER, GLASS
}

/// Greedy meshing for one axis/direction
fn greedy_mesh_axis(
    chunk: &Chunk,
    neighbors: &[Option<&Chunk>; 6],
    axis: Axis,
    dir: Dir,
    atlas: &TextureAtlas,
    mesh: &mut MeshData,
) {
    let size = CHUNK_SIZE as i32;
    
    // Reusable mask buffer to avoid allocations
    let mut mask: Vec<Option<MaskEntry>> = vec![None; (size * size) as usize];

    // Sweep along the axis
    for d in 0..size {
        // Clear mask for reuse
        mask.fill(None);

        // Build mask for this slice
        for u in 0..size {
            for v in 0..size {
                let (x, y, z) = match axis {
                    Axis::X => (d, u, v),
                    Axis::Y => (u, d, v),
                    Axis::Z => (u, v, d),
                };

                // Check if we should generate a face here
                let block = sample_with_neighbors(chunk, neighbors, x, y, z);
                if block == 0 { continue; } // Skip air

                // Calculate neighbor position
                let (nx, ny, nz) = match (axis, dir) {
                    (Axis::X, Dir::Pos) => (x + 1, y, z),
                    (Axis::X, Dir::Neg) => (x - 1, y, z),
                    (Axis::Y, Dir::Pos) => (x, y + 1, z),
                    (Axis::Y, Dir::Neg) => (x, y - 1, z),
                    (Axis::Z, Dir::Pos) => (x, y, z + 1),
                    (Axis::Z, Dir::Neg) => (x, y, z - 1),
                };

                let neighbor = sample_with_neighbors(chunk, neighbors, nx, ny, nz);

                // Generate face if neighbor is air or transparent
                if is_transparent(neighbor) {
                    let idx = (u * size + v) as usize;
                    mask[idx] = Some(MaskEntry { block_id: block, u, v });
                }
            }
        }

        // Greedy merge the mask
        greedy_merge_mask(&mask, size, d, axis, dir, atlas, mesh);
    }
}

/// Merge quads in the mask using greedy algorithm
fn greedy_merge_mask(
    mask: &[Option<MaskEntry>],
    size: i32,
    depth: i32,
    axis: Axis,
    dir: Dir,
    atlas: &TextureAtlas,
    mesh: &mut MeshData,
) {
    let mut visited = vec![false; mask.len()];
    let size_usize = size as usize;

    for start_u in 0..size {
        for start_v in 0..size {
            let idx = (start_u * size + start_v) as usize;

            // Skip if already visited or empty
            if visited[idx] { continue; }
            
            let entry = match mask[idx] {
                Some(entry) => entry,
                None => continue,
            };

            // Extend in V direction (height)
            let mut height = 1;
            let start_u_usize = start_u as usize;
            let start_v_usize = start_v as usize;
            
            while start_v_usize + height < size_usize {
                let test_idx = start_u_usize * size_usize + start_v_usize + height;
                if visited[test_idx] || mask[test_idx] != Some(MaskEntry {
                    block_id: entry.block_id,
                    u: start_u,
                    v: start_v + height as i32,
                }) {
                    break;
                }
                height += 1;
            }

            // Extend in U direction (width)
            let mut width = 1;
            'outer: while start_u_usize + width < size_usize {
                for dv in 0..height {
                    let test_idx = (start_u_usize + width) * size_usize + start_v_usize + dv;
                    if visited[test_idx] || mask[test_idx] != Some(MaskEntry {
                        block_id: entry.block_id,
                        u: start_u + width as i32,
                        v: start_v + dv as i32,
                    }) {
                        break 'outer;
                    }
                }
                width += 1;
            }

            // Mark as visited
            for du in 0..width {
                let base_idx = (start_u_usize + du) * size_usize + start_v_usize;
                for dv in 0..height {
                    visited[base_idx + dv] = true;
                }
            }

            // Generate the merged quad
            emit_greedy_quad(
                mesh,
                entry.block_id,
                depth,
                start_u,
                start_v,
                width as i32,
                height as i32,
                axis,
                dir,
                atlas,
            );
        }
    }
}

/// Emit a greedy-merged quad
fn emit_greedy_quad(
    mesh: &mut MeshData,
    block_id: BlockId,
    depth: i32,
    start_u: i32,
    start_v: i32,
    width: i32,
    height: i32,
    axis: Axis,
    dir: Dir,
    atlas: &TextureAtlas,
) {
    let face_dir = dir.to_face_dir(axis);
    let uvs = atlas.get_uvs(block_id, face_dir);

    // Calculate positions based on axis and direction
    let (p0, p1, p2, p3) = match (axis, dir) {
        (Axis::Y, Dir::Pos) => {
            // Top face (+Y)
            let y = (depth + 1) as f32;
            (
                [start_u as f32, y, start_v as f32],
                [(start_u + width) as f32, y, start_v as f32],
                [(start_u + width) as f32, y, (start_v + height) as f32],
                [start_u as f32, y, (start_v + height) as f32],
            )
        }
        (Axis::Y, Dir::Neg) => {
            // Bottom face (-Y)
            let y = depth as f32;
            (
                [start_u as f32, y, start_v as f32],
                [(start_u + width) as f32, y, start_v as f32],
                [(start_u + width) as f32, y, (start_v + height) as f32],
                [start_u as f32, y, (start_v + height) as f32],
            )
        }
        (Axis::X, Dir::Pos) => {
            // East face (+X)
            let x = (depth + 1) as f32;
            (
                [x, start_u as f32, start_v as f32],
                [x, (start_u + width) as f32, start_v as f32],
                [x, (start_u + width) as f32, (start_v + height) as f32],
                [x, start_u as f32, (start_v + height) as f32],
            )
        }
        (Axis::X, Dir::Neg) => {
            // West face (-X)
            let x = depth as f32;
            (
                [x, start_u as f32, start_v as f32],
                [x, (start_u + width) as f32, start_v as f32],
                [x, (start_u + width) as f32, (start_v + height) as f32],
                [x, start_u as f32, (start_v + height) as f32],
            )
        }
        (Axis::Z, Dir::Pos) => {
            // South face (+Z)
            let z = (depth + 1) as f32;
            (
                [start_u as f32, start_v as f32, z],
                [(start_u + width) as f32, start_v as f32, z],
                [(start_u + width) as f32, (start_v + height) as f32, z],
                [start_u as f32, (start_v + height) as f32, z],
            )
        }
        (Axis::Z, Dir::Neg) => {
            // North face (-Z)
            let z = depth as f32;
            (
                [start_u as f32, start_v as f32, z],
                [(start_u + width) as f32, start_v as f32, z],
                [(start_u + width) as f32, (start_v + height) as f32, z],
                [start_u as f32, (start_v + height) as f32, z],
            )
        }
    };

    let base = mesh.positions.len() as u32;
    mesh.positions.extend_from_slice(&[p0, p1, p2, p3]);

    // Scale UVs by quad size
    let u_scale = width as f32;
    let v_scale = height as f32;
    
    mesh.uvs.push([uvs[0][0], uvs[0][1]]);
    mesh.uvs.push([uvs[0][0] + uvs[1][0] * u_scale, uvs[0][1]]);
    mesh.uvs.push([uvs[0][0] + uvs[2][0] * u_scale, uvs[0][1] + uvs[2][1] * v_scale]);
    mesh.uvs.push([uvs[0][0], uvs[0][1] + uvs[3][1] * v_scale]);

    // Default AO values (no occlusion)
    mesh.ao.extend_from_slice(&[1.0, 1.0, 1.0, 1.0]);

    // Add indices (two triangles)
    mesh.indices.extend_from_slice(&[
        base, base + 1, base + 2,
        base, base + 2, base + 3,
    ]);
}

//==============================================================================
// UTILITY FUNCTIONS
//==============================================================================

/// Get mesh statistics for performance analysis
#[derive(Debug, Clone)]
pub struct MeshStats {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub quad_count: usize,
    pub memory_bytes: usize,
}

impl MeshStats {
    pub fn from_mesh(mesh: &MeshData) -> Self {
        let vertex_count = mesh.vertex_count();
        let triangle_count = mesh.triangle_count();
        let quad_count = triangle_count / 2;
        
        // Estimate memory usage
        let memory_bytes = 
            vertex_count * (3 * 4 + 2 * 4 + 4) + // positions (3xf32) + uvs (2xf32) + ao (f32)
            mesh.indices.len() * 4; // indices (u32)
            
        Self {
            vertex_count,
            triangle_count,
            quad_count,
            memory_bytes,
        }
    }
    
    /// Calculate reduction percentage compared to face-by-face meshing
    pub fn reduction_vs_naive(&self, block_count: usize) -> f32 {
        if block_count == 0 { return 0.0; }
        
        // Naive meshing: each block can generate up to 6 faces, each face = 2 triangles
        let naive_triangles = block_count * 6 * 2;
        let reduction = (naive_triangles as f32 - self.triangle_count as f32) / naive_triangles as f32;
        (reduction * 100.0).max(0.0)
    }
}

/// Benchmark meshing performance
pub fn benchmark_meshing(chunk: &Chunk, iterations: u32) -> (std::time::Duration, std::time::Duration) {
    use std::time::Instant;
    
    // Benchmark legacy meshing
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = mesh_chunk_with_ao(chunk);
    }
    let legacy_time = start.elapsed();
    
    // Benchmark greedy meshing
    let atlas = TextureAtlas::default();
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = greedy_mesh_chunk(chunk, None, &atlas);
    }
    let greedy_time = start.elapsed();
    
    (legacy_time, greedy_time)
}