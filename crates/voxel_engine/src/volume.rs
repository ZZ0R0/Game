//! Unified volume abstraction for voxel data
//!
//! Milestone 1: Two volume kinds:
//! 1. GridVolume: Finite, mutable, stored grid (for edited regions)
//! 2. CelestialVolume: Infinite, procedural with sparse deltas (for planets/terrain)

use crate::chunk::{BlockId, CHUNK_SIZE};
use crate::voxel_schema::{MaterialId, VoxelSchema};
use glam::{IVec3, Quat, Vec3};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Transform for positioning a volume in world space
#[derive(Debug, Clone)]
pub struct VolumeTransform {
    /// Position in world space (meters)
    pub position: Vec3,

    /// Rotation (quaternion)
    pub rotation: Quat,

    /// Uniform scale factor (always 1.0 for Milestone 1)
    pub scale: f32,
}

impl Default for VolumeTransform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
        }
    }
}

impl VolumeTransform {
    /// Create identity transform (no offset, no rotation, scale=1)
    pub fn identity() -> Self {
        Self::default()
    }

    /// Create transform with only position offset
    pub fn with_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: 1.0,
        }
    }

    /// Transform a local position to world space
    pub fn local_to_world(&self, local: Vec3) -> Vec3 {
        self.position + self.rotation * (local * self.scale)
    }

    /// Transform a world position to local space
    pub fn world_to_local(&self, world: Vec3) -> Vec3 {
        (self.rotation.inverse() * (world - self.position)) / self.scale
    }
}

/// Dirty region tracking for efficient updates
#[derive(Debug, Clone)]
pub struct DirtyRegions {
    /// Set of chunk positions that need updates
    dirty_chunks: HashSet<IVec3>,
}

impl DirtyRegions {
    pub fn new() -> Self {
        Self {
            dirty_chunks: HashSet::new(),
        }
    }

    /// Mark a chunk as dirty
    pub fn mark_chunk_dirty(&mut self, chunk_pos: IVec3) {
        self.dirty_chunks.insert(chunk_pos);
    }

    /// Mark a world position as dirty (marks containing chunk + neighbors if on boundary)
    pub fn mark_position_dirty(&mut self, world_pos: IVec3) {
        let chunk_pos = world_to_chunk_pos(world_pos);
        self.dirty_chunks.insert(chunk_pos);

        // Check if on chunk boundary and mark neighbors
        let local = world_pos - chunk_pos * CHUNK_SIZE as i32;
        let on_boundary_x = local.x == 0 || local.x == (CHUNK_SIZE as i32 - 1);
        let on_boundary_y = local.y == 0 || local.y == (CHUNK_SIZE as i32 - 1);
        let on_boundary_z = local.z == 0 || local.z == (CHUNK_SIZE as i32 - 1);

        if on_boundary_x {
            self.dirty_chunks
                .insert(chunk_pos + IVec3::new(if local.x == 0 { -1 } else { 1 }, 0, 0));
        }
        if on_boundary_y {
            self.dirty_chunks
                .insert(chunk_pos + IVec3::new(0, if local.y == 0 { -1 } else { 1 }, 0));
        }
        if on_boundary_z {
            self.dirty_chunks
                .insert(chunk_pos + IVec3::new(0, 0, if local.z == 0 { -1 } else { 1 }));
        }
    }

    /// Get all dirty chunks and clear the set
    pub fn take_dirty_chunks(&mut self) -> Vec<IVec3> {
        let chunks: Vec<IVec3> = self.dirty_chunks.drain().collect();
        chunks
    }

    /// Check if a chunk is dirty
    pub fn is_chunk_dirty(&self, chunk_pos: IVec3) -> bool {
        self.dirty_chunks.contains(&chunk_pos)
    }

    /// Clear all dirty flags
    pub fn clear(&mut self) {
        self.dirty_chunks.clear();
    }

    /// Get number of dirty chunks
    pub fn dirty_count(&self) -> usize {
        self.dirty_chunks.len()
    }
}

impl Default for DirtyRegions {
    fn default() -> Self {
        Self::new()
    }
}

/// Core trait for all volume types
///
/// Provides unified interface for querying and modifying voxel data,
/// whether it's stored in memory (Grid) or procedurally generated (Celestial)
pub trait Volume: Send + Sync {
    /// Get the voxel schema type
    fn schema(&self) -> &dyn VoxelSchema;

    /// Get the transform for this volume
    fn transform(&self) -> &VolumeTransform;

