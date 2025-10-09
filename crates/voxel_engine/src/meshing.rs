#![allow(clippy::many_single_char_names)]
use crate::chunk::{Chunk, BlockId, CHUNK_SIZE, ChunkManager};
use crate::atlas::{TextureAtlas, FaceDir};
use glam::IVec3;

/// Mesh data with position, UV, and AO
#[derive(Debug, Default, Clone)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub ao: Vec<f32>,           // Ambient occlusion per vertex (0..1)
    pub indices: Vec<u32>,
}

impl MeshData {
    /// Get statistics for this mesh
    pub fn stats(&self) -> MeshStats {
        MeshStats {
            vertex_count: self.positions.len(),
            triangle_count: self.indices.len() / 3,
            memory_bytes: self.memory_size(),
        }
    }
    
    /// Calculate memory usage
    pub fn memory_size(&self) -> usize {
        self.positions.len() * std::mem::size_of::<[f32; 3]>()
            + self.uvs.len() * std::mem::size_of::<[f32; 2]>()
            + self.ao.len() * std::mem::size_of::<f32>()
            + self.indices.len() * std::mem::size_of::<u32>()
    }
    
    /// Calculate axis-aligned bounding box
    pub fn calculate_aabb(&self) -> AABB {
        if self.positions.is_empty() {
            return AABB::default();
        }
        
        let mut min = glam::Vec3::splat(f32::MAX);
        let mut max = glam::Vec3::splat(f32::MIN);
        
        for pos in &self.positions {
            let p = glam::Vec3::from_array(*pos);
            min = min.min(p);
            max = max.max(p);
        }
        
        AABB { min, max }
    }
}

/// Axis-aligned bounding box
#[derive(Debug, Default, Clone, Copy)]
pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl AABB {
    pub fn center(&self) -> glam::Vec3 {
        (self.min + self.max) * 0.5
    }
    
    pub fn size(&self) -> glam::Vec3 {
        self.max - self.min
    }
}

/// Mesh statistics
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshStats {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub memory_bytes: usize,
}

/// Unified mesh build output for both block and density meshers
#[derive(Debug, Default, Clone)]
pub struct MeshBuildOutput {
    /// Vertex positions
    pub positions: Vec<[f32; 3]>,
    
    /// UV coordinates (for block meshes)
    pub uvs: Vec<[f32; 2]>,
    
    /// Normals (for density meshes)
    pub normals: Vec<[f32; 3]>,
    
    /// Ambient occlusion per vertex
    pub ao: Vec<f32>,
    
    /// Indices
    pub indices: Vec<u32>,
    
    /// Submesh ranges (for opaque/transparent separation)
    pub submeshes: Vec<SubmeshRange>,
    
    /// Axis-aligned bounding box
    pub aabb: AABB,
    
    /// Statistics
    pub stats: MeshStats,
}

/// Submesh range (for separating opaque/transparent geometry)
#[derive(Debug, Clone, Copy)]
pub struct SubmeshRange {
    pub start_index: u32,
    pub index_count: u32,
    pub material_type: MaterialType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialType {
    Opaque,
    Transparent,
}

impl MeshBuildOutput {
    /// Create from legacy MeshData (for block meshes)
    pub fn from_mesh_data(mesh: MeshData) -> Self {
        let aabb = mesh.calculate_aabb();
        let stats = mesh.stats();
        
        Self {
            positions: mesh.positions,
            uvs: mesh.uvs,
            normals: vec![], // No normals for block meshes
            ao: mesh.ao,
            indices: mesh.indices,
            submeshes: vec![SubmeshRange {
                start_index: 0,
                index_count: stats.triangle_count as u32 * 3,
                material_type: MaterialType::Opaque,
            }],
            aabb,
            stats,
        }
    }
    
