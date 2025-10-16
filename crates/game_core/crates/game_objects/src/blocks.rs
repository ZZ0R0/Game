// ...existing code...
use crate::objects::{IntDistance, IntOrientation, IntPosition};
use crate::grids::Grid;
use std::hash::Hash;
use std::sync::Arc;
use std::collections::HashMap;

// Modules de définition de blocks par catégorie
mod armor;
mod power;
mod production;
mod thrusters;
mod weapons;
mod utility;
mod cockpits;

// Module principal regroupant tout (BlockRegistry)
pub mod assets;

// BlockId: conserve la distinction Large/Small (Large(1) != Small(1))
#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlockId {
    Large(u32),
    Small(u32),
}

impl BlockId {
    pub fn value(&self) -> u32 {
        match self {
            BlockId::Large(v) | BlockId::Small(v) => *v,
        }
    }

    pub fn is_large(&self) -> bool {
        matches!(self, BlockId::Large(_))
    }
    pub fn is_small(&self) -> bool {
        matches!(self, BlockId::Small(_))
    }

    /// clé unique encodant le type + valeur (utile pour maps/DB)
    pub fn unique_key(&self) -> u64 {
        let type_id: u64 = match self {
            BlockId::Large(_) => 1,
            BlockId::Small(_) => 2,
        };
        (type_id << 32) | (self.value() as u64)
    }
}

/// Faces d'un block (convention: relative à l'orientation par défaut)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlockFace {
    Front,  // +Z
    Back,   // -Z
    Left,   // -X
    Right,  // +X
    Top,    // +Y
    Bottom, // -Y
}

impl BlockFace {
    /// Retourne la face opposée
    pub fn opposite(&self) -> BlockFace {
        match self {
            BlockFace::Front => BlockFace::Back,
            BlockFace::Back => BlockFace::Front,
            BlockFace::Left => BlockFace::Right,
            BlockFace::Right => BlockFace::Left,
            BlockFace::Top => BlockFace::Bottom,
            BlockFace::Bottom => BlockFace::Top,
        }
    }

    pub fn is_opposite(&self, other: BlockFace) -> bool {
        self.opposite() == other
    }
}

/// Zone de connexion sur une face (coords normalisées 0..1)
/// Exemple: une face de 6x6 subdivisions où seules 2 zones sont connectables
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MountPoint {
    /// Position start (x, y) sur la face en coordonnées normalisées (0..1)
    /// x=0 = bord gauche, x=1 = bord droit
    /// y=0 = bord bas, y=1 = bord haut
    pub start: (f32, f32),
    /// Taille (width, height) en coordonnées normalisées (0..1)
    pub size: (f32, f32),
}

impl MountPoint {
    pub fn new(start: (f32, f32), size: (f32, f32)) -> Self {
        Self { start, size }
    }

    /// Mount point couvrant toute la face (1x1)
    pub fn full_face() -> Self {
        Self::new((0.0, 0.0), (1.0, 1.0))
    }

    /// Crée un mount point à partir de subdivisions (grid-based)
    /// Ex: pour une face 6x6, subface en (2,3) de taille 2x1 :
    /// from_grid(2, 3, 2, 1, 6, 6) -> start=(0.33, 0.5), size=(0.33, 0.16)
    pub fn from_grid(start_x: u32, start_y: u32, width: u32, height: u32, grid_w: u32, grid_h: u32) -> Self {
        Self {
            start: (start_x as f32 / grid_w as f32, start_y as f32 / grid_h as f32),
            size: (width as f32 / grid_w as f32, height as f32 / grid_h as f32),
        }
    }

    /// Vérifie si deux mount points se chevauchent (même face)
    pub fn overlaps(&self, other: &MountPoint) -> bool {
        let (x1, y1) = self.start;
        let (w1, h1) = self.size;
        let (x2, y2) = other.start;
        let (w2, h2) = other.size;

        !(x1 + w1 <= x2 || x2 + w2 <= x1 || y1 + h1 <= y2 || y2 + h2 <= y1)
    }

