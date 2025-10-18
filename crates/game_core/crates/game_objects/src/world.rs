// world.rs — version corrigée: IDs u32, arènes génériques, deltas filtrables
use crate::utils::arena::{Arena, HasId};
use crate::grids::{Grid, GridDelta};
use crate::humanoids::{Humanoid, HumanoidDelta};
use crate::celestials::{Celestial, CelestialDelta};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct World {
    pub seed: u64,
    pub name: String,
    pub time: f64,

    // Arènes autoritatives (IDs = u32)
    pub grids: Arena<Grid, u32>,
    pub players: Arena<Humanoid, u32>,
    pub celestials: Arena<Celestial, u32>,

    // File de deltas en attente
    pub pending_deltas: Vec<WorldDelta>,
}

impl World {
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            seed,
            name,
            time: 0.0,
            grids: Arena::new(),
            players: Arena::new(),
            celestials: Arena::new(),
            pending_deltas: Vec::new(),
        }
    }

    // Insertions: les objets portent déjà leur Id u32
    pub fn insert_grid(&mut self, g: Grid) -> u32 { self.grids.insert(g) }
    pub fn insert_player(&mut self, p: Humanoid) -> u32 { self.players.insert(p) }
    pub fn insert_celestial(&mut self, c: Celestial) -> u32 { self.celestials.insert(c) }

    /// Empile un delta
    pub fn record_delta(&mut self, delta: WorldDelta) {
        self.pending_deltas.push(delta);
    }

    /// Fusionne, applique, et retourne le delta fusionné
    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<WorldDelta> {
        if self.pending_deltas.is_empty() { return None; }
        let merged = WorldDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged { d.apply_to(self); }
        merged
    }
}

/// WorldDelta: toutes les clés d’entités en u32
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDelta {
    pub time: Option<f64>,

    /// HashMap<grid_id, GridDelta>
    pub grids_delta: HashMap<u32, GridDelta>,

    /// HashMap<player_id, HumanoidDelta>
    pub players_delta: HashMap<u32, HumanoidDelta>,

    /// HashMap<celestial_id, CelestialDelta>
    pub celestials_delta: HashMap<u32, CelestialDelta>,

    pub timestamp: u64,
    pub sequence: u64,
}

impl WorldDelta {
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

    /// Application autoritative côté serveur
    pub fn apply_to(&self, world: &mut World) {
        if let Some(t) = self.time { world.time = t; }

        for (gid, gdelta) in &self.grids_delta {
            if let Some(g) = world.grids.get_mut(*gid) {
                gdelta.apply_to(g);
            }
        }
        for (pid, pdelta) in &self.players_delta {
            if let Some(p) = world.players.get_mut(*pid) {
                pdelta.apply_to(p);
            }
        }
        for (cid, cdelta) in &self.celestials_delta {
            if let Some(c) = world.celestials.get_mut(*cid) {
                cdelta.apply_to(c);
            }
        }
    }

    /// Fusion “last write wins” par séquence
    pub fn merge(mut deltas: Vec<WorldDelta>) -> Option<WorldDelta> {
        if deltas.is_empty() { return None; }
        deltas.sort_by_key(|d| d.sequence);

        let mut merged = deltas[0].clone();
        for d in deltas.into_iter().skip(1) {
            if d.time.is_some() { merged.time = d.time; }
            merged.grids_delta.extend(d.grids_delta);
            merged.players_delta.extend(d.players_delta);
            merged.celestials_delta.extend(d.celestials_delta);
            merged.timestamp = d.timestamp;
            merged.sequence  = d.sequence;
        }
        Some(merged)
    }

    pub fn is_empty(&self) -> bool {
        self.time.is_none()
            && self.grids_delta.is_empty()
            && self.players_delta.is_empty()
            && self.celestials_delta.is_empty()
    }

    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        size += self.grids_delta.values().map(|g| g.estimated_size()).sum::<usize>();
        size += self.players_delta.values().map(|p| p.estimated_size()).sum::<usize>();
        size += self.celestials_delta.values().map(|c| c.estimated_size()).sum::<usize>();
        size
    }

    pub fn add_grid_delta(&mut self, g: GridDelta) {
        self.grids_delta.insert(g.grid_id, g);
    }
    pub fn add_player_delta(&mut self, h: HumanoidDelta) {
        self.players_delta.insert(h.player_id, h);
    }
    pub fn add_celestial_delta(&mut self, c: CelestialDelta) {
        self.celestials_delta.insert(c.celestial_id, c);
    }

    pub fn count_modified_entities(&self) -> usize {
        self.grids_delta.len() + self.players_delta.len() + self.celestials_delta.len()
    }
}