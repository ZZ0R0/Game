// blocks.rs — Block stockable en arène (HasId<u32>), aucune référence forte à Grid
use crate::physics::{IntOrientation, IntPosition};
use crate::utils::arenas::HasId;
use crate::utils::ids::{BlockDefId, EntityId, FactionId};
use std::sync::Arc;

// -------- Types de base

#[derive(Debug, Clone)]
pub struct BlockDef {
    pub id: BlockDefId,
    pub name: String,
    pub footprint: (i32, i32, i32),
    pub mass: f32,
    pub integrity: f32,
    pub block_type: String,
    pub available_components: Vec<String>,
}
impl BlockDef {
    pub fn new(
        id: BlockDefId,
        name: impl Into<String>,
        fp: (i32, i32, i32),
        mass: f32,
        integ: f32,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            footprint: fp,
            mass,
            integrity: integ,
            block_type: kind.into(),
            available_components: Vec::new(),
        }
    }
}

// -------- Block instance + deltas

#[derive(Debug, Clone)]
pub struct Block {
    pub id: EntityId,
    pub grid_id: EntityId,
    pub def: Arc<BlockDef>,
    pub current_integrity: f32,
    pub current_mass: f32,
    pub position: IntPosition,
    pub orientation: IntOrientation,
    pub faction_id: FactionId,
}

// Pour Arena<Block, u32>
impl HasId<EntityId> for Block {
    #[inline]
    fn id_ref(&self) -> &EntityId {
        &self.id
    }
    #[inline]
    fn id_mut(&mut self) -> &mut EntityId {
        &mut self.id
    }
}

impl Block {
    pub fn new(
        id: EntityId,
        grid_id: EntityId,
        def: Arc<BlockDef>,
        position: IntPosition,
        orientation: IntOrientation,
        integrity: f32,
        faction_id: FactionId,
    ) -> Self {
        Self {
            id,
            grid_id,
            current_mass: def.mass,
            def: def,
            position: position,
            orientation: orientation,
            current_integrity: integrity,
            faction_id: faction_id,
        }
    }
}
