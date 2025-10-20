use crate::physics::IntPosition;
use crate::utils::ids::{CellId, ChunkId, MapId, RegionId};

pub const CELL_SIZE: i32 = 512;
pub const CHUNK_CELLS: i32 = 16;
pub const REGION_CHUNKS: i32 = 16;

pub const CHUNK_SIZE: i32 = CELL_SIZE * CHUNK_CELLS;
pub const REGION_SIZE: i32 = CHUNK_SIZE * REGION_CHUNKS;

#[inline]
fn div_floor(a: i32, b: i32) -> i32 {
    let d = a / b;
    let r = a % b;
    if r != 0 && (r < 0) != (b < 0) {
        d - 1
    } else {
        d
    }
}

#[inline]
pub fn pos_to_cell(p: &IntPosition) -> CellId {
    CellId(
        div_floor(p.x, CELL_SIZE),
        div_floor(p.y, CELL_SIZE),
        div_floor(p.z, CELL_SIZE),
    )
}

pub fn pos_to_chunk(p: &IntPosition) -> ChunkId {
    ChunkId(
        div_floor(p.x, CHUNK_SIZE),
        div_floor(p.y, CHUNK_SIZE),
        div_floor(p.z, CHUNK_SIZE),
    )
}

pub fn pos_to_region(p: &IntPosition) -> RegionId {
    RegionId(
        div_floor(p.x, REGION_SIZE),
        div_floor(p.y, REGION_SIZE),
        div_floor(p.z, REGION_SIZE),
    )
}

pub fn pos_to_map_id(ip: &IntPosition, m: &mut MapId) {
    let cell_id = pos_to_cell(ip);
    if m.cell_id == cell_id {
        return;
    }
    m.cell_id = cell_id;

    let chunk_id = pos_to_chunk(ip);
    if m.chunk_id == chunk_id {
        return;
    }
    m.chunk_id = chunk_id;

    m.region_id = pos_to_region(ip);
}
