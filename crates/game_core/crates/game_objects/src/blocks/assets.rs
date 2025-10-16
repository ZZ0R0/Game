/// Module pour les définitions de blocks du jeu
/// 
/// Ce module contient toutes les définitions de blocks (BlockDef) organisées par catégorie.
/// Chaque catégorie a son propre sous-module (armor, power, production, etc.)

use crate::blocks::{BlockDef, BlockId};
use std::sync::Arc;
use std::collections::HashMap;

// Import des modules de définition de blocks depuis le parent
use super::armor;
use super::power;
use super::production;
use super::thrusters;
use super::weapons;
use super::utility;
use super::cockpits;

// Optionnel: module d'exemples (déjà dans le parent)
// pub use super::examples;

/// Registry central de tous les blocks du jeu
pub struct BlockRegistry {
    blocks: HashMap<BlockId, Arc<BlockDef>>,
}

impl BlockRegistry {
    /// Crée un nouveau registry et enregistre tous les blocks
    pub fn new() -> Self {
        let mut registry = Self {
            blocks: HashMap::new(),
        };
        
        // Enregistrer tous les blocks par catégorie
        registry.register_armor_blocks();
        registry.register_power_blocks();
        registry.register_production_blocks();
        registry.register_thruster_blocks();
        registry.register_weapon_blocks();
        registry.register_utility_blocks();
        registry.register_cockpit_blocks();
        
        registry
    }
    
    /// Enregistre un block dans le registry
    pub fn register(&mut self, def: BlockDef) {
        let id = def.id.clone();
        self.blocks.insert(id, Arc::new(def));
    }
    
    /// Récupère un block par son ID
    pub fn get(&self, id: &BlockId) -> Option<Arc<BlockDef>> {
        self.blocks.get(id).cloned()
    }
    
    /// Liste tous les blocks disponibles
    pub fn list_all(&self) -> Vec<Arc<BlockDef>> {
        self.blocks.values().cloned().collect()
    }
    
    /// Liste les blocks d'un type spécifique
    pub fn list_by_type(&self, block_type: &str) -> Vec<Arc<BlockDef>> {
        self.blocks
            .values()
            .filter(|def| def.block_type == block_type)
            .cloned()
            .collect()
    }
    
    // Méthodes d'enregistrement par catégorie
    fn register_armor_blocks(&mut self) {
        for def in armor::create_all() {
            self.register(def);
        }
    }
    
    fn register_power_blocks(&mut self) {
        for def in power::create_all() {
            self.register(def);
        }
    }
    
    fn register_production_blocks(&mut self) {
        for def in production::create_all() {
            self.register(def);
        }
    }
    
    fn register_thruster_blocks(&mut self) {
        for def in thrusters::create_all() {
            self.register(def);
        }
    }
    
    fn register_weapon_blocks(&mut self) {
        for def in weapons::create_all() {
            self.register(def);
        }
    }
    
    fn register_utility_blocks(&mut self) {
        for def in utility::create_all() {
            self.register(def);
        }
    }
    
    fn register_cockpit_blocks(&mut self) {
        for def in cockpits::create_all() {
            self.register(def);
        }
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}