    /// Check if a voxel is solid at world position
    fn is_solid(&self, world_pos: IVec3) -> bool {
        self.schema().is_solid(world_pos)
    }

    /// Get material at world position
    fn material_at(&self, world_pos: IVec3) -> MaterialId {
        self.schema().material_at(world_pos)
    }

    /// Get surface sign at world position
    fn surface_sign(&self, world_pos: IVec3) -> f32 {
        self.schema().surface_sign(world_pos)
    }

    /// Set voxel at world position (if mutable)
    /// Returns true if successful
    fn set_voxel(&mut self, world_pos: IVec3, block: BlockId) -> bool;

    /// Get dirty regions for this volume
    fn dirty_regions(&self) -> &DirtyRegions;

    /// Take dirty regions (consume and clear)
    fn take_dirty_regions(&mut self) -> Vec<IVec3>;

    /// Get bounding box (None = unbounded)
    fn bounds(&self) -> Option<(IVec3, IVec3)>;

    /// Get human-readable name for this volume type
    fn volume_type(&self) -> &str;
}

/// Grid volume: Finite, mutable, stored in memory
///
/// Use cases:
/// - Player-edited regions
/// - Space stations / structures
/// - Bounded game worlds
///
/// Properties:
/// - Thread-safe reads (RwLock)
/// - Batched writes
/// - Explicit bounds
/// - Dirty region tracking
pub struct GridVolume {
    /// Stored chunks (only allocated chunks exist in map)
    chunks: Arc<RwLock<HashMap<IVec3, Box<dyn VoxelSchema>>>>,

    /// Transform in world space
    transform: VolumeTransform,

    /// Bounding box (min, max) in chunk coordinates
    bounds: Option<(IVec3, IVec3)>,

    /// Dirty regions
    dirty: Arc<RwLock<DirtyRegions>>,
}

impl GridVolume {
    /// Create a new grid volume with optional bounds
    pub fn new(transform: VolumeTransform, bounds: Option<(IVec3, IVec3)>) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            transform,
            bounds,
            dirty: Arc::new(RwLock::new(DirtyRegions::new())),
        }
    }

    /// Create an unbounded grid volume
    pub fn unbounded() -> Self {
        Self::new(VolumeTransform::identity(), None)
    }

    /// Create a bounded grid volume
    pub fn bounded(min_chunk: IVec3, max_chunk: IVec3) -> Self {
        Self::new(VolumeTransform::identity(), Some((min_chunk, max_chunk)))
    }

    /// Insert a chunk with schema
    pub fn insert_chunk(&mut self, chunk_pos: IVec3, schema: Box<dyn VoxelSchema>) {
        let mut chunks = self.chunks.write().unwrap();
        chunks.insert(chunk_pos, schema);

        let mut dirty = self.dirty.write().unwrap();
        dirty.mark_chunk_dirty(chunk_pos);
    }

    /// Remove a chunk
    pub fn remove_chunk(&mut self, chunk_pos: IVec3) -> Option<Box<dyn VoxelSchema>> {
        let mut chunks = self.chunks.write().unwrap();
        chunks.remove(&chunk_pos)
    }

    /// Get chunk count
    pub fn chunk_count(&self) -> usize {
        let chunks = self.chunks.read().unwrap();
        chunks.len()
    }

    /// Check if chunk exists
    pub fn has_chunk(&self, chunk_pos: IVec3) -> bool {
        let chunks = self.chunks.read().unwrap();
        chunks.contains_key(&chunk_pos)
    }
}

impl Volume for GridVolume {
    fn schema(&self) -> &dyn VoxelSchema {
        // This is a bit tricky with RwLock - for now, return a dummy
        // In real impl, we'd need to handle this differently
        unimplemented!("GridVolume uses per-chunk schemas")
    }

    fn transform(&self) -> &VolumeTransform {
        &self.transform
    }

    fn set_voxel(&mut self, world_pos: IVec3, _block: BlockId) -> bool {
        let chunk_pos = world_to_chunk_pos(world_pos);

        // Check bounds
        if let Some((min, max)) = self.bounds {
            if chunk_pos.x < min.x
                || chunk_pos.x > max.x
                || chunk_pos.y < min.y
                || chunk_pos.y > max.y
                || chunk_pos.z < min.z
                || chunk_pos.z > max.z
            {
                return false;
            }
        }

        // Mark dirty
        let mut dirty = self.dirty.write().unwrap();
        dirty.mark_position_dirty(world_pos);

        // TODO: Actual voxel modification
        true
    }

