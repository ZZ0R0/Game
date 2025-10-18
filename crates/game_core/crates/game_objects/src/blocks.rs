// blocks.rs — Block stockable en arène (HasId<u32>), aucune référence forte à Grid
use crate::factions::FactionId;
use crate::physics::{IntPositionDelta, IntOrientation, IntPosition};
use crate::utils::arena::HasId;
use std::collections::HashMap;
use std::sync::Arc;

// Modules de définition
mod armor;
mod cockpits;
mod power;
mod production;
mod thrusters;
mod utility;
mod weapons;

// Registry
pub mod assets;

// -------- Types de base

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
    pub fn unique_key(&self) -> u64 {
        let type_id: u64 = if self.is_large() { 1 } else { 2 };
        (type_id << 32) | (self.value() as u64)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlockFace {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}
impl BlockFace {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MountPoint {
    pub start: (f32, f32),
    pub size: (f32, f32),
}
impl MountPoint {
    pub fn new(start: (f32, f32), size: (f32, f32)) -> Self {
        Self { start, size }
    }
    pub fn full_face() -> Self {
        Self::new((0.0, 0.0), (1.0, 1.0))
    }
    pub fn from_grid(sx: u32, sy: u32, w: u32, h: u32, gw: u32, gh: u32) -> Self {
        Self {
            start: (sx as f32 / gw as f32, sy as f32 / gh as f32),
            size: (w as f32 / gw as f32, h as f32 / gh as f32),
        }
    }
    pub fn overlaps(&self, o: &MountPoint) -> bool {
        let (x1, y1) = self.start;
        let (w1, h1) = self.size;
        let (x2, y2) = o.start;
        let (w2, h2) = o.size;
        !(x1 + w1 <= x2 || x2 + w2 <= x1 || y1 + h1 <= y2 || y2 + h2 <= y1)
    }
    pub fn overlap_area(&self, o: &MountPoint) -> f32 {
        if !self.overlaps(o) {
            return 0.0;
        }
        let (x1, y1) = self.start;
        let (w1, h1) = self.size;
        let (x2, y2) = o.start;
        let (w2, h2) = o.size;
        let ox = (x1 + w1).min(x2 + w2) - x1.max(x2);
        let oy = (y1 + h1).min(y2 + h2) - y1.max(y2);
        ox * oy
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model3DRef {
    pub path: String,
    pub scale: f32,
}
impl Model3DRef {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            scale: 1.0,
        }
    }
}

pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

// -------- Composants

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InventoryItem {
    pub item_id: String,
    pub quantity: f32,
    pub volume_per_unit: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductionItem {
    pub blueprint_id: String,
    pub quantity: u32,
    pub progress: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BlockComponent {
    Inventory {
        max_volume: f32,
        current_volume: f32,
        items: Vec<InventoryItem>,
    },
    PowerProducer {
        max_output: f32,
        current_output: f32,
        efficiency: f32,
        fuel_consumption: f32,
    },
    PowerConsumer {
        required_power: f32,
        current_draw: f32,
        is_powered: bool,
    },
    PowerStorage {
        max_capacity: f32,
        current_charge: f32,
        charge_rate: f32,
        discharge_rate: f32,
    },
    Producer {
        production_queue: Vec<ProductionItem>,
        production_speed: f32,
        current_progress: f32,
    },
    Thruster {
        max_force: f32,
        current_force: f32,
        direction: BlockFace,
        fuel_efficiency: f32,
    },
    Weapon {
        damage: f32,
        range: f32,
        fire_rate: f32,
        ammo_type: String,
        current_ammo: u32,
    },
    Control {
        can_pilot: bool,
        has_occupant: bool,
        occupant_id: Option<u64>,
    },
}

#[derive(Debug, Clone)]
pub struct BlockDef {
    pub id: BlockId,
    pub name: String,
    pub footprint: (i32, i32, i32),
    pub mass: f32,
    pub integrity: f32,
    pub block_type: String,
    pub mount_points: HashMap<BlockFace, Vec<MountPoint>>,
    pub model: Model3DRef,
    pub available_components: Vec<String>,
}
impl BlockDef {
    pub fn new(
        id: BlockId,
        name: impl Into<String>,
        fp: (i32, i32, i32),
        mass: f32,
        integ: f32,
        kind: impl Into<String>,
        model: Model3DRef,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            footprint: fp,
            mass,
            integrity: integ,
            block_type: kind.into(),
            mount_points: HashMap::new(),
            model,
            available_components: Vec::new(),
        }
    }
    pub fn add_mount_point(&mut self, f: BlockFace, m: MountPoint) {
        self.mount_points.entry(f).or_default().push(m);
    }
    pub fn with_available_components(mut self, c: Vec<String>) -> Self {
        self.available_components = c;
        self
    }
    pub fn with_component(mut self, name: impl Into<String>) -> Self {
        self.available_components.push(name.into());
        self
    }
    pub fn set_full_cube_mounts(&mut self) {
        for f in [
            BlockFace::Front,
            BlockFace::Back,
            BlockFace::Left,
            BlockFace::Right,
            BlockFace::Top,
            BlockFace::Bottom,
        ] {
            self.add_mount_point(f, MountPoint::full_face());
        }
    }
    pub fn can_connect_to(
        &self,
        self_face: BlockFace,
        other: &BlockDef,
        other_face: BlockFace,
    ) -> bool {
        if !self_face.is_opposite(other_face) {
            return false;
        }
        match (
            self.mount_points.get(&self_face),
            other.mount_points.get(&other_face),
        ) {
            (Some(sm), Some(om)) => sm.iter().any(|s| om.iter().any(|o| s.overlaps(o))),
            _ => false,
        }
    }
}

// -------- Block instance + deltas

#[derive(Debug, Clone)]
pub struct Block {
    pub in_grid_id: u32, // ID instance dans la grille
    pub grid_id: u32,    // Grid propriétaire
    pub def: Arc<BlockDef>,
    pub current_integrity: f32,
    pub current_mass: f32,
    pub position: IntPosition,
    pub orientation: IntOrientation,
    pub components: HashMap<String, BlockComponent>,
    pub pending_deltas: Vec<BlockDelta>,
    pub faction_id: FactionId,
}

// Pour Arena<Block, u32>
impl HasId<u32> for Block {
    #[inline]
    fn id_ref(&self) -> &u32 {
        &self.in_grid_id
    }
}

impl Block {
    pub fn new(
        in_grid_id: u32,
        grid_id: u32,
        def: Arc<BlockDef>,
        position: IntPosition,
        orientation: IntOrientation,
        integrity: f32,
        faction_id: FactionId,
    ) -> Self {
        Self {
            in_grid_id: in_grid_id,
            grid_id: grid_id,
            current_mass: def.mass,
            def: def,
            position: position,
            orientation: orientation,
            current_integrity: integrity,
            components: HashMap::new(),
            pending_deltas: Vec::new(),
            faction_id: faction_id,
        }
    }

    pub fn can_place_at_grid_coords(&self, _grid_id: u32, _pos: BlockPosition) -> bool {
        true
    }

    pub fn distance_to_grid_center(&self) -> IntPositionDelta {
        IntPositionDelta::between(&self.position, &IntPosition::zero())
    }

    pub fn update_component(&mut self, ty: &str, new_state: BlockComponent) {
        self.components.insert(ty.to_string(), new_state);
    }
    pub fn get_component(&self, ty: &str) -> Option<&BlockComponent> {
        self.components.get(ty)
    }
    pub fn get_component_mut(&mut self, ty: &str) -> Option<&mut BlockComponent> {
        self.components.get_mut(ty)
    }
    pub fn with_components(mut self, comps: Vec<(&str, BlockComponent)>) -> Self {
        for (k, c) in comps {
            self.components.insert(k.to_string(), c);
        }
        self
    }
    pub fn has_component(&self, ty: &str) -> bool {
        self.components.contains_key(ty)
    }

    pub fn record_delta(&mut self, d: BlockDelta) {
        self.pending_deltas.push(d);
    }
    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<BlockDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = BlockDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }

    pub fn create_integrity_delta(&self, integrity: f32, ts: u64, seq: u64) -> BlockDelta {
        BlockDelta {
            in_grid_id: self.in_grid_id,
            integrity: Some(integrity),
            mass: None,
            component_changes: HashMap::new(),
            timestamp: ts,
            sequence: seq,
        }
    }
    pub fn create_mass_delta(&self, mass: f32, ts: u64, seq: u64) -> BlockDelta {
        BlockDelta {
            in_grid_id: self.in_grid_id,
            integrity: None,
            mass: Some(mass),
            component_changes: HashMap::new(),
            timestamp: ts,
            sequence: seq,
        }
    }
    pub fn create_component_delta(
        &self,
        key: &str,
        change: ComponentDelta,
        ts: u64,
        seq: u64,
    ) -> BlockDelta {
        let mut component_changes = HashMap::new();
        component_changes.insert(key.to_string(), change);
        BlockDelta {
            in_grid_id: self.in_grid_id,
            integrity: None,
            mass: None,
            component_changes,
            timestamp: ts,
            sequence: seq,
        }
    }
}

// ---- Deltas

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComponentDelta {
    Added(BlockComponent),
    Removed,
    Modified(ComponentChange),
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockDelta {
    pub in_grid_id: u32,
    pub integrity: Option<f32>,
    pub mass: Option<f32>,
    pub component_changes: HashMap<String, ComponentDelta>,
    pub timestamp: u64,
    pub sequence: u64,
}

impl BlockDelta {
    pub fn empty(in_grid_id: u32, timestamp: u64, sequence: u64) -> Self {
        Self {
            in_grid_id,
            integrity: None,
            mass: None,
            component_changes: HashMap::new(),
            timestamp,
            sequence,
        }
    }

    pub fn apply_to(&self, block: &mut Block) {
        if let Some(v) = self.integrity {
            block.current_integrity = v;
        }
        if let Some(m) = self.mass {
            block.current_mass = m;
        }
        for (k, ch) in &self.component_changes {
            match ch {
                ComponentDelta::Added(comp) => {
                    block.components.insert(k.clone(), comp.clone());
                }
                ComponentDelta::Removed => {
                    block.components.remove(k);
                }
                ComponentDelta::Modified(mc) => {
                    if let Some(comp) = block.components.get_mut(k) {
                        Self::apply_component_change(comp, mc);
                    }
                }
            }
        }
    }

    fn apply_component_change(c: &mut BlockComponent, ch: &ComponentChange) {
        match (c, ch) {
            (
                BlockComponent::Inventory {
                    current_volume,
                    items,
                    ..
                },
                ComponentChange::Inventory {
                    volume_change,
                    items_added,
                    items_removed,
                },
            ) => {
                if let Some(v) = volume_change {
                    *current_volume = *v;
                }
                items.extend(items_added.clone());
                items.retain(|it| !items_removed.contains(&it.item_id));
            }
            (
                BlockComponent::PowerStorage { current_charge, .. },
                ComponentChange::PowerStorage { charge_change },
            ) => {
                if let Some(v) = charge_change {
                    *current_charge = *v;
                }
            }
            (
                BlockComponent::PowerProducer {
                    current_output,
                    efficiency,
                    ..
                },
                ComponentChange::PowerProducer {
                    output_change,
                    efficiency_change,
                },
            ) => {
                if let Some(v) = output_change {
                    *current_output = *v;
                }
                if let Some(e) = efficiency_change {
                    *efficiency = *e;
                }
            }
            (
                BlockComponent::PowerConsumer {
                    current_draw,
                    is_powered,
                    ..
                },
                ComponentChange::PowerConsumer {
                    power_draw_change,
                    is_powered_change,
                },
            ) => {
                if let Some(v) = power_draw_change {
                    *current_draw = *v;
                }
                if let Some(p) = is_powered_change {
                    *is_powered = *p;
                }
            }
            (
                BlockComponent::Thruster { current_force, .. },
                ComponentChange::Thruster { force_change },
            ) => {
                if let Some(v) = force_change {
                    *current_force = *v;
                }
            }
            (
                BlockComponent::Weapon { current_ammo, .. },
                ComponentChange::Weapon { ammo_change },
            ) => {
                if let Some(v) = ammo_change {
                    *current_ammo = *v;
                }
            }
            (
                BlockComponent::Control {
                    occupant_id,
                    has_occupant,
                    ..
                },
                ComponentChange::Control { occupant_change },
            ) => {
                if let Some(o) = occupant_change {
                    *occupant_id = *o;
                    *has_occupant = o.is_some();
                }
            }
            (
                BlockComponent::Producer {
                    current_progress,
                    production_queue,
                    ..
                },
                ComponentChange::Producer {
                    queue_change,
                    progress_change,
                },
            ) => {
                if let Some(v) = progress_change {
                    *current_progress = *v;
                }
                if let Some(q) = queue_change {
                    *production_queue = q.clone();
                }
            }
            _ => {}
        }
    }

    pub fn merge(mut deltas: Vec<BlockDelta>) -> Option<BlockDelta> {
        if deltas.is_empty() {
            return None;
        }
        deltas.sort_by_key(|d| d.sequence);
        let mut merged = deltas[0].clone();
        for d in deltas.into_iter().skip(1) {
            if d.integrity.is_some() {
                merged.integrity = d.integrity;
            }
            if d.mass.is_some() {
                merged.mass = d.mass;
            }
            merged.component_changes.extend(d.component_changes);
            merged.timestamp = d.timestamp;
            merged.sequence = d.sequence;
        }
        Some(merged)
    }

    pub fn is_empty(&self) -> bool {
        self.integrity.is_none() && self.mass.is_none() && self.component_changes.is_empty()
    }

    pub fn estimated_size(&self) -> usize {
        let mut s = std::mem::size_of::<Self>();
        s += self.component_changes.len() * 64;
        s
    }
}
