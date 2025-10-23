// grids.rs — arène locale de blocks par grid + API minimale

use crate::blocks::Block;
use crate::entities::Entity;
use crate::logics::{LogicalObject, LogicalObjectDelta};
use crate::physics::boundaries::RectBoundaries;
use crate::physics::{PhysicalObject, PhysicalObjectDelta};
use crate::utils::arenas::with_current_write;
use crate::utils::arenas::{Arena, HasId};
use crate::utils::ids::{EntityId, PlayerId};

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
    pub block_ids: Vec<EntityId>,
}

impl HasId<EntityId> for Grid {
    fn id_ref(&self) -> &EntityId {
        &self.id
    }
    fn id_mut(&mut self) -> &mut EntityId {
        &mut self.id
    }
}

impl Grid {
    pub fn spawn(name: Option<String>, physical_object: Option<PhysicalObject>, logical_object: Option<LogicalObject>, size_class: Option<GridSizeClass>) -> EntityId {
        with_current_write(|a| {
            let id = a.alloc_entity_id();

            let g = Grid {
                id: id,
                name: name,
                physical_object: physical_object,
                logical_object: logical_object,
                size_class: size_class,
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
            a.tag_humanoid(id);
            id
        })
    }

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

    /// À appeler dans ton constructeur de Grid si tu en ajoutes un.
    pub fn init_block_storage(&mut self) {
        self.blocks = Arena::new();
        self.block_ids = Vec::new();
        self.block_counter = 0;
    }

    // ---------- Gestion IDs de blocks ----------
    #[inline]
    pub fn alloc_block_id(&mut self) -> BlockId {
        let v = self.block_counter;
        self.block_counter = v.wrapping_add(1);
        BlockId(v)
    }

    // ---------- Arène locale API ----------
    /// Insère un block déjà porteur de son `BlockId`. Retourne ce `BlockId`.
    #[inline]
    pub fn insert_block(&mut self, b: Block) -> BlockId {
        let id = self.blocks.insert(b);
        if !self.block_ids.contains(&id) {
            self.block_ids.push(id);
        }
        id
    }

    #[inline]
    pub fn block(&self, id: BlockId) -> Option<&Block> {
        self.blocks.get(id)
    }

    #[inline]
    pub fn block_mut(&mut self, id: BlockId) -> Option<&mut Block> {
        self.blocks.get_mut(id)
    }

    #[inline]
    pub fn remove_block(&mut self, id: BlockId) -> Option<Block> {
        self.block_ids.retain(|&x| x != id);
        self.blocks.remove(id)
    }
}

#[derive(Debug, Clone)]
pub struct GridDelta {
    pub timestamp: u32,
    pub entity_id: EntityId,
}
impl GridDelta {}

#[derive(Debug, Clone)]
pub struct HumanoidDelta {
    pub timestamp: Option<u64>,
    pub physical_object_delta: Option<PhysicalObjectDelta>,
    pub logical_object_delta: Option<LogicalObjectDelta>,
}

impl HumanoidDelta {
    pub fn merge(mut deltas: Vec<HumanoidDelta>) -> Option<HumanoidDelta> {
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
        }
        Some(m)
    }

    pub fn apply_to(&self, humanoid: &mut Humanoid) {
        if let Some(ref pod) = self.physical_object_delta {
            if let Some(ref mut po) = humanoid.physical_object {
                pod.apply_to(po);
            }
        }
        if let Some(ref lod) = self.logical_object_delta {
            if let Some(ref mut lo) = humanoid.logical_object {
                lod.apply_to(lo);
            }
        }
    }
}
