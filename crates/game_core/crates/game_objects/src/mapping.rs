// mapping.rs — sparse index; movement via SpatialUpdater; AOI split in 3 stages
use crate::entities::Entity;
use crate::physics::{IntPosition, SpatialUpdater};
use crate::utils::arena::Arena;
use std::collections::{HashMap, HashSet};

pub const CELL_SIZE: i32 = 512;
pub const CHUNK_CELLS: i32 = 16;
pub const REGION_CHUNKS: i32 = 16;

pub const CHUNK_SIZE: i32 = CELL_SIZE * CHUNK_CELLS;
pub const REGION_SIZE: i32 = CHUNK_SIZE * REGION_CHUNKS;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CellId(pub i32, pub i32, pub i32);
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ChunkId(pub i32, pub i32, pub i32);
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RegionId(pub i32, pub i32, pub i32);

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
pub fn pos_to_cell(p: IntPosition) -> CellId {
    CellId(
        div_floor(p.x, CELL_SIZE),
        div_floor(p.y, CELL_SIZE),
        div_floor(p.z, CELL_SIZE),
    )
}
#[inline]
pub fn cell_to_chunk(c: CellId) -> ChunkId {
    ChunkId(
        div_floor(c.0, CHUNK_CELLS),
        div_floor(c.1, CHUNK_CELLS),
        div_floor(c.2, CHUNK_CELLS),
    )
}
#[inline]
pub fn chunk_to_region(k: ChunkId) -> RegionId {
    RegionId(
        div_floor(k.0, REGION_CHUNKS),
        div_floor(k.1, REGION_CHUNKS),
        div_floor(k.2, REGION_CHUNKS),
    )
}

#[derive(Copy, Clone, Debug)]
pub struct EntitySpatial {
    pub cell: CellId,
    pub chunk: ChunkId,
    pub region: RegionId,
}