    fn dirty_regions(&self) -> &DirtyRegions {
        // Can't return &DirtyRegions from RwLock easily
        unimplemented!("Use take_dirty_regions instead")
    }

    fn take_dirty_regions(&mut self) -> Vec<IVec3> {
        let mut dirty = self.dirty.write().unwrap();
        dirty.take_dirty_chunks()
    }

    fn bounds(&self) -> Option<(IVec3, IVec3)> {
        self.bounds
    }

    fn volume_type(&self) -> &str {
        "GridVolume"
    }
}

/// Celestial volume: Infinite, procedurally generated with sparse deltas
///
/// Use cases:
/// - Planets
/// - Asteroids
/// - Infinite terrain
///
/// Properties:
/// - Read-through procedural provider
/// - Sparse delta storage for modifications
/// - Virtually unbounded
/// - Lazy generation
pub struct CelestialVolume {
    /// Procedural generator (trait object)
    provider: Arc<dyn ProceduralProvider>,

    /// Sparse delta storage (overrides procedural data)
    deltas: Arc<RwLock<HashMap<IVec3, Box<dyn VoxelSchema>>>>,

    /// Transform in world space
    transform: VolumeTransform,

    /// Dirty regions
    dirty: Arc<RwLock<DirtyRegions>>,
}

impl CelestialVolume {
    /// Create a new celestial volume with a procedural provider
    pub fn new(provider: Arc<dyn ProceduralProvider>, transform: VolumeTransform) -> Self {
        Self {
            provider,
            deltas: Arc::new(RwLock::new(HashMap::new())),
            transform,
            dirty: Arc::new(RwLock::new(DirtyRegions::new())),
        }
    }

    /// Get delta count (modified chunks)
    pub fn delta_count(&self) -> usize {
        let deltas = self.deltas.read().unwrap();
        deltas.len()
    }

    /// Check if chunk has modifications
    pub fn has_delta(&self, chunk_pos: IVec3) -> bool {
        let deltas = self.deltas.read().unwrap();
        deltas.contains_key(&chunk_pos)
    }

    /// Generate chunk (read-through: check delta first, then procedural)
    pub fn get_chunk_schema(&self, chunk_pos: IVec3) -> Box<dyn VoxelSchema> {
        // Check for delta first
        {
            let deltas = self.deltas.read().unwrap();
            if deltas.contains_key(&chunk_pos) {
                // TODO: Return delta (need to clone schema)
                // For now, fall through to procedural
            }
        }

        // Generate procedurally
        self.provider.generate_chunk(chunk_pos)
    }

    /// Clear all deltas (reset to pure procedural)
    pub fn clear_deltas(&mut self) {
        let mut deltas = self.deltas.write().unwrap();
        deltas.clear();
    }
}

impl Volume for CelestialVolume {
    fn schema(&self) -> &dyn VoxelSchema {
        unimplemented!("CelestialVolume uses per-chunk schemas")
    }

    fn transform(&self) -> &VolumeTransform {
        &self.transform
    }

    fn set_voxel(&mut self, world_pos: IVec3, _block: BlockId) -> bool {
        let _chunk_pos = world_to_chunk_pos(world_pos);

        // Mark dirty
        let mut dirty = self.dirty.write().unwrap();
        dirty.mark_position_dirty(world_pos);

        // TODO: Store delta
        true
    }

    fn dirty_regions(&self) -> &DirtyRegions {
        unimplemented!("Use take_dirty_regions instead")
    }

    fn take_dirty_regions(&mut self) -> Vec<IVec3> {
        let mut dirty = self.dirty.write().unwrap();
        dirty.take_dirty_chunks()
    }

    fn bounds(&self) -> Option<(IVec3, IVec3)> {
        None // Unbounded
    }

    fn volume_type(&self) -> &str {
        "CelestialVolume"
    }
}

/// Trait for procedural voxel generation
///
/// Implementors provide infinite terrain generation
pub trait ProceduralProvider: Send + Sync {
    /// Generate a chunk at the given position
    fn generate_chunk(&self, chunk_pos: IVec3) -> Box<dyn VoxelSchema>;

    /// Get provider name
    fn provider_name(&self) -> &str;
}

/// Helper: Convert world position to chunk position
pub fn world_to_chunk_pos(world_pos: IVec3) -> IVec3 {
    IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE as i32),
        world_pos.y.div_euclid(CHUNK_SIZE as i32),
        world_pos.z.div_euclid(CHUNK_SIZE as i32),
    )
}
