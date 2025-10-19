# README_aoi_system.md

## Purpose
Compute the **physical Area Of Interest (AOI)** per player for streaming and culling. AOI decides what is *loaded physically*, not what data is visible. Knowledge will filter fields later.

## Location
Place this file at:
`crates/game_core/crates/game_objects/README_aoi_system.md`

## Files to implement
- `crates/game_core/crates/game_objects/src/aoi.rs`
- Touch points in:
  - `.../src/world.rs` (query entry points)
  - `.../src/entities.rs` (positions)
  - `.../src/grids.rs` (grid bounds / cell occupancy)

## Public API (target)
- `pub struct Aoi;`
- `impl Aoi {`
  - `pub fn query_sphere(world: &World, center: Vec3, radius_m: f32, out: &mut SmallVec<[EntityId; 128]>)`
  - `pub fn query_cone(world: &World, eye: Vec3, dir: Vec3, fov_deg: f32, far_m: f32, out: &mut SmallVec<[EntityId; 128]>)`
  - `pub fn rebuild_shard(world: &World, shard_id: u32)`
  - `pub fn swap(&mut self)` // double-buffer index if needed
`}`

## Data sources
- `entities.rs` for positions and AABB
- `grids.rs` for grid-space indices and block bounds
- `mapping.rs` / `physics.rs` if you already have AABB helpers

## Steps
1. **Cell hashing**: add a simple 3D grid hash (cell size e.g. 64–128 m) mapping `cell -> SmallVec<EntityId>`.
2. **Shard rebuild**: per shard, rebuild cell occupancy from world state.
3. **Queries**:
   - Sphere: iterate candidate cells in bounding cube, distance check.
   - Cone: sphere prefilter, then dot(dir, to_entity) threshold.
4. **Integration**:
   - Expose `world.aoi_query_sphere(player_pos, 10_000.0)` in `world.rs` for convenience.
   - Do not apply Knowledge here.

## Tests
- Edge cases at radius boundary.
- Teleport: rebuild places entity in correct cells.
- Performance: cardinality × distance sweep.