    /// Create from separated meshes (opaque + transparent)
    pub fn from_separated_mesh(separated: SeparatedMesh) -> Self {
        let mut output = Self::default();
        
        let opaque_vert_count = separated.opaque.positions.len() as u32;
        let opaque_tri_count = (separated.opaque.indices.len() / 3) as u32;
        let _transparent_vert_count = separated.transparent.positions.len() as u32;
        let transparent_tri_count = (separated.transparent.indices.len() / 3) as u32;
        
        // Combine positions
        output.positions.extend_from_slice(&separated.opaque.positions);
        output.positions.extend_from_slice(&separated.transparent.positions);
        
        // Combine UVs
        output.uvs.extend_from_slice(&separated.opaque.uvs);
        output.uvs.extend_from_slice(&separated.transparent.uvs);
        
        // Combine AO
        output.ao.extend_from_slice(&separated.opaque.ao);
        output.ao.extend_from_slice(&separated.transparent.ao);
        
        // Combine indices (offset transparent indices)
        output.indices.extend_from_slice(&separated.opaque.indices);
        output.indices.extend(
            separated.transparent.indices.iter().map(|i| i + opaque_vert_count)
        );
        
        // Create submesh ranges
        output.submeshes = vec![
            SubmeshRange {
                start_index: 0,
                index_count: opaque_tri_count * 3,
                material_type: MaterialType::Opaque,
            },
            SubmeshRange {
                start_index: opaque_tri_count * 3,
                index_count: transparent_tri_count * 3,
                material_type: MaterialType::Transparent,
            },
        ];
        
        // Calculate AABB
        if !output.positions.is_empty() {
            let mut min = glam::Vec3::splat(f32::MAX);
            let mut max = glam::Vec3::splat(f32::MIN);
            
            for pos in &output.positions {
                let p = glam::Vec3::from_array(*pos);
                min = min.min(p);
                max = max.max(p);
            }
            
            output.aabb = AABB { min, max };
        }
        
        // Calculate stats
        output.stats = MeshStats {
            vertex_count: output.positions.len(),
            triangle_count: (output.indices.len() / 3),
            memory_bytes: output.positions.len() * std::mem::size_of::<[f32; 3]>()
                + output.uvs.len() * std::mem::size_of::<[f32; 2]>()
                + output.ao.len() * std::mem::size_of::<f32>()
                + output.indices.len() * std::mem::size_of::<u32>(),
        };
        
        output
    }
}

/// Legacy structure for compatibility
#[derive(Debug, Default)]
pub struct MeshPosUv {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// Legacy mesh function - simple face-by-face meshing (kept for compatibility)
pub fn mesh_chunk_v2(chunk: &Chunk) -> MeshPosUv {
    let mut m = MeshPosUv::default();
    m.positions.reserve(6 * 4 * 1024);
    m.uvs.reserve(6 * 4 * 1024);
    m.indices.reserve(6 * 6 * 1024);

    let size = CHUNK_SIZE as i32;

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                if sample(chunk, x, y, z) == 0 { continue; } // 0 = AIR

                // For each of the 6 directions, if neighbor is air, emit that face.
                for (nx, ny, nz, face) in FACES {
                    let ax = x + nx;
                    let ay = y + ny;
                    let az = z + nz;
                    if ax < 0 || ay < 0 || az < 0 || ax >= size || ay >= size || az >= size
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

/// Mesh function that returns MeshData (with AO support)
pub fn mesh_chunk_with_ao(chunk: &Chunk) -> MeshData {
    let legacy = mesh_chunk_v2(chunk);
    
    // Count positions before move
    let ao_count = legacy.positions.len();
    
    // Convert to MeshData with default AO (1.0 = fully lit)
    MeshData {
        positions: legacy.positions,
        uvs: legacy.uvs,
        ao: vec![1.0; ao_count],
        indices: legacy.indices,
    }
}

#[inline]
fn sample(chunk: &Chunk, x: i32, y: i32, z: i32) -> BlockId {
    if x < 0 || y < 0 || z < 0 { return 0; } // AIR
    let xu = x as usize;
    let yu = y as usize;
    let zu = z as usize;
    if xu >= CHUNK_SIZE || yu >= CHUNK_SIZE || zu >= CHUNK_SIZE { return 0; }
    chunk.get(xu, yu, zu)
}

type Quad = ([f32; 3], [f32; 3], [f32; 3], [f32; 3]);

// Neighbor offset and which face to emit from the cube centered at (x,y,z).
const FACES: &[(i32, i32, i32, usize)] = &[
    ( 1,  0,  0, 0), // +X
    (-1,  0,  0, 1), // -X
    ( 0,  1,  0, 2), // +Y
    ( 0, -1,  0, 3), // -Y
    ( 0,  0,  1, 4), // +Z
    ( 0,  0, -1, 5), // -Z
];

const FACE_QUADS: [Quad; 6] = [
    // +X
    ([1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]),
    // -X
    ([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0]),
    // +Y
    ([0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]),
    // -Y
    ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]),
    // +Z
    ([0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
    // -Z
    ([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]),
];

#[inline]
fn emit_face(m: &mut MeshPosUv, x: f32, y: f32, z: f32, face_id: usize) {
    let base = m.positions.len() as u32;
    let q = FACE_QUADS[face_id];

    // Two triangles: (0,1,2) and (0,2,3)
    m.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

    // UVs are full-tile 0..1
    const UV: [[f32; 2]; 4] = [[0.0, 0.0],[1.0, 0.0],[1.0, 1.0],[0.0, 1.0]];

    for (i, p) in [q.0, q.1, q.2, q.3].into_iter().enumerate() {
        m.positions.push([x + p[0], y + p[1], z + p[2]]);
        m.uvs.push(UV[i]);
    }
}

//==============================================================================
// GREEDY MESHING WITH AO
//==============================================================================

/// Axis for sweeping during greedy meshing
#[derive(Debug, Clone, Copy)]
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

/// Sample block with neighbor chunk support
fn sample_with_neighbors(
    chunk: &Chunk,
    neighbors: &[Option<&Chunk>; 6],
    x: i32,
    y: i32,
    z: i32,
) -> BlockId {
    let size = CHUNK_SIZE as i32;
    
    // Inside current chunk
    if x >= 0 && x < size && y >= 0 && y < size && z >= 0 && z < size {
        return chunk.get(x as usize, y as usize, z as usize);
    }
    
    // Check neighbors: [+X, -X, +Y, -Y, +Z, -Z]
    if x >= size && y >= 0 && y < size && z >= 0 && z < size {
        if let Some(n) = neighbors[0] {
            return n.get(0, y as usize, z as usize);
        }
    } else if x < 0 && y >= 0 && y < size && z >= 0 && z < size {
        if let Some(n) = neighbors[1] {
            return n.get((size - 1) as usize, y as usize, z as usize);
        }
    } else if y >= size && x >= 0 && x < size && z >= 0 && z < size {
        if let Some(n) = neighbors[2] {
            return n.get(x as usize, 0, z as usize);
        }
    } else if y < 0 && x >= 0 && x < size && z >= 0 && z < size {
        if let Some(n) = neighbors[3] {
            return n.get(x as usize, (size - 1) as usize, z as usize);
        }
    } else if z >= size && x >= 0 && x < size && y >= 0 && y < size {
        if let Some(n) = neighbors[4] {
            return n.get(x as usize, y as usize, 0);
        }
    } else if z < 0 && x >= 0 && x < size && y >= 0 && y < size {
        if let Some(n) = neighbors[5] {
            return n.get(x as usize, y as usize, (size - 1) as usize);
        }
    }
    
    // Out of all neighbor bounds
    0 // AIR
}

/// Calculate ambient occlusion for a vertex (4-tap)
/// Returns a value from 0.0 (fully occluded) to 1.0 (not occluded)
fn calculate_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    // Minecraft-style AO with 4-tap sampling
    if side1 && side2 {
        0.0 // Fully occluded (both sides block light)
    } else {
        let count = side1 as u8 + side2 as u8 + corner as u8;
        match count {
            0 => 1.0,   // No occlusion
            1 => 0.75,  // Light occlusion
            2 => 0.5,   // Medium occlusion
            _ => 0.25,  // Heavy occlusion
        }
    }
}

/// Calculate AO for all 4 vertices of a quad
/// Returns [ao0, ao1, ao2, ao3] for each corner
fn calculate_quad_ao(
    chunk: &Chunk,
    neighbors: &[Option<&Chunk>; 6],
    x: i32,
    y: i32,
    z: i32,
    axis: Axis,
    dir: Dir,
) -> [f32; 4] {
    // Get the 8 surrounding blocks for AO sampling
    let sample_ao = |dx: i32, dy: i32, dz: i32| -> bool {
        let block = sample_with_neighbors(chunk, neighbors, x + dx, y + dy, z + dz);
        block != 0 && !is_transparent(block)
    };
    
    match (axis, dir) {
        (Axis::Y, Dir::Pos) => {
            // Top face (+Y) - looking down at face
            // Corners: (0,0), (1,0), (1,1), (0,1)
            let v0 = calculate_ao(
                sample_ao(-1, 1, 0), sample_ao(0, 1, -1), sample_ao(-1, 1, -1)
            );
            let v1 = calculate_ao(
                sample_ao(1, 1, 0), sample_ao(0, 1, -1), sample_ao(1, 1, -1)
            );
            let v2 = calculate_ao(
                sample_ao(1, 1, 0), sample_ao(0, 1, 1), sample_ao(1, 1, 1)
            );
            let v3 = calculate_ao(
                sample_ao(-1, 1, 0), sample_ao(0, 1, 1), sample_ao(-1, 1, 1)
            );
            [v0, v1, v2, v3]
        }
        (Axis::Y, Dir::Neg) => {
            // Bottom face (-Y)
            let v0 = calculate_ao(
                sample_ao(-1, -1, 0), sample_ao(0, -1, -1), sample_ao(-1, -1, -1)
            );
            let v1 = calculate_ao(
                sample_ao(1, -1, 0), sample_ao(0, -1, -1), sample_ao(1, -1, -1)
            );
            let v2 = calculate_ao(
                sample_ao(1, -1, 0), sample_ao(0, -1, 1), sample_ao(1, -1, 1)
            );
            let v3 = calculate_ao(
                sample_ao(-1, -1, 0), sample_ao(0, -1, 1), sample_ao(-1, -1, 1)
            );
            [v0, v1, v2, v3]
        }
        (Axis::X, Dir::Pos) => {
            // East face (+X)
            let v0 = calculate_ao(
                sample_ao(1, -1, 0), sample_ao(1, 0, -1), sample_ao(1, -1, -1)
            );
            let v1 = calculate_ao(
                sample_ao(1, -1, 0), sample_ao(1, 0, 1), sample_ao(1, -1, 1)
            );
            let v2 = calculate_ao(
                sample_ao(1, 1, 0), sample_ao(1, 0, 1), sample_ao(1, 1, 1)
            );
            let v3 = calculate_ao(
                sample_ao(1, 1, 0), sample_ao(1, 0, -1), sample_ao(1, 1, -1)
            );
            [v0, v1, v2, v3]
        }
        (Axis::X, Dir::Neg) => {
            // West face (-X)
            let v0 = calculate_ao(
                sample_ao(-1, -1, 0), sample_ao(-1, 0, -1), sample_ao(-1, -1, -1)
            );
            let v1 = calculate_ao(
                sample_ao(-1, -1, 0), sample_ao(-1, 0, 1), sample_ao(-1, -1, 1)
            );
            let v2 = calculate_ao(
                sample_ao(-1, 1, 0), sample_ao(-1, 0, 1), sample_ao(-1, 1, 1)
            );
            let v3 = calculate_ao(
                sample_ao(-1, 1, 0), sample_ao(-1, 0, -1), sample_ao(-1, 1, -1)
            );
            [v0, v1, v2, v3]
        }
        (Axis::Z, Dir::Pos) => {
            // South face (+Z)
            let v0 = calculate_ao(
                sample_ao(-1, 0, 1), sample_ao(0, -1, 1), sample_ao(-1, -1, 1)
            );
            let v1 = calculate_ao(
                sample_ao(1, 0, 1), sample_ao(0, -1, 1), sample_ao(1, -1, 1)
            );
            let v2 = calculate_ao(
                sample_ao(1, 0, 1), sample_ao(0, 1, 1), sample_ao(1, 1, 1)
            );
            let v3 = calculate_ao(
                sample_ao(-1, 0, 1), sample_ao(0, 1, 1), sample_ao(-1, 1, 1)
            );
            [v0, v1, v2, v3]
        }
        (Axis::Z, Dir::Neg) => {
            // North face (-Z)
            let v0 = calculate_ao(
                sample_ao(-1, 0, -1), sample_ao(0, -1, -1), sample_ao(-1, -1, -1)
            );
            let v1 = calculate_ao(
                sample_ao(1, 0, -1), sample_ao(0, -1, -1), sample_ao(1, -1, -1)
            );
            let v2 = calculate_ao(
                sample_ao(1, 0, -1), sample_ao(0, 1, -1), sample_ao(1, 1, -1)
            );
            let v3 = calculate_ao(
                sample_ao(-1, 0, -1), sample_ao(0, 1, -1), sample_ao(-1, 1, -1)
            );
            [v0, v1, v2, v3]
        }
    }
}

/// Greedy meshing for a single chunk with neighbor support
pub fn greedy_mesh_chunk(
    chunk: &Chunk,
    chunk_manager: Option<&ChunkManager>,
    atlas: &TextureAtlas,
) -> MeshData {
    // Get neighbor chunks
    let neighbors = if let Some(manager) = chunk_manager {
        let pos = chunk.position;
        [
            manager.get_chunk(pos + IVec3::new(1, 0, 0)),   // +X
            manager.get_chunk(pos + IVec3::new(-1, 0, 0)),  // -X
            manager.get_chunk(pos + IVec3::new(0, 1, 0)),   // +Y
            manager.get_chunk(pos + IVec3::new(0, -1, 0)),  // -Y
            manager.get_chunk(pos + IVec3::new(0, 0, 1)),   // +Z
            manager.get_chunk(pos + IVec3::new(0, 0, -1)),  // -Z
        ]
    } else {
        [None, None, None, None, None, None]
    };
    
    let mut mesh = MeshData::default();
    mesh.positions.reserve(4096);
    mesh.uvs.reserve(4096);
    mesh.ao.reserve(4096);
    mesh.indices.reserve(6144);
    
    // Sweep along each axis in both directions
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        for dir in [Dir::Pos, Dir::Neg] {
            greedy_mesh_axis(chunk, &neighbors, axis, dir, atlas, &mut mesh);
        }
    }
    
    mesh
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
    
    // Mask for tracking which faces to generate
    let mut mask = vec![None; (size * size) as usize];
    
    // Sweep along the axis
    for d in 0..size {
        // Clear mask
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
                if block == 0 { continue; } // AIR
                
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
                if neighbor == 0 || is_transparent(neighbor) {
                    let idx = (u * size + v) as usize;
                    mask[idx] = Some((block, u, v));
                }
            }
        }
        
