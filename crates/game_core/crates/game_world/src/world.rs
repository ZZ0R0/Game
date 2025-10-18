use game_objects::{grids, players, celestials};
use serde;
use std::collections::HashMap;
use Vec;

#[derive(Debug, Clone)]
pub struct World {
    pub seed : u64,
    pub name : String,
    pub grids : Vec<grids::Grid>,
    pub players : Vec<players::Player>,
    pub celestials : Vec<celestials::CelestialBody>,
    pub time : f64,
    /// Liste des changements en attente (pour le système de delta)
    pub pending_deltas: Vec<WorldDelta>,
}

impl World {
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            seed,
            name,
            grids: Vec::new(),
            players: Vec::new(),
            celestials: Vec::new(),
            time: 0.0,
            pending_deltas: Vec::new(),
        }
    }
    
    /// Enregistre un changement à appliquer plus tard
    pub fn record_delta(&mut self, delta: WorldDelta) {
        self.pending_deltas.push(delta);
    }
    
    /// Applique tous les deltas en attente et retourne le delta global fusionné
    /// Cette méthode fusionne tous les deltas accumulés, les applique au monde,
    /// et retourne le delta final pour transmission réseau
    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<WorldDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        
        // Fusionner tous les deltas
        let merged = WorldDelta::merge(self.pending_deltas.clone());
        
        // Appliquer le delta fusionné
        if let Some(ref delta) = merged {
            delta.apply_to(self);
        }
        
        // Vider la file des deltas en attente
        self.pending_deltas.clear();
        
        merged
    }
    
    /// Crée un delta pour un changement de temps global
    pub fn create_time_delta(&self, new_time: f64, timestamp: u64, sequence: u64) -> WorldDelta {
        WorldDelta {
            time: Some(new_time),
            grids_delta: HashMap::new(),
            players_delta: HashMap::new(),
            celestials_delta: HashMap::new(),
            timestamp,
            sequence,
        }
    }
}

/// Delta représentant les changements du monde entier entre deux états
/// 
/// Le WorldDelta est le delta de plus haut niveau qui contient :
/// - Le temps de jeu global
/// - Les deltas de toutes les grilles (vaisseaux)
/// - Les deltas de tous les joueurs
/// - Les deltas de tous les corps célestes
/// 
/// Ce système permet de synchroniser l'état complet du monde sur le réseau
/// en n'envoyant que les changements, optimisant ainsi massivement la bande passante.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldDelta {
    /// Temps de jeu global (None si inchangé)
    pub time: Option<f64>,
    
    /// Changements des grilles (HashMap<grid_id, GridDelta>)
    /// Contient uniquement les grilles qui ont changé
    pub grids_delta: HashMap<u32, grids::GridDelta>,
    
    /// Changements des joueurs (HashMap<player_id, PlayerDelta>)
    /// Contient uniquement les joueurs qui ont changé
    pub players_delta: HashMap<u32, players::PlayerDelta>,
    
    /// Changements des corps célestes (HashMap<celestial_id, CelestialDelta>)
    /// Contient uniquement les corps célestes qui ont changé
    pub celestials_delta: HashMap<u32, celestials::CelestialDelta>,
    
    /// Timestamp du delta (pour ordering)
    pub timestamp: u64,
    
    /// Numéro de séquence (pour garantir l'ordre)
    pub sequence: u64,
}

impl WorldDelta {
    /// Crée un delta vide
    pub fn empty(timestamp: u64, sequence: u64) -> Self {
        Self {
            time: None,
            grids_delta: HashMap::new(),
            players_delta: HashMap::new(),
            celestials_delta: HashMap::new(),
            timestamp,
            sequence,
        }
    }
    
