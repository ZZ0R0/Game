// grids.rs — minimal, delta physique centralisé dans objects::PhysicalObjectDelta

use crate::logics::LogicalObject;
use crate::physics::{PhysicalObject, RectBounds};
use crate::utils::arenas::HasId;
use crate::utils::ids::{EntityId, PlayerId};

#[derive(Debug, Clone, Hash)]
pub enum GridSizeClass {
    Small,
    Large,
}

#[derive(Clone)]
pub struct Grid {
    pub id: EntityId,
    pub name: String,
    pub physical_object: PhysicalObject,
    pub logical_object: LogicalObject,
    pub size_class: GridSizeClass,
    pub boundaries: RectBounds,
    pub hash: u64,
    pub pending_deltas: Vec<GridDelta>,
    pub player_id: PlayerId,
}

impl HasId<EntityId> for Grid {
    fn id_ref(&self) -> &EntityId {
        &self.id
    }
    fn id_mut(&mut self) -> &mut EntityId {
        &mut self.id
    }
}

impl Grid {}

#[derive(Debug, Clone)]
pub struct GridDelta {
    pub timestamp: u32,
    pub entity_id: EntityId,
}
impl GridDelta {}
