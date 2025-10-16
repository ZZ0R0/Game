use crate::objects::{PhysicalObject, FloatPosition, FloatOrientation, Velocity, Acceleration};
use std::collections::HashMap;

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
pub struct CelestialBody {
    pub id: CelestialId,
    pub name: String,
    pub celestial_type: CelestialType,
    pub physical: PhysicalObject,
    pub radius: f32,
    pub gravity_strength: f32,
    pub atmosphere: bool,
    /// Liste des changements en attente (pour le système de delta)
    pub pending_deltas: Vec<CelestialDelta>,
}

impl CelestialBody {
    pub fn new(
        id: u32,
        name: String,
        celestial_type: CelestialType,
        physical: PhysicalObject,
        radius: f32,
        gravity_strength: f32,
        atmosphere: bool,
    ) -> Self {
        Self {
            id: CelestialId(id),
            name,
            celestial_type,
            physical,
            radius,
            gravity_strength,
            atmosphere,
            pending_deltas: Vec::new(),
        }
    }

    pub fn earth_like_planet(id: u32) -> Self {
        Self::new(
            id,
            "Earth-like Planet".to_string(),
            CelestialType::Planet,
            PhysicalObject::default(),
            60000.0, // 60km radius
            9.81,    // Earth gravity
            true,
        )
    }

    pub fn small_moon(id: u32) -> Self {
        Self::new(
            id,
            "Small Moon".to_string(),
            CelestialType::Moon,
            PhysicalObject::default(),
            10000.0, // 10km radius
            1.62,    // Moon-like gravity
            false,
        )
    }

    pub fn asteroid(id: u32) -> Self {
        Self::new(
            id,
            "Asteroid".to_string(),
            CelestialType::Asteroid,
            PhysicalObject::default(),
            500.0, // 500m radius
            0.1,   // Very low gravity
            false,
        )
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
    pub fn apply_to(&self, celestial: &mut CelestialBody) {
        if let Some(ref pos) = self.position {
            celestial.physical.placed.position = pos.clone();
        }
        
        if let Some(ref orient) = self.orientation {
            celestial.physical.placed.orientation = orient.clone();
        }
        
        if let Some(ref vel) = self.velocity {
            celestial.physical.velocity = vel.clone();
        }
        
        if let Some(ref accel) = self.acceleration {
            celestial.physical.acceleration = accel.clone();
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