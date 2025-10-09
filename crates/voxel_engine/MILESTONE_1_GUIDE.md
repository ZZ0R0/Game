# Milestone 1 - Unified Voxel Model

## Overview
This milestone introduces a unified abstraction for voxel volumes that supports both:
- **Grid volumes**: Finite, mutable, stored grids (for player-edited regions)
- **Celestial volumes**: Infinite, procedural terrain with sparse modifications

## Architecture

```
Volume (trait)
├── GridVolume (mutable storage)
└── CelestialVolume (procedural + deltas)

VoxelSchema (trait)
├── BlockSchema (discrete blocks)
└── DensitySchema (smooth density field)
```

## Usage Examples

### 1. Creating a Grid Volume (Player Base)

```rust
use voxel_engine::{GridVolume, VolumeTransform, BlockSchema};
use glam::{IVec3, Vec3};

// Create a bounded grid volume for a player base
let min = IVec3::new(-10, 0, -10);  // -320m to +320m in XZ
let max = IVec3::new(10, 5, 10);     // 0m to 160m in Y
let mut grid = GridVolume::bounded(min, max);

// Add a chunk with block data
let chunk_schema = BlockSchema::new(IVec3::ZERO);
grid.insert_chunk(IVec3::ZERO, Box::new(chunk_schema));

// Modify voxels (thread-safe)
grid.set_voxel(IVec3::new(5, 10, 5), voxel_engine::STONE);

// Get dirty regions for mesh updates
let dirty_chunks = grid.take_dirty_regions();
for chunk_pos in dirty_chunks {
    println!("Need to remesh chunk: {:?}", chunk_pos);
}
```

### 2. Creating a Celestial Volume (Planet)

```rust
use voxel_engine::{CelestialVolume, TerrainGenerator, TerrainConfig, VolumeTransform};
use std::sync::Arc;
use glam::Vec3;

// Create terrain generator
let config = TerrainConfig {
    base_height: 64.0,
    amplitude: 32.0,
    frequency: 0.02,
    water_level: 60,
    seed: 12345,
};
let generator = Arc::new(TerrainGenerator::new(config));

// Create celestial volume positioned at (1000, 0, 2000)
let transform = VolumeTransform::with_position(Vec3::new(1000.0, 0.0, 2000.0));
let mut planet = CelestialVolume::new(generator, transform);

// Query voxels (generates on-demand)
let is_solid = planet.is_solid(IVec3::new(1050, 70, 2050));

// Modify terrain (stored as delta)
planet.set_voxel(IVec3::new(1050, 70, 2050), voxel_engine::AIR);

// Check modification count
println!("Modified chunks: {}", planet.delta_count());
```

### 3. Using Different Schemas

```rust
use voxel_engine::{BlockSchema, DensitySchema, VoxelSchema};
use glam::IVec3;

// Block schema (discrete)
let mut block_schema = BlockSchema::new(IVec3::ZERO);
block_schema.set_local(0, 0, 0, voxel_engine::STONE);
assert!(block_schema.is_solid(IVec3::ZERO));
assert_eq!(block_schema.surface_sign(IVec3::ZERO), 1.0);  // Solid

// Density schema (smooth)
let mut density_schema = DensitySchema::new(IVec3::ZERO);
density_schema.set_local(0, 0, 0, 200, voxel_engine::MAT_STONE);  // 200/255 density
assert!(density_schema.is_solid(IVec3::ZERO));
assert!(density_schema.surface_sign(IVec3::ZERO) > 0.0);  // Inside surface

// Surface voxel (density = 128)
density_schema.set_local(1, 0, 0, 128, voxel_engine::MAT_DIRT);
assert_eq!(density_schema.surface_sign(IVec3::new(1, 0, 0)), 0.0);  // Exactly on surface
```

### 4. Coordinate Conversions

```rust
use voxel_engine::{world_to_chunk_pos, chunk::CHUNK_SIZE};
use glam::IVec3;

// World position to chunk position
let world_pos = IVec3::new(100, 50, -10);
let chunk_pos = world_to_chunk_pos(world_pos);
assert_eq!(chunk_pos, IVec3::new(3, 1, -1));

// Negative coordinates (handled correctly)
let neg_world = IVec3::new(-1, 0, 0);
let neg_chunk = world_to_chunk_pos(neg_world);
assert_eq!(neg_chunk, IVec3::new(-1, 0, 0));  // NOT zero!

// Local to world
let chunk_pos = IVec3::new(2, 1, 0);
let local = IVec3::new(15, 20, 8);
let world = chunk_pos * CHUNK_SIZE as i32 + local;
assert_eq!(world, IVec3::new(79, 52, 8));
```

### 5. Dirty Region Tracking

