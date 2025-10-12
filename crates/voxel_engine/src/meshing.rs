//! Simple voxel mesh generation
//! 
//! Uses face-by-face meshing algorithm that is simple, reliable, and fast enough.

use crate::chunk::{BlockId, Chunk, CHUNK_SIZE};

/// Simple mesh data with positions, UVs, and indices
#[derive(Debug, Default, Clone)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
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

/// Simple mesh function that returns MeshData
pub fn mesh_chunk_with_ao(chunk: &Chunk) -> MeshData {
    let legacy = mesh_chunk_v2(chunk);

    // Convert to MeshData
    MeshData {
        positions: legacy.positions,
        uvs: legacy.uvs,
        indices: legacy.indices,
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