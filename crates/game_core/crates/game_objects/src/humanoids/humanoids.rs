// humanoids.rs — Humanoid stocké en entité globale + remove() interne

use crate::entities::Entity;
use crate::logics::components::antenna::Antenna;
use crate::logics::{LogicalObject, LogicalObjectDelta};
use crate::physics::{PhysicalObject, PhysicalObjectDelta};
use crate::utils::arenas::with_current_write;
use crate::utils::ids::EntityId;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Humanoid {
    pub id: EntityId,
    pub name: Option<String>,
    pub physical_object: Option<PhysicalObject>,
    pub logical_object: Option<LogicalObject>,
    pub pending_deltas: Vec<HumanoidDelta>,
}

impl Humanoid {
    pub fn spawn(name: Option<String>, physical_object: Option<PhysicalObject>) -> EntityId {
        with_current_write(|a| {
            let id = a.alloc_entity_id();

            // LogicalObject + composants locaux
            let mut lo = LogicalObject::new(None);
            let _ant_id = Antenna::spawn(&mut lo, 10.0, 10_000.0, 200.0, true, None);

            let h = Humanoid {
                id,
                name,
                physical_object,
                logical_object: Some(lo),
                pending_deltas: Vec::new(),
            };

            let e = Arc::new(RwLock::new(Entity::Humanoid(h)));
            let back = a.insert_entity(e);
            debug_assert_eq!(back, id);

            a.tag_entity(id);
            a.tag_physical(id);
            a.tag_logical(id);
            a.tag_humanoid(id);
            id
        })
    }

    /// Supprime récursivement cet humanoïde (nettoie ses composants)
    #[inline]
    pub fn remove(id: EntityId) -> bool {
        with_current_write(|a| {
            let Some(h) = a.get_entity(id) else {
                return false;
            };

            {
                let mut g = h.write().unwrap();
                if let Entity::Humanoid(ref mut hu) = *g {
                    if let Some(ref mut lo) = hu.logical_object {
                        lo.clear_components();
                    }
                } else {
                    return false;
                }
            }

            let existed = a.remove_entity(id).is_some();
            if existed {
                a.lists.entity_ids.retain(|&x| x != id);
                a.lists.physical_entity_ids.retain(|&x| x != id);
                a.lists.logical_entity_ids.retain(|&x| x != id);
                a.lists.humanoid_ids.retain(|&x| x != id);
            }
            existed
        })
    }

    pub fn record_delta(&mut self, delta: HumanoidDelta) {
        self.pending_deltas.push(delta);
    }

    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<HumanoidDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = HumanoidDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }
}

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
