use crate::objects::{PhysicalObject, FloatPosition, FloatOrientation, Velocity, Acceleration};

#[derive(Debug, Clone)]
pub struct CelestialId(pub u32);

#[derive(Debug, Clone)]
pub enum CelestialType {
    Planet,
    Moon,
    Asteroid,
    Star,
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
    /// Liste des changements en attente (pour le système de delta)
    pub pending_deltas: Vec<CelestialDelta>,
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

    pub fn get_position(&self) -> &FloatPosition {
        &self.physical_object.placed_object.position
    }

    pub fn get_orientation(&self) -> &FloatOrientation {
        &self.physical_object.placed_object.orientation
    }

    pub fn get_velocity(&self) -> &Velocity {
        &self.physical_object.velocity
    }

    pub fn get_acceleration(&self) -> &Acceleration {
        &self.physical_object.acceleration
    }

    pub fn set_position(&mut self, position: FloatPosition) {
        self.physical_object.placed_object.position = position;
    }

    pub fn set_orientation(&mut self, orientation: FloatOrientation) {
        self.physical_object.placed_object.orientation = orientation;
    }

    pub fn set_velocity(&mut self, velocity: Velocity) {
        self.physical_object.velocity = velocity;
    }

    pub fn set_acceleration(&mut self, acceleration: Acceleration) {
        self.physical_object.acceleration = acceleration;
    }
    
    /// Enregistre un changement à appliquer plus tard
    pub fn record_delta(&mut self, delta: CelestialDelta) {
        self.pending_deltas.push(delta);
    }
    
    /// Applique tous les deltas en attente et retourne le delta global fusionné
    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<CelestialDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        
        let merged = CelestialDelta::merge(self.pending_deltas.clone());
        
        if let Some(ref delta) = merged {
            delta.apply_to(self);
        }
        
        self.pending_deltas.clear();
        merged
    }
    
    /// Crée un delta pour un changement de position (pour les corps en mouvement)
    pub fn create_position_delta(&self, new_position: FloatPosition, timestamp: u64, sequence: u64) -> CelestialDelta {
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
    
    /// Crée un delta pour un changement de rotation
    pub fn create_rotation_delta(&self, new_orientation: FloatOrientation, timestamp: u64, sequence: u64) -> CelestialDelta {
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

/// Delta représentant les changements d'un corps céleste entre deux états
/// 
/// Les corps célestes peuvent bouger (orbites) et tourner sur eux-mêmes.
/// Ce delta permet de synchroniser ces mouvements sur le réseau.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CelestialDelta {
    /// ID du corps céleste concerné
    pub celestial_id: u32,
    
    /// Position dans l'espace (None si inchangée)
    pub position: Option<FloatPosition>,
    
    /// Orientation/Rotation (None si inchangée)
    pub orientation: Option<FloatOrientation>,
    
    /// Vélocité orbitale (None si inchangée)
    pub velocity: Option<Velocity>,
    
    /// Accélération (None si inchangée)
    pub acceleration: Option<Acceleration>,
    
    /// Timestamp du delta
    pub timestamp: u64,
    
    /// Numéro de séquence
    pub sequence: u64,
}

impl CelestialDelta {
    /// Crée un delta vide
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
    
    /// Applique le delta à un corps céleste
    pub fn apply_to(&self, celestial: &mut Celestial) {
        if let Some(ref pos) = self.position {
            celestial.physical_object.placed_object.position = pos.clone();
        }
        
        if let Some(ref orient) = self.orientation {
            celestial.physical_object.placed_object.orientation = orient.clone();
        }
        
        if let Some(ref vel) = self.velocity {
            celestial.physical_object.velocity = vel.clone();
        }
        
        if let Some(ref accel) = self.acceleration {
            celestial.physical_object.acceleration = accel.clone();
        }
    }
    
    /// Fusionne plusieurs deltas en séquence
    pub fn merge(deltas: Vec<CelestialDelta>) -> Option<CelestialDelta> {
        if deltas.is_empty() {
            return None;
        }
        
        let mut sorted = deltas;
        sorted.sort_by_key(|d| d.sequence);
        
        let mut merged = sorted[0].clone();
        
        for delta in sorted.iter().skip(1) {
            if delta.position.is_some() {
                merged.position = delta.position.clone();
            }
            if delta.orientation.is_some() {
                merged.orientation = delta.orientation.clone();
            }
            if delta.velocity.is_some() {
                merged.velocity = delta.velocity.clone();
            }
            if delta.acceleration.is_some() {
                merged.acceleration = delta.acceleration.clone();
            }
            
            merged.timestamp = delta.timestamp;
            merged.sequence = delta.sequence;
        }
        
        Some(merged)
    }
    
    /// Vérifie si le delta est vide
    pub fn is_empty(&self) -> bool {
        self.position.is_none() &&
        self.orientation.is_none() &&
        self.velocity.is_none() &&
        self.acceleration.is_none()
    }
    
    /// Taille estimée en octets
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}