        // Greedy mesh the mask
        greedy_merge_mask(&mask, size, d, axis, dir, atlas, mesh);
    }
}

/// Check if a block is transparent
fn is_transparent(block_id: BlockId) -> bool {
    block_id == 6 || block_id == 7 // Water or Glass
}

/// Merge quads in the mask using greedy algorithm
fn greedy_merge_mask(
    mask: &[Option<(BlockId, i32, i32)>],
    size: i32,
    depth: i32,
    axis: Axis,
    dir: Dir,
    atlas: &TextureAtlas,
    mesh: &mut MeshData,
) {
    let mut visited = vec![false; mask.len()];
    
    for start_u in 0..size {
        for start_v in 0..size {
            let idx = (start_u * size + start_v) as usize;
            
            if visited[idx] || mask[idx].is_none() {
                continue;
            }
            
            let (block_id, _, _) = mask[idx].unwrap();
            
            // Extend in V direction
            let mut height = 1;
            while start_v + height < size {
                let test_idx = (start_u * size + start_v + height) as usize;
                if visited[test_idx] || mask[test_idx] != Some((block_id, start_u, start_v + height)) {
                    break;
                }
                height += 1;
            }
            
            // Extend in U direction
            let mut width = 1;
            'outer: while start_u + width < size {
                for dv in 0..height {
                    let test_idx = ((start_u + width) * size + start_v + dv) as usize;
                    if visited[test_idx] || mask[test_idx] != Some((block_id, start_u + width, start_v + dv)) {
                        break 'outer;
                    }
                }
                width += 1;
            }
            
            // Mark as visited
            for du in 0..width {
                for dv in 0..height {
                    let mark_idx = ((start_u + du) * size + start_v + dv) as usize;
                    visited[mark_idx] = true;
                }
            }
            
            // Generate the merged quad
            emit_greedy_quad(
                mesh,
                block_id,
                depth,
                start_u,
                start_v,
                width,
                height,
                axis,
                dir,
                atlas,
            );
        }
    }
}