    /// Calcule l'aire de chevauchement avec un autre mount point
    pub fn overlap_area(&self, other: &MountPoint) -> f32 {
        if !self.overlaps(other) {
            return 0.0;
        }

        let (x1, y1) = self.start;
        let (w1, h1) = self.size;
        let (x2, y2) = other.start;
        let (w2, h2) = other.size;

        let overlap_x = (x1 + w1).min(x2 + w2) - x1.max(x2);
        let overlap_y = (y1 + h1).min(y2 + h2) - y1.max(y2);

        overlap_x * overlap_y
    }
}

/// Référence au modèle 3D (à adapter selon ton moteur de rendu)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model3DRef {
    /// Chemin/ID du modèle (ex: "models/blocks/armor_large.glb")
    pub path: String,
    /// Échelle optionnelle
    pub scale: f32,
}

impl Model3DRef {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into(), scale: 1.0 }
    }
}

pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Items stockés dans un inventaire
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InventoryItem {
    pub item_id: String,
    pub quantity: f32,
    pub volume_per_unit: f32,
}

/// Items en production
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductionItem {
    pub blueprint_id: String,
    pub quantity: u32,
    pub progress: f32,
}

/// Composants optionnels qu'un block peut avoir (système de components comme Space Engineers)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BlockComponent {
    /// Inventaire (cargo, coffre)
    Inventory {
        max_volume: f32,        // m³
        current_volume: f32,
        items: Vec<InventoryItem>,
    },
    /// Production d'énergie (réacteur, panneau solaire)
    PowerProducer {
        max_output: f32,        // MW
        current_output: f32,
        efficiency: f32,        // 0.0-1.0
        fuel_consumption: f32,  // unités/s
    },
    /// Consommation d'énergie
    PowerConsumer {
        required_power: f32,    // MW
        current_draw: f32,
        is_powered: bool,
    },
    /// Stockage d'énergie (batterie)
    PowerStorage {
        max_capacity: f32,      // MWh
        current_charge: f32,
        charge_rate: f32,
        discharge_rate: f32,
    },
    /// Production (assembleur, raffinerie)
    Producer {
        production_queue: Vec<ProductionItem>,
        production_speed: f32,
        current_progress: f32,
    },
    /// Propulsion (thruster)
    Thruster {
        max_force: f32,         // Newtons
        current_force: f32,
        direction: BlockFace,   // Direction de poussée
        fuel_efficiency: f32,
    },
    /// Arme (tourelle, lance-missile)
    Weapon {
        damage: f32,
        range: f32,
        fire_rate: f32,
        ammo_type: String,
        current_ammo: u32,
    },
    /// Contrôle (cockpit, siège)
    Control {
        can_pilot: bool,
        has_occupant: bool,
        occupant_id: Option<u64>,
    },
}

/// BlockDef = modèle / référence contenant toutes les propriétés "fixes"
#[derive(Debug, Clone)]
pub struct BlockDef {
    pub id: BlockId,
    pub name: String,
    /// footprint en cellules de grille (width, height, depth)
    pub footprint: (i32, i32, i32),
    pub mass: f32,
    pub integrity: f32,
    pub block_type: String,
    
    /// Mount points par face (HashMap<BlockFace, Vec<MountPoint>>)
    /// Chaque face peut avoir 0..N mount points (zones connectables)
    pub mount_points: HashMap<BlockFace, Vec<MountPoint>>,
    
    /// Modèle 3D associé
    pub model: Model3DRef,
    
    /// Composants disponibles pour ce type de block (template)
    pub available_components: Vec<String>,
}

