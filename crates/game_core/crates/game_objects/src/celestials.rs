// celestials.rs — version corrigée : compatible arène (HasId<u32>), IDs u32 
use crate::utils::arena::HasId;
use crate::physics::{Acceleration, FloatOrientation, FloatPosition, PhysicalObject, Velocity};
use std::f32::NAN;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CelestialId(pub u32);

#[derive(Debug, Clone)]
pub enum CelestialType {
    Planet,
    Moon,
    Asteroid,
    Star,
    Undefined
}

#[derive(Debug, Clone)]
pub struct Celestial {
    pub id: CelestialId,
    pub name: String,
    pub celestial_type: CelestialType,
    pub physical_object: PhysicalObject,
    pub radius: f32,
    pub gravity_strength: f32,
    pub atmosphere: bool,
    /// Deltas en attente
    pub pending_deltas: Vec<CelestialDelta>,
}

// Pour Arena<Celestial, u32>
impl HasId<u32> for Celestial {
    #[inline]
    fn id_ref(&self) -> &u32 {
        &self.id.0
    }
}

impl Celestial {
    pub fn new(
        id: u32,
        name: String,
        celestial_type: CelestialType,
        physical_object: PhysicalObject,
        radius: f32,
        gravity_strength: f32,
        atmosphere: bool,
    ) -> Self {
        Self {
            id: CelestialId(id),
            name,
            celestial_type,
            physical_object,
            radius,
            gravity_strength,
            atmosphere,
            pending_deltas: Vec::new(),
        }
    }

    #[inline]
    pub fn get_position(&self) -> &FloatPosition {
        &self.physical_object.placed_object.position
    }
    #[inline]
    pub fn get_orientation(&self) -> &FloatOrientation {
        &self.physical_object.placed_object.orientation
    }
    #[inline]
    pub fn get_velocity(&self) -> &Velocity {
        &self.physical_object.velocity
    }
    #[inline]
    pub fn get_acceleration(&self) -> &Acceleration {
        &self.physical_object.acceleration
    }

    #[inline]
    pub fn set_position(&mut self, p: FloatPosition) {
        self.physical_object.placed_object.position = p;
    }
    #[inline]
    pub fn set_orientation(&mut self, o: FloatOrientation) {
        self.physical_object.placed_object.orientation = o;
    }
    #[inline]
    pub fn set_velocity(&mut self, v: Velocity) {
        self.physical_object.velocity = v;
    }
    #[inline]
    pub fn set_acceleration(&mut self, a: Acceleration) {
        self.physical_object.acceleration = a;
    }

    pub fn record_delta(&mut self, delta: CelestialDelta) {
        self.pending_deltas.push(delta);
    }

    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<CelestialDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = CelestialDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }

    pub fn create_position_delta(
        &self,
        new_position: FloatPosition,
        timestamp: u64,
        sequence: u64,
    ) -> CelestialDelta {
        CelestialDelta {
            celestial_id: self.id.0,
            position: Some(new_position),
            orientation: None,
            velocity: None,
            acceleration: None,
            timestamp,
            sequence,
        }
    }

    pub fn create_rotation_delta(
        &self,
        new_orientation: FloatOrientation,
        timestamp: u64,
        sequence: u64,
    ) -> CelestialDelta {
        CelestialDelta {
            celestial_id: self.id.0,
            position: None,
            orientation: Some(new_orientation),
            velocity: None,
            acceleration: None,
            timestamp,
            sequence,
        }
    }
}

/// Delta céleste
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CelestialDelta {
    pub celestial_id: u32,
    pub position: Option<FloatPosition>,
    pub orientation: Option<FloatOrientation>,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
    pub timestamp: u64,
    pub sequence: u64,
}

impl CelestialDelta {
    pub fn empty(celestial_id: u32, timestamp: u64, sequence: u64) -> Self {
        Self {
            celestial_id,
            position: None,
            orientation: None,
            velocity: None,
            acceleration: None,
            timestamp,
            sequence,
        }
    }

    pub fn apply_to(&self, celestial: &mut Celestial) {
        if let Some(ref pos) = self.position {
            celestial.physical_object.placed_object.position = pos.clone();
        }
        if let Some(ref o) = self.orientation {
            celestial.physical_object.placed_object.orientation = o.clone();
        }
        if let Some(ref v) = self.velocity {
            celestial.physical_object.velocity = v.clone();
        }
        if let Some(ref a) = self.acceleration {
            celestial.physical_object.acceleration = a.clone();
        }
    }

    pub fn merge(mut deltas: Vec<CelestialDelta>) -> Option<CelestialDelta> {
        if deltas.is_empty() {
            return None;
        }
        deltas.sort_by_key(|d| d.sequence);
        let mut merged = deltas[0].clone();
        for d in deltas.into_iter().skip(1) {
            if d.position.is_some() {
                merged.position = d.position.clone();
            }
            if d.orientation.is_some() {
                merged.orientation = d.orientation.clone();
            }
            if d.velocity.is_some() {
                merged.velocity = d.velocity.clone();
            }
            if d.acceleration.is_some() {
                merged.acceleration = d.acceleration.clone();
            }
            merged.timestamp = d.timestamp;
            merged.sequence = d.sequence;
        }
        Some(merged)
    }

    pub fn is_empty(&self) -> bool {
        self.position.is_none()
            && self.orientation.is_none()
            && self.velocity.is_none()
            && self.acceleration.is_none()
    }

    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