/// Emit a greedy-merged quad with configurable AO
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
    emit_greedy_quad_with_ao(mesh, block_id, depth, start_u, start_v, width, height, axis, dir, atlas, None, None)
}

/// Emit a greedy-merged quad with optional AO calculation
fn emit_greedy_quad_with_ao(
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
    chunk: Option<&Chunk>,
    neighbors: Option<&[Option<&Chunk>; 6]>,
) {
    let face_dir = dir.to_face_dir(axis);
    let uvs = atlas.get_uvs(block_id, face_dir);
    
    // Calculate positions
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
                [x, start_u as f32, (start_v + height) as f32],
                [x, (start_u + width) as f32, (start_v + height) as f32],
                [x, (start_u + width) as f32, start_v as f32],
            )
        }
        (Axis::X, Dir::Neg) => {
            // West face (-X)
            let x = depth as f32;
            (
                [x, start_u as f32, start_v as f32],
                [x, start_u as f32, (start_v + height) as f32],
                [x, (start_u + width) as f32, (start_v + height) as f32],
                [x, (start_u + width) as f32, start_v as f32],
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
    
    // Calculate AO if chunk data is available (configurable)
    let ao_values = if let (Some(c), Some(n)) = (chunk, neighbors) {
        // Calculate AO for the first voxel of this quad
        let (x, y, z) = match axis {
            Axis::X => (depth, start_u, start_v),
            Axis::Y => (start_u, depth, start_v),
            Axis::Z => (start_u, start_v, depth),
        };
        calculate_quad_ao(c, n, x, y, z, axis, dir)
    } else {
        // No AO (fully lit)
        [1.0, 1.0, 1.0, 1.0]
    };
    
    // Add vertices
    let base = mesh.positions.len() as u32;
    mesh.positions.extend_from_slice(&[p0, p1, p2, p3]);
    
    // Scale UVs by quad size
    let u_scale = width as f32;
    let v_scale = height as f32;
    mesh.uvs.push([uvs[0][0], uvs[0][1]]);
    mesh.uvs.push([uvs[0][0] + uvs[1][0] * u_scale, uvs[0][1]]);
    mesh.uvs.push([uvs[0][0] + uvs[2][0] * u_scale, uvs[0][1] + uvs[2][1] * v_scale]);
    mesh.uvs.push([uvs[0][0], uvs[0][1] + uvs[3][1] * v_scale]);
    
    mesh.ao.extend_from_slice(&ao_values);
    
    // Add indices (two triangles)
    mesh.indices.extend_from_slice(&[
        base, base + 1, base + 2,
        base, base + 2, base + 3,
    ]);
}

//==============================================================================
// TRANSPARENCY SYSTEM
//==============================================================================

/// Separate meshes for opaque and transparent blocks
#[derive(Debug, Default)]
pub struct SeparatedMesh {
    pub opaque: MeshData,
    pub transparent: MeshData,
}

/// Generate meshes with opaque/transparent separation
pub fn greedy_mesh_chunk_separated(
    chunk: &Chunk,
    chunk_manager: Option<&ChunkManager>,
    atlas: &TextureAtlas,
) -> SeparatedMesh {
    let mut opaque = MeshData::default();
    let mut transparent = MeshData::default();
    
    opaque.positions.reserve(4096);
    opaque.uvs.reserve(4096);
    opaque.ao.reserve(4096);
    opaque.indices.reserve(6144);
    
    transparent.positions.reserve(512);
    transparent.uvs.reserve(512);
    transparent.ao.reserve(512);
    transparent.indices.reserve(768);
    
    // Get neighbor chunks
    let neighbors = if let Some(manager) = chunk_manager {
        let pos = chunk.position;
        [
            manager.get_chunk(pos + IVec3::new(1, 0, 0)),
            manager.get_chunk(pos + IVec3::new(-1, 0, 0)),
            manager.get_chunk(pos + IVec3::new(0, 1, 0)),
            manager.get_chunk(pos + IVec3::new(0, -1, 0)),
            manager.get_chunk(pos + IVec3::new(0, 0, 1)),
            manager.get_chunk(pos + IVec3::new(0, 0, -1)),
        ]
    } else {
        [None, None, None, None, None, None]
    };
    
    let size = CHUNK_SIZE as i32;
    
    // Process each block
    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let block = chunk.get(x as usize, y as usize, z as usize);
                if block == 0 { continue; } // Skip AIR
                
                let mesh = if is_transparent(block) {
                    &mut transparent
                } else {
                    &mut opaque
                };
                
                // Check 6 faces
                for axis in [Axis::X, Axis::Y, Axis::Z] {
                    for dir in [Dir::Pos, Dir::Neg] {
                        let (nx, ny, nz) = match (axis, dir) {
                            (Axis::X, Dir::Pos) => (x + 1, y, z),
                            (Axis::X, Dir::Neg) => (x - 1, y, z),
                            (Axis::Y, Dir::Pos) => (x, y + 1, z),
                            (Axis::Y, Dir::Neg) => (x, y - 1, z),
                            (Axis::Z, Dir::Pos) => (x, y, z + 1),
                            (Axis::Z, Dir::Neg) => (x, y, z - 1),
                        };
                        
                        let neighbor = sample_with_neighbors(chunk, &neighbors, nx, ny, nz);
                        
                        // Render face if neighbor is air or different transparency
                        let should_render = neighbor == 0 || 
                            (is_transparent(block) != is_transparent(neighbor));
                        
                        if should_render {
                            emit_simple_face(
                                mesh,
                                x as f32,
                                y as f32,
                                z as f32,
                                block,
                                axis,
                                dir,
                                atlas,
                            );
                        }
                    }
                }
            }
        }
    }
    
    SeparatedMesh { opaque, transparent }
}

