# Coordinate Systems and Transformations

## World Units
- **Base unit**: 1.0 = 1 meter in physical space
- **Chunk size**: 32³ voxels = 32 meters per side
- **Voxel size**: 1 voxel = 1 meter

## Coordinate Spaces

### 1. World Coordinates (IVec3)
- **Description**: Global 3D position in the infinite world
- **Range**: Theoretically unbounded (-∞ to +∞)
- **Units**: Meters (integer)
- **Example**: `IVec3(100, 50, -200)` = position at (100m, 50m, -200m)

### 2. Chunk Coordinates (IVec3)
- **Description**: Which chunk a position belongs to
- **Range**: Unbounded
- **Units**: Chunks (1 chunk = 32 meters)
- **Example**: `IVec3(3, 1, -6)` = chunk at column 3, layer 1, row -6

### 3. Local Voxel Coordinates (usize)
- **Description**: Position within a single chunk
- **Range**: 0..31 (inclusive) for each axis
- **Units**: Voxels (1 voxel = 1 meter)
- **Example**: `(15, 20, 8)` = voxel in the middle of chunk

## Conversion Formulas

### World → Chunk
```rust
fn world_to_chunk(world_pos: IVec3) -> IVec3 {
    IVec3::new(
        world_pos.x.div_euclid(32),
        world_pos.y.div_euclid(32),
        world_pos.z.div_euclid(32),
    )
}
```
**Note**: Uses `div_euclid` to handle negatives correctly  
**Example**: `world_to_chunk(IVec3(100, 50, -10))` → `IVec3(3, 1, -1)`

### Chunk → World (center)
```rust
fn chunk_to_world(chunk_pos: IVec3) -> Vec3 {
    Vec3::new(
        chunk_pos.x as f32 * 32.0 + 16.0,
        chunk_pos.y as f32 * 32.0 + 16.0,
        chunk_pos.z as f32 * 32.0 + 16.0,
    )
}
```
**Returns**: Center of the chunk in world space (floating point)

### World → Local Voxel
```rust
fn world_to_local(world_pos: IVec3, chunk_pos: IVec3) -> Option<IVec3> {
    let local = world_pos - chunk_pos * 32;
    if local.x >= 0 && local.x < 32
        && local.y >= 0 && local.y < 32
        && local.z >= 0 && local.z < 32 {
        Some(local)
    } else {
        None
    }
}
```
**Returns**: `None` if world_pos is outside the chunk

### Local → World
```rust
fn local_to_world(local: IVec3, chunk_pos: IVec3) -> IVec3 {
    chunk_pos * 32 + local
}
```

## Edge Cases and Rules

### Chunk Boundaries
- Voxels at chunk edges (local coordinate = 0 or 31) must propagate updates to neighbors
- Example: Setting voxel at `(31, y, z)` in chunk `(0, 0, 0)` affects chunk `(1, 0, 0)`

### Negative Coordinates
- **CRITICAL**: Use `div_euclid()` NOT regular division for negative coordinates
- Wrong: `-1 / 32 = 0` (incorrect!)
- Correct: `-1.div_euclid(32) = -1` ✓

### Floating Point World Positions
For camera/entity positions (Vec3):
```rust
fn world_float_to_chunk(world_pos: Vec3) -> IVec3 {
    IVec3::new(
        (world_pos.x / 32.0).floor() as i32,
        (world_pos.y / 32.0).floor() as i32,
        (world_pos.z / 32.0).floor() as i32,
    )
}
```

## Volume Transforms (Milestone 1)

Each volume has an associated transform:
```rust
pub struct VolumeTransform {
    pub position: Vec3,    // World space offset
    pub rotation: Quat,    // Rotation (identity = no rotation)
    pub scale: f32,        // Always 1.0 for now
}
```

**Default**: Identity transform (position = origin, no rotation, scale = 1.0)

## Invariants

### Exactness
✅ All conversions must be **exact and reversible** for integer coordinates:
```rust
assert_eq!(world_to_local(local_to_world(pos, chunk), chunk), Some(pos));
```

### Consistency
✅ Same world position must always map to same chunk:
```rust
let chunk1 = world_to_chunk(world_pos);
let chunk2 = world_to_chunk(world_pos);
assert_eq!(chunk1, chunk2);
```

### Determinism
✅ Dirty regions must be deterministic:
- If block at boundary changes, **exactly** the affected neighbor chunks are marked dirty
- No false positives or false negatives

## Implementation Status
- ✅ Basic world↔chunk conversion exists (`ChunkManager::world_to_chunk`)
- ✅ Local↔world conversion exists (`Chunk::world_to_local`, `local_to_world`)
- ✅ Negative coordinate handling is correct (uses `div_euclid`)
- ⚠️  No VolumeTransform system yet (Milestone 1)
- ⚠️  Edge case handling exists but not fully documented