impl BlockDef {
    pub fn new(
        id: BlockId,
        name: impl Into<String>,
        footprint: (i32, i32, i32),
        mass: f32,
        integrity: f32,
        block_type: impl Into<String>,
        model: Model3DRef,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            footprint,
            mass,
            integrity,
            block_type: block_type.into(),
            mount_points: HashMap::new(),
            model,
            available_components: Vec::new(),
        }
    }

    /// Ajoute un mount point à une face
    pub fn add_mount_point(&mut self, face: BlockFace, mount_point: MountPoint) {
        self.mount_points.entry(face).or_insert_with(Vec::new).push(mount_point);
    }
    
    /// Définit les composants disponibles pour ce type de block (builder pattern)
    pub fn with_available_components(mut self, components: Vec<String>) -> Self {
        self.available_components = components;
        self
    }
    
    /// Ajoute un composant disponible (builder pattern)
    pub fn with_component(mut self, component_name: impl Into<String>) -> Self {
        self.available_components.push(component_name.into());
        self
    }

    /// Définit toutes les faces comme entièrement connectables (cube plein)
    pub fn set_full_cube_mounts(&mut self) {
        for face in [BlockFace::Front, BlockFace::Back, BlockFace::Left, 
                     BlockFace::Right, BlockFace::Top, BlockFace::Bottom] {
            self.add_mount_point(face, MountPoint::full_face());
        }
    }

    /// Vérifie si deux blocks peuvent se connecter (faces opposées + overlap)
    pub fn can_connect_to(&self, self_face: BlockFace, other: &BlockDef, other_face: BlockFace) -> bool {
        // Vérifie que les faces sont opposées
        if !self_face.is_opposite(other_face) {
            return false;
        }

        let self_mounts = self.mount_points.get(&self_face);
        let other_mounts = other.mount_points.get(&other_face);

        match (self_mounts, other_mounts) {
            (Some(sm), Some(om)) => {
                // Au moins un mount point doit se chevaucher
                sm.iter().any(|s| om.iter().any(|o| s.overlaps(o)))
            }
            _ => false,
        }
    }
}

/// Block = instance placée qui référence un BlockDef (référence partagée)
#[derive(Debug, Clone)]
pub struct Block {
    pub in_grid_id: u64,
    pub ref_grid: Grid,
    pub def: Arc<BlockDef>, 
    pub current_integrity: f32,
    pub current_mass: f32,
    pub position: IntPosition,
    pub orientation: IntOrientation,
    /// États actuels des composants (valeurs modifiables au runtime)
    pub components: HashMap<String, BlockComponent>,
    /// Liste des changements en attente (pour le système de delta)
    pub pending_deltas: Vec<BlockDelta>,
}

impl Block {
    /// create a new instance from a definition and a relative object (position/orientation)
    pub fn new(in_grid_id: u64, def: Arc<BlockDef>, ref_grid: Grid, position: IntPosition, orientation: IntOrientation, integrity: f32) -> Self {
        Self {
            in_grid_id: in_grid_id,
            ref_grid: ref_grid,
            position:  position,
            orientation: orientation,
            current_integrity: integrity,
            current_mass: def.mass,
            def: def,
            components: HashMap::new(),
            pending_deltas: Vec::new(),
        }
    }

    /// Vérifie si le block peut tenir et si l'espace est libre dans la grille.
    /// Nécessite que Grid fournisse : in_bounds(origin, size) et is_area_free(origin, size)
    pub fn can_place_at_grid_coords(&self, _grid: &Grid, position: BlockPosition) -> bool {
        let _footprint = self.def.footprint;
        let _origin = (position.x, position.y, position.z);
        // TODO: Implement is_area_free in Grid
        // grid.is_area_free(origin, footprint)
        true
    }

    pub fn distance_to_grid_center(&self) -> IntDistance{
        IntDistance::between(&self.position, &IntPosition::zero())
    }

    /// Met à jour un composant spécifique
    pub fn update_component(&mut self, component_type: &str, new_state: BlockComponent) {
        self.components.insert(component_type.to_string(), new_state);
    }
    
    /// Récupère un composant spécifique
    pub fn get_component(&self, component_type: &str) -> Option<&BlockComponent> {
        self.components.get(component_type)
    }
    
    /// Récupère un composant mutable
    pub fn get_component_mut(&mut self, component_type: &str) -> Option<&mut BlockComponent> {
        self.components.get_mut(component_type)
    }
    
    /// Ajoute plusieurs composants à la fois (builder pattern)
    pub fn with_components(mut self, components: Vec<(&str, BlockComponent)>) -> Self {
        for (key, component) in components {
            self.components.insert(key.to_string(), component);
        }
        self
    }
    
    /// Vérifie si le block a un composant spécifique
    pub fn has_component(&self, component_type: &str) -> bool {
        self.components.contains_key(component_type)
    }
    
    /// Enregistre un changement à appliquer plus tard
    pub fn record_delta(&mut self, delta: BlockDelta) {
        self.pending_deltas.push(delta);
    }
    
    /// Applique tous les deltas en attente et retourne le delta global fusionné
    /// Cette méthode fusionne tous les deltas accumulés, les applique au block,
    /// et retourne le delta final pour transmission réseau
    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<BlockDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        
        // Fusionner tous les deltas
        let merged = BlockDelta::merge(self.pending_deltas.clone());
        
