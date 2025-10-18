use crate::objects::{PhysicalObject, FloatPosition, FloatOrientation, Velocity, Acceleration};
use crate::blocks::{Block, BlockDelta};
use ahash::AHasher;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use rand::{Rng, rng};


#[derive(Debug, Clone, Hash)]
pub struct Boundaries {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
    pub z_min: i32,
    pub z_max: i32,
}

impl Boundaries {
    pub fn null() -> Self {
        Self {
            x_min: 0,
            x_max: 0,
            y_min: 0,
            y_max: 0,
            z_min: 0,
            z_max: 0,
        }
    }
}


#[derive(Debug, Clone, Hash)]
pub enum GridSizeClass {
    Small,
    Large,
}



#[derive(Debug, Clone)]
pub struct GridId(pub u32);

impl GridId {
    pub fn new() -> Self {
        use Rng;
        let mut rng = rng();
        Self(rng.random())
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub id: GridId,
    pub name: String,
    pub physical: PhysicalObject,
    pub size_class: GridSizeClass,
    pub blocks: Vec<Block>,
    pub boundaries: Boundaries,
    pub hash: u64,
    /// Liste des changements en attente (pour le système de delta)
    pub pending_deltas: Vec<GridDelta>,
}

impl Grid {
    fn calculate_hash(&self) -> u64 {
        let mut hasher = AHasher::default();
        self.id.0.hash(&mut hasher);
        self.name.hash(&mut hasher);
        self.size_class.hash(&mut hasher);
        // Position et autres propriétés physiques importantes
        (self.physical.placed.position.x as u32).hash(&mut hasher);
        (self.physical.placed.position.y as u32).hash(&mut hasher);
        (self.physical.placed.position.z as u32).hash(&mut hasher);
        hasher.finish()
    }

    pub fn update_hash(&mut self) {
        self.hash = self.calculate_hash();
    }

    pub fn update_mass(&mut self) -> f32 {
        let mut total_mass: f32 = 0.0;
        for block in &self.blocks {
            total_mass += block.current_mass;
        }
        // assign the computed mass to the physical object, then return it
        self.physical.mass = total_mass;
        total_mass
    }
}

impl Grid {
    pub fn new(id: u32, name: String, physical: PhysicalObject, size_class: GridSizeClass, blocks: Vec<Block>) -> Self {
        let mut grid = Self {
            id: GridId(id),
            name,
            physical,
            size_class,
            blocks,
            boundaries: Boundaries::null(),
            hash: 0,
            pending_deltas: Vec::new(),
        };
        grid.hash = grid.calculate_hash();
        grid
    }
    
    pub fn add_block(&mut self, block: Block) {
        let center_distance = block.distance_to_grid_center();
        if center_distance.x < self.boundaries.x_min {
            self.boundaries.x_min = center_distance.x;
        }
        if center_distance.x > self.boundaries.x_max {
            self.boundaries.x_max = center_distance.x;
        }
        if center_distance.y < self.boundaries.y_min {
            self.boundaries.y_min = center_distance.y;
        }
        if center_distance.y > self.boundaries.y_max {
            self.boundaries.y_max = center_distance.y;
        }
        if center_distance.z < self.boundaries.z_min {
            self.boundaries.z_min = center_distance.z;
        }
        if center_distance.z > self.boundaries.z_max {
            self.boundaries.z_max = center_distance.z;
        }
        self.blocks.push(block);
        self.update_hash();
    }
    
    /// Enregistre un changement à appliquer plus tard
    pub fn record_delta(&mut self, delta: GridDelta) {
        self.pending_deltas.push(delta);
    }
    
    /// Applique tous les deltas en attente et retourne le delta global fusionné
    /// Cette méthode fusionne tous les deltas accumulés, les applique à la grille,
    /// et retourne le delta final pour transmission réseau
    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<GridDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        
        // Fusionner tous les deltas
        let merged = GridDelta::merge(self.pending_deltas.clone());
        
        // Appliquer le delta fusionné
        if let Some(ref delta) = merged {
            delta.apply_to(self);
        }
        
        // Vider la file des deltas en attente
        self.pending_deltas.clear();
        
        merged
    }
    
    /// Crée un delta pour un changement de position
    pub fn create_position_delta(&self, new_position: FloatPosition, timestamp: u64, sequence: u64) -> GridDelta {
        GridDelta {
            grid_id: self.id.0,
            position: Some(new_position),
            orientation: None,
            velocity: None,
            acceleration: None,
            mass: None,
            blocks_delta: HashMap::new(),
            timestamp,
            sequence,
        }
    }
    
    /// Crée un delta pour un changement d'orientation
    pub fn create_orientation_delta(&self, new_orientation: FloatOrientation, timestamp: u64, sequence: u64) -> GridDelta {
        GridDelta {
            grid_id: self.id.0,
            position: None,
            orientation: Some(new_orientation),
            velocity: None,
            acceleration: None,
            mass: None,
            blocks_delta: HashMap::new(),
            timestamp,
            sequence,
        }
    }
    