pub struct SpatialIndex {
    pub entities: Arena<Entity, u32>,
    pub cells: HashMap<CellId, Vec<u32>>,
    pub chunks: HashMap<ChunkId, Vec<u32>>,
    pub regions: HashMap<RegionId, Vec<u32>>,
    pub e_spatials: HashMap<u32, EntitySpatial>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            entities: Arena::new(),
            cells: HashMap::new(),
            chunks: HashMap::new(),
            regions: HashMap::new(),
            e_spatials: HashMap::new(),
        }
    }

    pub fn insert_entity(&mut self, e: Entity) -> u32 {
        let id = self.entities.insert(e);
        let p = self
            .entities
            .get(id)
            .unwrap()
            .get_position()
            .to_int_position();
        let c0 = pos_to_cell(p);
        let k0 = cell_to_chunk(c0);
        let r0 = chunk_to_region(k0);
        self.cells.entry(c0).or_default().push(id);
        self.chunks.entry(k0).or_default().push(id);
        self.regions.entry(r0).or_default().push(id);
        self.e_spatials.insert(
            id,
            EntitySpatial {
                cell: c0,
                chunk: k0,
                region: r0,
            },
        );
        id
    }

    pub fn remove_entity(&mut self, id: u32) -> Option<Entity> {
        if let Some(sp) = self.e_spatials.remove(&id) {
            remove_id(&mut self.cells, sp.cell, id);
            remove_id(&mut self.chunks, sp.chunk, id);
            remove_id(&mut self.regions, sp.region, id);
        }
        self.entities.remove(id)
    }

    #[inline]
    pub fn tags_of(&self, id: u32) -> Option<EntitySpatial> {
        self.e_spatials.get(&id).copied()
    }

    /* -------- AOI split functions -------- */

    /// 1) Regions within radius that contain entities.
    pub fn query_regions_with_entities_in_radius(
        &self,
        center: IntPosition,
        radius: i32,
    ) -> Vec<RegionId> {
        let crx = div_floor(center.x, REGION_SIZE);
        let cry = div_floor(center.y, REGION_SIZE);
        let crz = div_floor(center.z, REGION_SIZE);
        let rr = (radius + REGION_SIZE - 1) / REGION_SIZE;

        let mut out = Vec::new();
        for rx in (crx - rr)..=(crx + rr) {
            for ry in (cry - rr)..=(cry + rr) {
                for rz in (crz - rr)..=(crz + rr) {
                    let rid = RegionId(rx, ry, rz);
                    if let Some(list) = self.regions.get(&rid) {
                        if !list.is_empty()
                            && aabb_sphere_intersect(center.clone(), radius, region_bounds(rid))
                        {
                            out.push(rid);
                        }
                    }
                }
            }
        }
        out
    }

    /// 2) Chunks within radius and inside provided regions that contain entities.
    pub fn query_chunks_with_entities_in_radius(
        &self,
        center: IntPosition,
        radius: i32,
        regions: &[RegionId],
    ) -> Vec<ChunkId> {
        let ccx = div_floor(center.x, CHUNK_SIZE);
        let ccy = div_floor(center.y, CHUNK_SIZE);
        let ccz = div_floor(center.z, CHUNK_SIZE);
        let rc = (radius + CHUNK_SIZE - 1) / CHUNK_SIZE;

        let mut out = Vec::new();
        for &RegionId(rx, ry, rz) in regions {
            let kx0 = rx * REGION_CHUNKS;
            let ky0 = ry * REGION_CHUNKS;
            let kz0 = rz * REGION_CHUNKS;
            for kx in (ccx - rc).max(kx0)..=((ccx + rc).min(kx0 + REGION_CHUNKS - 1)) {
                for ky in (ccy - rc).max(ky0)..=((ccy + rc).min(ky0 + REGION_CHUNKS - 1)) {
                    for kz in (ccz - rc).max(kz0)..=((kz0 + REGION_CHUNKS - 1).min(ccz + rc)) {
                        let kid = ChunkId(kx, ky, kz);
                        if let Some(list) = self.chunks.get(&kid) {
                            if !list.is_empty()
                                && aabb_sphere_intersect(center.clone(), radius, chunk_bounds(kid))
                            {
                                out.push(kid);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// 3) Cells within radius and inside provided chunks that contain entities.
    /// Returns both cell ids and the deduped entity list inside them.
    pub fn query_cells_with_entities_in_radius(
        &self,
        center: IntPosition,
        radius: i32,
        chunks: &[ChunkId],
    ) -> (Vec<CellId>, Vec<u32>) {
        let clx = div_floor(center.x, CELL_SIZE);
        let cly = div_floor(center.y, CELL_SIZE);
        let clz = div_floor(center.z, CELL_SIZE);
        let rcell = (radius + CELL_SIZE - 1) / CELL_SIZE;

        let mut cells_out = Vec::new();
        let mut ents_out = Vec::new();
        let mut seen = HashSet::new();

        for &ChunkId(kx, ky, kz) in chunks {
            let cx0 = kx * CHUNK_CELLS;
            let cy0 = ky * CHUNK_CELLS;
            let cz0 = kz * CHUNK_CELLS;

            for cx in (clx - rcell).max(cx0)..=((clx + rcell).min(cx0 + CHUNK_CELLS - 1)) {
                for cy in (cly - rcell).max(cy0)..=((cly + rcell).min(cy0 + CHUNK_CELLS - 1)) {
                    for cz in (clz - rcell).max(cz0)..=((cz0 + CHUNK_CELLS - 1).min(clz + rcell)) {
                        let cid = CellId(cx, cy, cz);
                        if let Some(clist) = self.cells.get(&cid) {
                            if !clist.is_empty()
                                && aabb_sphere_intersect(center.clone(), radius, cell_bounds(cid))
                            {
                                cells_out.push(cid);
                                for &eid in clist {
                                    if seen.insert(eid) {
                                        ents_out.push(eid);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        (cells_out, ents_out)
    }
}

/* -------- SpatialUpdater implementation -------- */

impl SpatialUpdater for SpatialIndex {
    fn update_on_move(&mut self, id: u32, old_pos: IntPosition, new_pos: IntPosition) {
        let old_cell = pos_to_cell(old_pos);
        let old_chunk = cell_to_chunk(old_cell);
        let old_reg = chunk_to_region(old_chunk);

        let new_cell = pos_to_cell(new_pos);
        let new_chunk = cell_to_chunk(new_cell);
        let new_reg = chunk_to_region(new_chunk);

        if new_cell != old_cell {
            remove_id(&mut self.cells, old_cell, id);
            self.cells.entry(new_cell).or_default().push(id);
        }
        if new_chunk != old_chunk {
            remove_id(&mut self.chunks, old_chunk, id);
            self.chunks.entry(new_chunk).or_default().push(id);
        }
        if new_reg != old_reg {
            remove_id(&mut self.regions, old_reg, id);
            self.regions.entry(new_reg).or_default().push(id);
        }
        self.e_spatials.insert(
            id,
            EntitySpatial {
                cell: new_cell,
                chunk: new_chunk,
                region: new_reg,
            },
        );
    }
}

/* -------- Utils -------- */

fn remove_id<K: Eq + std::hash::Hash>(map: &mut HashMap<K, Vec<u32>>, key: K, id: u32) {
    if let Some(v) = map.get_mut(&key) {
        if let Some(i) = v.iter().position(|&e| e == id) {
            v.swap_remove(i);
        }
        if v.is_empty() {
            map.remove(&key);
        }
    }
}

/* ---- AABB vs sphere helpers ---- */

#[inline]
fn region_bounds(r: RegionId) -> ((i32, i32, i32), (i32, i32, i32)) {
    let (x0, y0, z0) = (r.0 * REGION_SIZE, r.1 * REGION_SIZE, r.2 * REGION_SIZE);
    (
        (x0, y0, z0),
        (x0 + REGION_SIZE, y0 + REGION_SIZE, z0 + REGION_SIZE),
    )
}
#[inline]
fn chunk_bounds(k: ChunkId) -> ((i32, i32, i32), (i32, i32, i32)) {
    let (x0, y0, z0) = (k.0 * CHUNK_SIZE, k.1 * CHUNK_SIZE, k.2 * CHUNK_SIZE);
    (
        (x0, y0, z0),
        (x0 + CHUNK_SIZE, y0 + CHUNK_SIZE, z0 + CHUNK_SIZE),
    )
}
#[inline]
fn cell_bounds(c: CellId) -> ((i32, i32, i32), (i32, i32, i32)) {
    let (x0, y0, z0) = (c.0 * CELL_SIZE, c.1 * CELL_SIZE, c.2 * CELL_SIZE);
    (
        (x0, y0, z0),
        (x0 + CELL_SIZE, y0 + CELL_SIZE, z0 + CELL_SIZE),
    )
}

fn aabb_sphere_intersect(
    center: IntPosition,
    radius: i32,
    bounds: ((i32, i32, i32), (i32, i32, i32)),
) -> bool {
    let (minb, maxb) = bounds;
    let px = clamp(center.x, minb.0, maxb.0);
    let py = clamp(center.y, minb.1, maxb.1);
    let pz = clamp(center.z, minb.2, maxb.2);
    let dx = (center.x - px) as i64;
    let dy = (center.y - py) as i64;
    let dz = (center.z - pz) as i64;
    let dist2 = dx * dx + dy * dy + dz * dz;
    dist2 <= (radius as i64) * (radius as i64)
}
#[inline]
fn clamp(v: i32, lo: i32, hi: i32) -> i32 {
    if v < lo {
        lo
    } else if v > hi {
        v
    } else {
        v
    }
}