        // Appliquer le delta fusionné
        if let Some(ref delta) = merged {
            delta.apply_to(self);
        }
        
        // Vider la file des deltas en attente
        self.pending_deltas.clear();
        
        merged
    }
    
    /// Crée un delta pour un changement d'intégrité
    pub fn create_integrity_delta(&self, new_integrity: f32, timestamp: u64, sequence: u64) -> BlockDelta {
        BlockDelta {
            in_grid_id: self.in_grid_id,
            integrity: Some(new_integrity),
            mass: None,
            component_changes: HashMap::new(),
            timestamp,
            sequence,
        }
    }
    
    /// Crée un delta pour un changement de masse
    pub fn create_mass_delta(&self, new_mass: f32, timestamp: u64, sequence: u64) -> BlockDelta {
        BlockDelta {
            in_grid_id: self.in_grid_id,
            integrity: None,
            mass: Some(new_mass),
            component_changes: HashMap::new(),
            timestamp,
            sequence,
        }
    }
    
    /// Crée un delta pour un changement de composant
    pub fn create_component_delta(&self, component_key: &str, change: ComponentDelta, timestamp: u64, sequence: u64) -> BlockDelta {
        let mut component_changes = HashMap::new();
        component_changes.insert(component_key.to_string(), change);
        
        BlockDelta {
            in_grid_id: self.in_grid_id,
            integrity: None,
            mass: None,
            component_changes,
            timestamp,
            sequence,
        }
    }
}


/// Delta pour un composant spécifique
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComponentDelta {
    /// Composant ajouté
    Added(BlockComponent),
    /// Composant supprimé
    Removed,
    /// Composant modifié (changements partiels)
    Modified(ComponentChange),
}

/// Changements spécifiques par type de composant
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComponentChange {
    Inventory {
        volume_change: Option<f32>,
        items_added: Vec<InventoryItem>,
        items_removed: Vec<String>,
    },
    PowerProducer {
        output_change: Option<f32>,
        efficiency_change: Option<f32>,
    },
    PowerStorage {
        charge_change: Option<f32>,
    },
    PowerConsumer {
        power_draw_change: Option<f32>,
        is_powered_change: Option<bool>,
    },
    Producer {
        queue_change: Option<Vec<ProductionItem>>,
        progress_change: Option<f32>,
    },
    Thruster {
        force_change: Option<f32>,
    },
    Weapon {
        ammo_change: Option<u32>,
    },
    Control {
        occupant_change: Option<Option<u64>>,
    },
}

/// Delta représentant les changements d'un block entre deux états
/// 
/// Ce delta ne contient PAS la position ni l'orientation car un block
/// ne bouge pas au sein d'une grille. Ces informations sont fixes.
/// 
/// Le `in_grid_id` identifie de manière unique le block au sein de sa grille.
/// C'est différent du `block_id` (BlockId) qui identifie le TYPE de block
/// (ex: Large(100) = Battery). Le `in_grid_id` est l'identifiant de l'INSTANCE
/// spécifique du block dans la grille (ex: la battery #42 sur ce vaisseau).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockDelta {
    /// ID unique du block dans sa grille (identifiant de l'instance)
    pub in_grid_id: u64,
    
    /// Intégrité (None si inchangée)
    pub integrity: Option<f32>,
    
    /// Masse (None si inchangée) - peut changer avec l'inventaire
    pub mass: Option<f32>,
    
    /// Changements dans les composants
    pub component_changes: HashMap<String, ComponentDelta>,
    
    /// Timestamp du delta (pour ordering)
    pub timestamp: u64,
    
    /// Numéro de séquence (pour garantir l'ordre)
    pub sequence: u64,
}

impl BlockDelta {
    /// Crée un delta vide
    pub fn empty(in_grid_id: u64, timestamp: u64, sequence: u64) -> Self {
        Self {
            in_grid_id,
            integrity: None,
            mass: None,
            component_changes: HashMap::new(),
            timestamp,
            sequence,
        }
    }
    