    /// Crée un delta pour un changement de vélocité
    pub fn create_velocity_delta(&self, new_velocity: Velocity, timestamp: u64, sequence: u64) -> GridDelta {
        GridDelta {
            grid_id: self.id.0,
            position: None,
            orientation: None,
            velocity: Some(new_velocity),
            acceleration: None,
            mass: None,
            blocks_delta: HashMap::new(),
            timestamp,
            sequence,
        }
    }
}

/// Delta représentant les changements d'une grille entre deux états
/// 
/// Le GridDelta contient :
/// - Les changements de propriétés physiques (position, orientation, vélocité, etc.)
/// - Les deltas de tous les blocks qui ont changé (par leur in_grid_id)
/// 
/// Ce système permet de synchroniser efficacement l'état d'un vaisseau complet
/// sur le réseau en n'envoyant que les changements.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridDelta {
    /// ID de la grille concernée
    pub grid_id: u32,
    
    /// Position dans l'espace (None si inchangée)
    pub position: Option<FloatPosition>,
    
    /// Orientation dans l'espace (None si inchangée)
    pub orientation: Option<FloatOrientation>,
    
    /// Vélocité (None si inchangée)
    pub velocity: Option<Velocity>,
    
    /// Accélération (None si inchangée)
    pub acceleration: Option<Acceleration>,
    
    /// Masse totale (None si inchangée)
    pub mass: Option<f32>,
    
    /// Changements des blocks (HashMap<in_grid_id, BlockDelta>)
    /// Contient uniquement les blocks qui ont changé
    pub blocks_delta: HashMap<u64, BlockDelta>,
    
    /// Timestamp du delta (pour ordering)
    pub timestamp: u64,
    
    /// Numéro de séquence (pour garantir l'ordre)
    pub sequence: u64,
}

impl GridDelta {
    /// Crée un delta vide
    pub fn empty(grid_id: u32, timestamp: u64, sequence: u64) -> Self {
        Self {
            grid_id,
            position: None,
            orientation: None,
            velocity: None,
            acceleration: None,
            mass: None,
            blocks_delta: HashMap::new(),
            timestamp,
            sequence,
        }
    }
    
    /// Applique le delta à une grille
    pub fn apply_to(&self, grid: &mut Grid) {
        // Appliquer les changements physiques
        if let Some(ref pos) = self.position {
            grid.physical.placed.position = pos.clone();
        }
        
        if let Some(ref orient) = self.orientation {
            grid.physical.placed.orientation = orient.clone();
        }
        
        if let Some(ref vel) = self.velocity {
            grid.physical.velocity = vel.clone();
        }
        
        if let Some(ref accel) = self.acceleration {
            grid.physical.acceleration = accel.clone();
        }
        
        if let Some(mass) = self.mass {
            grid.physical.mass = mass;
        }
        
        // Appliquer les changements de blocks
        for (block_in_grid_id, block_delta) in &self.blocks_delta {
            // Trouver le block correspondant dans la grille
            if let Some(block) = grid.blocks.iter_mut().find(|b| b.in_grid_id == *block_in_grid_id) {
                block_delta.apply_to(block);
            }
        }
        
        // Mettre à jour le hash de la grille
        grid.update_hash();
    }
    
    /// Fusionne plusieurs deltas en séquence
    /// Prend en compte l'ordre chronologique (par sequence) et applique les changements
    /// de manière cumulative pour créer un seul delta final
    pub fn merge(deltas: Vec<GridDelta>) -> Option<GridDelta> {
        if deltas.is_empty() {
            return None;
        }
        
        // Trier par séquence pour garantir l'ordre chronologique
        let mut sorted = deltas;
        sorted.sort_by_key(|d| d.sequence);
        
        let mut merged = sorted[0].clone();
        
        for delta in sorted.iter().skip(1) {
            // Prendre la dernière valeur pour chaque champ physique
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
            if delta.mass.is_some() {
                merged.mass = delta.mass;
            }
            
            // Fusionner les deltas de blocks
            // Pour chaque block, on prend le dernier delta disponible
            for (block_id, block_delta) in &delta.blocks_delta {
                merged.blocks_delta.insert(*block_id, block_delta.clone());
            }
            
            merged.timestamp = delta.timestamp;
            merged.sequence = delta.sequence;
        }
        
        Some(merged)
    }
    
    /// Vérifie si le delta est vide (aucun changement)
    pub fn is_empty(&self) -> bool {
        self.position.is_none() &&
        self.orientation.is_none() &&
        self.velocity.is_none() &&
        self.acceleration.is_none() &&
        self.mass.is_none() &&
        self.blocks_delta.is_empty()
    }
    
    /// Taille estimée en octets (pour optimisation réseau)
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        // Ajouter la taille de chaque BlockDelta
        for block_delta in self.blocks_delta.values() {
            size += block_delta.estimated_size();
        }
        size
    }
    
    /// Ajoute un BlockDelta au GridDelta
    pub fn add_block_delta(&mut self, block_delta: BlockDelta) {
        self.blocks_delta.insert(block_delta.in_grid_id, block_delta);
    }
}