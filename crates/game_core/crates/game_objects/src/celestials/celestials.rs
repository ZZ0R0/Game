// celestials.rs — Celestial + remove() simple

use crate::physics::PhysicalObject;
use crate::utils::arenas::{with_current_write, HasId};
use crate::utils::ids::EntityId;

#[derive(Debug, Clone)]
pub struct Celestial {
    pub id: EntityId,
    pub name: String,
    pub celestial_type: CelestialType,
    pub physical_object: PhysicalObject,
    pub radius: f32,
    pub gravity_strength: f32,
    pub atmosphere: bool,
    /// Deltas en attente
    pub pending_deltas: Vec<CelestialDelta>,
}

#[derive(Debug, Clone)]
pub enum CelestialType {
    Planet,
    Moon,
    Asteroid,
    Star,
    Undefined,
}

// Pour Arena<Celestial, EntityId>
impl HasId<EntityId> for Celestial {
    #[inline]
    fn id_ref(&self) -> &EntityId {
        &self.id
    }
    #[inline]
    fn id_mut(&mut self) -> &mut EntityId {
        &mut self.id
    }
}

impl Celestial {
    pub fn new(
        id: EntityId,
        name: String,
        celestial_type: CelestialType,
        physical_object: PhysicalObject,
        radius: f32,
        gravity_strength: f32,
        atmosphere: bool,
    ) -> Self {
        Self {
            id,
            name,
            celestial_type,
            physical_object,
            radius,
            gravity_strength,
            atmosphere,
            pending_deltas: Vec::new(),
        }
    }

    /// Suppression non récursive (pas de contenu enfant pour l’instant)
    #[inline]
    pub fn remove(id: EntityId) -> bool {
        with_current_write(|a| {
            let existed = a.remove_entity(id).is_some();
            if existed {
                a.lists.entity_ids.retain(|&x| x != id);
                a.lists.physical_entity_ids.retain(|&x| x != id);
                a.lists.celestial_ids.retain(|&x| x != id);
            }
            existed
        })
    }
}

/// Delta céleste
#[derive(Debug, Clone)]
pub struct CelestialDelta {
    pub timestamp: u64,
    pub id: EntityId,
}