    /// Applique le delta à un block
    pub fn apply_to(&self, block: &mut Block) {
        if let Some(integrity) = self.integrity {
            block.current_integrity = integrity;
        }
        
        if let Some(mass) = self.mass {
            block.current_mass = mass;
        }
        
        // Appliquer changements de composants
        for (key, change) in &self.component_changes {
            match change {
                ComponentDelta::Added(comp) => {
                    block.components.insert(key.clone(), comp.clone());
                }
                ComponentDelta::Removed => {
                    block.components.remove(key);
                }
                ComponentDelta::Modified(comp_change) => {
                    if let Some(comp) = block.components.get_mut(key) {
                        Self::apply_component_change(comp, comp_change);
                    }
                }
            }
        }
    }
    
    /// Applique un changement de composant
    fn apply_component_change(component: &mut BlockComponent, change: &ComponentChange) {
        match (component, change) {
            (BlockComponent::Inventory { current_volume, items, .. },
             ComponentChange::Inventory { volume_change, items_added, items_removed }) => {
                if let Some(vol) = volume_change {
                    *current_volume = *vol;
                }
                items.extend(items_added.clone());
                items.retain(|item| !items_removed.contains(&item.item_id));
            }
            (BlockComponent::PowerStorage { current_charge, .. },
             ComponentChange::PowerStorage { charge_change }) => {
                if let Some(charge) = charge_change {
                    *current_charge = *charge;
                }
            }
            (BlockComponent::PowerProducer { current_output, efficiency, .. },
             ComponentChange::PowerProducer { output_change, efficiency_change }) => {
                if let Some(output) = output_change {
                    *current_output = *output;
                }
                if let Some(eff) = efficiency_change {
                    *efficiency = *eff;
                }
            }
            (BlockComponent::PowerConsumer { current_draw, is_powered, .. },
             ComponentChange::PowerConsumer { power_draw_change, is_powered_change }) => {
                if let Some(draw) = power_draw_change {
                    *current_draw = *draw;
                }
                if let Some(powered) = is_powered_change {
                    *is_powered = *powered;
                }
            }
            (BlockComponent::Thruster { current_force, .. },
             ComponentChange::Thruster { force_change }) => {
                if let Some(force) = force_change {
                    *current_force = *force;
                }
            }
            (BlockComponent::Weapon { current_ammo, .. },
             ComponentChange::Weapon { ammo_change }) => {
                if let Some(ammo) = ammo_change {
                    *current_ammo = *ammo;
                }
            }
            (BlockComponent::Control { occupant_id, has_occupant, .. },
             ComponentChange::Control { occupant_change }) => {
                if let Some(occ) = occupant_change {
                    *occupant_id = *occ;
                    *has_occupant = occ.is_some();
                }
            }
            (BlockComponent::Producer { current_progress, production_queue, .. },
             ComponentChange::Producer { queue_change, progress_change }) => {
                if let Some(prog) = progress_change {
                    *current_progress = *prog;
                }
                if let Some(queue) = queue_change {
                    *production_queue = queue.clone();
                }
            }
            _ => {}
        }
    }
    
    /// Fusionne plusieurs deltas en séquence
    /// Prend en compte l'ordre chronologique (par sequence) et applique les changements
    /// de manière cumulative pour créer un seul delta final
    pub fn merge(deltas: Vec<BlockDelta>) -> Option<BlockDelta> {
        if deltas.is_empty() {
            return None;
        }
        
        // Trier par séquence pour garantir l'ordre chronologique
        let mut sorted = deltas;
        sorted.sort_by_key(|d| d.sequence);
        
        let mut merged = sorted[0].clone();
        
        for delta in sorted.iter().skip(1) {
            // Prendre la dernière valeur pour chaque champ
            if delta.integrity.is_some() {
                merged.integrity = delta.integrity;
            }
            if delta.mass.is_some() {
                merged.mass = delta.mass;
            }
            
            // Fusionner composants (dernière valeur gagne)
            for (key, change) in &delta.component_changes {
                merged.component_changes.insert(key.clone(), change.clone());
            }
            
            merged.timestamp = delta.timestamp;
            merged.sequence = delta.sequence;
        }
        
        Some(merged)
    }
    
    /// Vérifie si le delta est vide (aucun changement)
    pub fn is_empty(&self) -> bool {
        self.integrity.is_none() &&
        self.mass.is_none() &&
        self.component_changes.is_empty()
    }
    
    /// Taille estimée en octets (pour optimisation réseau)
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        size += self.component_changes.len() * 64; // Estimation
        size
    }
}

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
