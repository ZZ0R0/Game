// grids.rs — Grid stockant UNIQUEMENT des EntityId de blocks (blocks = entités globales)

use crate::entities::Entity;
use crate::logics::{LogicalObject, LogicalObjectDelta};
use crate::physics::boundaries::RectBoundaries;
use crate::physics::{PhysicalObject, PhysicalObjectDelta};
use crate::utils::arenas::with_current_write;
use crate::utils::ids::EntityId;

#[derive(Debug, Clone, Hash)]
pub enum GridSizeClass {
    Small,
    Large,
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub id: EntityId,
    pub name: Option<String>,
    pub physical_object: Option<PhysicalObject>,
    pub logical_object: Option<LogicalObject>,
    pub size_class: Option<GridSizeClass>,
    pub boundaries: Option<RectBoundaries>,
    pub pending_deltas: Vec<GridDelta>,
    /// Uniquement des IDs d'entités de type Block
    pub block_ids: Vec<EntityId>,
}

impl Grid {
    pub fn spawn(
        name: Option<String>,
        physical_object: Option<PhysicalObject>,
        logical_object: Option<LogicalObject>,
        size_class: Option<GridSizeClass>,
    ) -> EntityId {
        with_current_write(|a| {
            let id = a.alloc_entity_id();

            let g = Grid {
                id,
                name,
                physical_object,
                logical_object,
                size_class,
                boundaries: Some(RectBoundaries::null()),
                pending_deltas: Vec::new(),
                block_ids: Vec::new(),
            };
            let e = Entity::Grid(g);

            let back = a.insert_entity(e);
            debug_assert_eq!(back, id);

            a.tag_entity(id);
            a.tag_physical(id);
            a.tag_logical(id);
            id
        })
    }

    // ---- Gestion des blocks via IDs -----------------------------------------

    #[inline]
    pub fn add_block_id(&mut self, bid: EntityId) {
        if !self.block_ids.contains(&bid) {
            self.block_ids.push(bid);
        }
    }

    #[inline]
    pub fn remove_block_id(&mut self, bid: EntityId) -> bool {
        let len0 = self.block_ids.len();
        self.block_ids.retain(|&x| x != bid);
        self.block_ids.len() != len0
    }

    #[inline]
    pub fn has_block_id(&self, bid: EntityId) -> bool {
        self.block_ids.contains(&bid)
    }

    #[inline]
    pub fn set_block_ids(&mut self, mut ids: Vec<EntityId>) {
        ids.sort_unstable();
        ids.dedup();
        self.block_ids = ids;
    }

    // ---- Deltas -------------------------------------------------------------

    pub fn record_delta(&mut self, delta: GridDelta) {
        self.pending_deltas.push(delta);
    }

    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<GridDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = GridDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }
}

#[derive(Debug, Clone)]
pub struct GridDelta {
    pub timestamp: Option<u64>,
    pub physical_object_delta: Option<PhysicalObjectDelta>,
    pub logical_object_delta: Option<LogicalObjectDelta>,
    /// Remplacement complet optionnel de la liste d'IDs de blocks
    pub block_ids: Option<Vec<EntityId>>,
}

impl GridDelta {
    pub fn merge(mut deltas: Vec<GridDelta>) -> Option<GridDelta> {
        if deltas.is_empty() {
            return None;
        }
        deltas.sort_by_key(|d| d.timestamp);
        let mut m = deltas.remove(0);
        for d in deltas {
            if d.physical_object_delta.is_some() {
                m.physical_object_delta = d.physical_object_delta;
            }
            if d.logical_object_delta.is_some() {
                m.logical_object_delta = d.logical_object_delta;
            }
            if d.block_ids.is_some() {
                m.block_ids = d.block_ids;
            }
        }
        Some(m)
    }

    pub fn apply_to(&self, grid: &mut Grid) {
        if let Some(ref pod) = self.physical_object_delta {
            if let Some(ref mut po) = grid.physical_object {
                pod.apply_to(po);
            }
        }
        if let Some(ref lod) = self.logical_object_delta {
            if let Some(ref mut lo) = grid.logical_object {
                lod.apply_to(lo);
            }
        }
        if let Some(ref ids) = self.block_ids {
            let mut ids2 = ids.clone();
            ids2.sort_unstable();
            ids2.dedup();
            grid.block_ids = ids2;
        }
    }
}
