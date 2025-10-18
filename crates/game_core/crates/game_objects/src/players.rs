use crate::objects::{PhysicalObject, FloatPosition, FloatOrientation, Velocity, Acceleration};
use crate::factions::FactionId;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u32);

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub physical_object: PhysicalObject,
    pub health: f32,
    pub oxygen: f32,
    pub hydrogen: f32,
    pub energy: f32,
    /// Liste des changements en attente (pour le système de delta)
    pub pending_deltas: Vec<PlayerDelta>,

    pub faction_id: FactionId,
}

impl Player {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id: PlayerId(id),
            name,
            physical_object: PhysicalObject::undefined(),
            health: 100.0,
            oxygen: 100.0,
            hydrogen: 100.0,
            energy: 100.0,
            pending_deltas: Vec::new(),
            faction_id: FactionId(0),
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

    pub fn spawn_at(&mut self, position: FloatPosition, orientation: FloatOrientation) {
        self.physical_object.placed_object.position = position;
        self.physical_object.placed_object.orientation = orientation;
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(100.0);
    }

    /// Update player resources (oxygen, hydrogen, energy) over time
    pub fn update_resources(&mut self, delta_time: f32) {
        // Consume oxygen over time (1% per second when not in atmosphere)
        self.oxygen = (self.oxygen - delta_time).max(0.0);
        
        // Energy drains slowly (0.5% per second)
        self.energy = (self.energy - delta_time * 0.5).max(0.0);
        
        // Take damage if oxygen is too low
        if self.oxygen <= 0.0 {
            self.take_damage(delta_time * 10.0); // 10 damage per second without oxygen
        }
    }

    /// Refill oxygen (e.g., when in pressurized area)
    pub fn refill_oxygen(&mut self) {
        self.oxygen = 100.0;
    }

    /// Refill energy (e.g., when near power source)
    pub fn refill_energy(&mut self) {
        self.energy = 100.0;
    }

    /// Check if player needs critical resources
    pub fn needs_oxygen(&self) -> bool {
        self.oxygen < 20.0
    }

    pub fn needs_energy(&self) -> bool {
        self.energy < 20.0
    }

    /// Move player to a new position
    pub fn move_to(&mut self, position: FloatPosition) {
        self.physical_object.placed_object.position = position;
    }
    
    /// Enregistre un changement à appliquer plus tard
    pub fn record_delta(&mut self, delta: PlayerDelta) {
        self.pending_deltas.push(delta);
    }
    
    /// Applique tous les deltas en attente et retourne le delta global fusionné
    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<PlayerDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        
        let merged = PlayerDelta::merge(self.pending_deltas.clone());
        
        if let Some(ref delta) = merged {
            delta.apply_to(self);
        }
        
        self.pending_deltas.clear();
        merged
    }
    
    /// Crée un delta pour un changement de position
    pub fn create_position_delta(&self, new_position: FloatPosition, timestamp: u64, sequence: u64) -> PlayerDelta {
        PlayerDelta {
            player_id: self.id.0,
            position: Some(new_position),
            orientation: None,
            velocity: None,
            health: None,
            oxygen: None,
            hydrogen: None,
            energy: None,
            timestamp,
            sequence,
        }
    }
    
    /// Crée un delta pour les ressources (health, oxygen, hydrogen, energy)
    pub fn create_resources_delta(&self, timestamp: u64, sequence: u64) -> PlayerDelta {
        PlayerDelta {
            player_id: self.id.0,
            position: None,
            orientation: None,
            velocity: None,
            health: Some(self.health),
            oxygen: Some(self.oxygen),
            hydrogen: Some(self.hydrogen),
            energy: Some(self.energy),
            timestamp,
            sequence,
        }
    }
}

/// Delta représentant les changements d'un joueur entre deux états
/// 
/// Le PlayerDelta contient tous les aspects qui peuvent changer pour un joueur :
/// - Position et mouvement dans l'espace
/// - Ressources vitales (santé, oxygène, énergie)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerDelta {
    /// ID du joueur concerné
    pub player_id: u32,
    
    /// Position (None si inchangée)
    pub position: Option<FloatPosition>,
    
    /// Orientation (None si inchangée)
    pub orientation: Option<FloatOrientation>,
    
    /// Vélocité (None si inchangée)
    pub velocity: Option<Velocity>,
    
    /// Santé (None si inchangée)
    pub health: Option<f32>,
    
    /// Oxygène (None si inchangée)
    pub oxygen: Option<f32>,
    
    /// Hydrogène (None si inchangée)
    pub hydrogen: Option<f32>,
    
    /// Énergie (None si inchangée)
    pub energy: Option<f32>,
    
    /// Timestamp du delta
    pub timestamp: u64,
    
    /// Numéro de séquence
    pub sequence: u64,
}

impl PlayerDelta {
    /// Crée un delta vide
    pub fn empty(player_id: u32, timestamp: u64, sequence: u64) -> Self {
        Self {
            player_id,
            position: None,
            orientation: None,
            velocity: None,
            health: None,
            oxygen: None,
            hydrogen: None,
            energy: None,
            timestamp,
            sequence,
        }
    }
    
    /// Applique le delta à un joueur
    pub fn apply_to(&self, player: &mut Player) {
        if let Some(ref pos) = self.position {
            player.physical_object.placed_object.position = pos.clone();
        }
        
        if let Some(ref orient) = self.orientation {
            player.physical_object.placed_object.orientation = orient.clone();
        }
        
        if let Some(ref vel) = self.velocity {
            player.physical_object.velocity = vel.clone();
        }
        
        if let Some(health) = self.health {
            player.health = health;
        }
        
        if let Some(oxygen) = self.oxygen {
            player.oxygen = oxygen;
        }
        
        if let Some(hydrogen) = self.hydrogen {
            player.hydrogen = hydrogen;
        }
        
        if let Some(energy) = self.energy {
            player.energy = energy;
        }
    }
    
    /// Fusionne plusieurs deltas en séquence
    pub fn merge(deltas: Vec<PlayerDelta>) -> Option<PlayerDelta> {
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
            if delta.health.is_some() {
                merged.health = delta.health;
            }
            if delta.oxygen.is_some() {
                merged.oxygen = delta.oxygen;
            }
            if delta.hydrogen.is_some() {
                merged.hydrogen = delta.hydrogen;
            }
            if delta.energy.is_some() {
                merged.energy = delta.energy;
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
        self.health.is_none() &&
        self.oxygen.is_none() &&
        self.hydrogen.is_none() &&
        self.energy.is_none()
    }
    
    /// Taille estimée en octets
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}