    /// Applique le delta au monde
    pub fn apply_to(&self, world: &mut World) {
        // Appliquer le changement de temps
        if let Some(time) = self.time {
            world.time = time;
        }
        
        // Appliquer les changements de grilles
        for (grid_id, grid_delta) in &self.grids_delta {
            if let Some(grid) = world.grids.iter_mut().find(|g| g.id.0 == *grid_id) {
                grid_delta.apply_to(grid);
            }
        }
        
        // Appliquer les changements de joueurs
        for (player_id, player_delta) in &self.players_delta {
            if let Some(player) = world.players.iter_mut().find(|p| p.id.0 == *player_id) {
                player_delta.apply_to(player);
            }
        }
        
        // Appliquer les changements de corps célestes
        for (celestial_id, celestial_delta) in &self.celestials_delta {
            if let Some(celestial) = world.celestials.iter_mut().find(|c| c.id.0 == *celestial_id) {
                celestial_delta.apply_to(celestial);
            }
        }
    }
    
    /// Fusionne plusieurs deltas en séquence
    /// Prend en compte l'ordre chronologique (par sequence) et applique les changements
    /// de manière cumulative pour créer un seul delta final
    pub fn merge(deltas: Vec<WorldDelta>) -> Option<WorldDelta> {
        if deltas.is_empty() {
            return None;
        }
        
        // Trier par séquence pour garantir l'ordre chronologique
        let mut sorted = deltas;
        sorted.sort_by_key(|d| d.sequence);
        
        let mut merged = sorted[0].clone();
        
        for delta in sorted.iter().skip(1) {
            // Prendre la dernière valeur pour le temps
            if delta.time.is_some() {
                merged.time = delta.time;
            }
            
            // Fusionner les deltas de grilles
            for (grid_id, grid_delta) in &delta.grids_delta {
                merged.grids_delta.insert(*grid_id, grid_delta.clone());
            }
            
            // Fusionner les deltas de joueurs
            for (player_id, player_delta) in &delta.players_delta {
                merged.players_delta.insert(*player_id, player_delta.clone());
            }
            
            // Fusionner les deltas de corps célestes
            for (celestial_id, celestial_delta) in &delta.celestials_delta {
                merged.celestials_delta.insert(*celestial_id, celestial_delta.clone());
            }
            
            merged.timestamp = delta.timestamp;
            merged.sequence = delta.sequence;
        }
        
        Some(merged)
    }
    
    /// Vérifie si le delta est vide (aucun changement)
    pub fn is_empty(&self) -> bool {
        self.time.is_none() &&
        self.grids_delta.is_empty() &&
        self.players_delta.is_empty() &&
        self.celestials_delta.is_empty()
    }
    
    /// Taille estimée en octets (pour optimisation réseau)
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        
        // Ajouter la taille de chaque GridDelta
        for grid_delta in self.grids_delta.values() {
            size += grid_delta.estimated_size();
        }
        
        // Ajouter la taille de chaque PlayerDelta
        for player_delta in self.players_delta.values() {
            size += player_delta.estimated_size();
        }
        
        // Ajouter la taille de chaque CelestialDelta
        for celestial_delta in self.celestials_delta.values() {
            size += celestial_delta.estimated_size();
        }
        
        size
    }
    
    /// Ajoute un GridDelta au WorldDelta
    pub fn add_grid_delta(&mut self, grid_delta: grids::GridDelta) {
        self.grids_delta.insert(grid_delta.grid_id, grid_delta);
    }
    
    /// Ajoute un PlayerDelta au WorldDelta
    pub fn add_player_delta(&mut self, player_delta: players::PlayerDelta) {
        self.players_delta.insert(player_delta.player_id, player_delta);
    }
    
    /// Ajoute un CelestialDelta au WorldDelta
    pub fn add_celestial_delta(&mut self, celestial_delta: celestials::CelestialDelta) {
        self.celestials_delta.insert(celestial_delta.celestial_id, celestial_delta);
    }
    
    /// Compte le nombre total d'entités modifiées
    pub fn count_modified_entities(&self) -> usize {
        self.grids_delta.len() + self.players_delta.len() + self.celestials_delta.len()
    }
}
 