```rust
use voxel_engine::DirtyRegions;
use glam::IVec3;

let mut dirty = DirtyRegions::new();

// Mark a voxel change
dirty.mark_position_dirty(IVec3::new(5, 10, 5));

// Mark boundary voxel (also marks neighbors)
dirty.mark_position_dirty(IVec3::new(31, 0, 0));  // Edge of chunk
assert!(dirty.dirty_count() >= 2);  // Main chunk + neighbor

// Consume dirty chunks
let chunks_to_update = dirty.take_dirty_chunks();
for chunk_pos in chunks_to_update {
    // Regenerate mesh for this chunk
    println!("Remesh: {:?}", chunk_pos);
}

assert_eq!(dirty.dirty_count(), 0);  // Cleared after taking
```

### 6. Volume Transforms

```rust
use voxel_engine::VolumeTransform;
use glam::{Vec3, Quat};

// Identity transform
let identity = VolumeTransform::identity();
assert_eq!(identity.position, Vec3::ZERO);
assert_eq!(identity.scale, 1.0);

// Offset transform
let offset = VolumeTransform::with_position(Vec3::new(1000.0, 0.0, 2000.0));

// Local to world
let local = Vec3::new(50.0, 10.0, 100.0);
let world = offset.local_to_world(local);
assert_eq!(world, Vec3::new(1050.0, 10.0, 2100.0));

// World to local (inverse)
let back = offset.world_to_local(world);
assert!((back - local).length() < 0.001);
```

## Integration with Existing Code

### Migrating from ChunkManager

```rust
// OLD: ChunkManager
let mut manager = ChunkManager::new();
manager.insert(chunk);
manager.set_block(world_pos, STONE);

// NEW: GridVolume
let mut grid = GridVolume::unbounded();
grid.insert_chunk(chunk_pos, Box::new(schema));
grid.set_voxel(world_pos, STONE);
```

### Using with Meshing

```rust
use voxel_engine::{GridVolume, mesh_chunk};

let mut volume = GridVolume::unbounded();
// ... populate volume ...

// Get dirty chunks
let dirty = volume.take_dirty_regions();

for chunk_pos in dirty {
    if let Some(chunk_data) = get_chunk_data(&volume, chunk_pos) {
        let mesh = mesh_chunk(&chunk_data);
        // Upload mesh to GPU...
    }
}
```

## Testing

Run tests:
```bash
cargo test -p voxel_engine
```

Test specific modules:
```bash
cargo test -p voxel_engine voxel_schema
cargo test -p voxel_engine volume
```

## Performance Characteristics

### GridVolume
- **Memory**: O(n) where n = number of allocated chunks
- **Read**: O(1) with RwLock contention
- **Write**: O(1) + dirty tracking overhead
- **Thread-safe**: Yes (RwLock)

### CelestialVolume
- **Memory**: O(m) where m = number of modified chunks (deltas)
- **Read**: O(1) procedural generation or O(1) delta lookup
- **Write**: O(1) delta insertion
- **Thread-safe**: Yes (RwLock on deltas)

### BlockSchema
- **Memory**: 64 KiB per chunk (32³ × u16)
- **Query**: O(1)

### DensitySchema
- **Memory**: 64 KiB per chunk (32³ × 2 bytes)
- **Query**: O(1)
- **Surface extraction**: Compatible with marching cubes

## Invariants (Milestone 1 Acceptance)

✅ **Coordinate conversions are exact and reversible**
```rust
let world = IVec3::new(100, 50, -10);
let chunk = world_to_chunk_pos(world);
let local = world - chunk * 32;
let reconstructed = chunk * 32 + local;
assert_eq!(world, reconstructed);
```

✅ **Both schemas answer occupancy consistently**
```rust
// Air is never solid
assert!(!block_schema.is_solid(air_pos));
assert!(!density_schema.is_solid(air_pos));

// Solid blocks are solid
assert!(block_schema.is_solid(stone_pos));
assert!(density_schema.is_solid(dense_pos));
```

✅ **Dirty regions are deterministic**
```rust
// Changing a voxel marks exactly the affected chunks
dirty.mark_position_dirty(boundary_voxel);
let chunks = dirty.take_dirty_chunks();
// chunks contains main chunk + affected neighbors (deterministic)
```

✅ **Thread-safe reads**
```rust
// Multiple threads can read simultaneously
let grid = Arc::new(GridVolume::unbounded());
let handles: Vec<_> = (0..10)
    .map(|i| {
        let g = grid.clone();
        std::thread::spawn(move || {
            g.is_solid(IVec3::new(i, 0, 0))
        })
    })
    .collect();
```

## Next Steps (Future Milestones)

- **Milestone 2**: Implement marching cubes for DensitySchema
- **Milestone 3**: Add rotation support to VolumeTransform
- **Milestone 4**: Optimize delta storage with run-length encoding
- **Milestone 5**: Multi-volume rendering and collision