/// Emit a single face (for transparency rendering)
fn emit_simple_face(
    mesh: &mut MeshData,
    x: f32,
    y: f32,
    z: f32,
    block_id: BlockId,
    axis: Axis,
    dir: Dir,
    atlas: &TextureAtlas,
) {
    let face_dir = dir.to_face_dir(axis);
    let uvs = atlas.get_uvs(block_id, face_dir);
    
    let (p0, p1, p2, p3) = match (axis, dir) {
        (Axis::Y, Dir::Pos) => (
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
        ),
        (Axis::Y, Dir::Neg) => (
            [x, y, z],
            [x + 1.0, y, z],
            [x + 1.0, y, z + 1.0],
            [x, y, z + 1.0],
        ),
        (Axis::X, Dir::Pos) => (
            [x + 1.0, y, z],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z],
        ),
        (Axis::X, Dir::Neg) => (
            [x, y, z],
            [x, y, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x, y + 1.0, z],
        ),
        (Axis::Z, Dir::Pos) => (
            [x, y, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
        ),
        (Axis::Z, Dir::Neg) => (
            [x, y, z],
            [x + 1.0, y, z],
            [x + 1.0, y + 1.0, z],
            [x, y + 1.0, z],
        ),
    };
    
    let base = mesh.positions.len() as u32;
    mesh.positions.extend_from_slice(&[p0, p1, p2, p3]);
    mesh.uvs.extend_from_slice(&uvs);
    mesh.ao.extend_from_slice(&[1.0, 1.0, 1.0, 1.0]);
    
    mesh.indices.extend_from_slice(&[
        base, base + 1, base + 2,
        base, base + 2, base + 3,
    ]);
}
