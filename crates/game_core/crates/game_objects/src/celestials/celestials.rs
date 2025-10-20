// celestials.rs — version corrigée : compatible arène (HasId<u32>), IDs u32
use crate::physics::PhysicalObject;
use crate::utils::arenas::HasId;
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

// Pour Arena<Celestial, u32>
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
}

/// Delta céleste
#[derive(Debug, Clone)]
pub struct CelestialDelta {
    timestamp: u64,
    pub id: EntityId,
}
