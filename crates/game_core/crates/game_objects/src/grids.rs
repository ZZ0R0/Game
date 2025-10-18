// grids.rs — minimal, delta physique centralisé dans objects::PhysicalObjectDelta
use crate::blocks::{Block, BlockDelta};
use crate::physics::{
    FloatOrientation, FloatPosition, PhysicalObject, PhysicalObjectDelta, RectBounds, Velocity,
};
use crate::utils::arena::{Arena, HasId};
use ahash::AHasher;
use rand::{rng, Rng};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Hash)]
pub enum GridSizeClass {
    Small,
    Large,
}

#[derive(Debug, Clone)]
pub struct GridId(pub u32);
impl GridId {
    pub fn new() -> Self {
        let mut r = rng();
        Self(r.random())
    }
}

#[derive(Clone)]
pub struct Grid {
    pub id: GridId,
    pub name: String,
    pub physical_object: PhysicalObject,
    pub size_class: GridSizeClass,
    pub blocks: Arena<Block, u32>, // in_grid_id = u32
    pub boundaries: RectBounds,
    pub hash: u64,
    pub pending_deltas: Vec<GridDelta>,
    pub player_controlled: bool,
}

impl HasId<u32> for Grid {
    fn id_ref(&self) -> &u32 {
        &self.id.0
    }
}

impl Grid {
    fn calculate_hash(&self) -> u64 {
        let mut h = AHasher::default();
        self.id.0.hash(&mut h);
        self.name.hash(&mut h);
        self.size_class.hash(&mut h);
        (self.physical_object.placed_object.position.x as u32).hash(&mut h);
        (self.physical_object.placed_object.position.y as u32).hash(&mut h);
        (self.physical_object.placed_object.position.z as u32).hash(&mut h);
        h.finish()
    }
    pub fn update_hash(&mut self) {
        self.hash = self.calculate_hash();
    }

    pub fn update_mass(&mut self) -> f32 {
        let mut total = 0.0;
        for (_id, b) in self.blocks.iter() {
            total += b.current_mass;
        }
        self.physical_object.mass = total;
        total
    }

    pub fn new(
        id: u32,
        name: String,
        physical_object: PhysicalObject,
        size_class: GridSizeClass,
        blocks: impl IntoIterator<Item = Block>,
    ) -> Self {
        let mut arena = Arena::<Block, u32>::new();
        for b in blocks {
            arena.insert(b);
        }
        let mut grid = Self {
            id: GridId(id),
            name,
            physical_object,
            size_class,
            blocks: arena,
            boundaries: RectBounds::null(),
            hash: 0,
            pending_deltas: Vec::new(),
            player_controlled: false,
        };
        grid.hash = grid.calculate_hash();
        grid
    }

    pub fn undefined() -> Self {
        Self {
            id: GridId(0),
            name: String::new(),
            physical_object: PhysicalObject::undefined(),
            size_class: GridSizeClass::Small,
            blocks: Arena::<Block, u32>::new(),
            boundaries: RectBounds::null(),
            hash: 0,
            pending_deltas: Vec::new(),
            player_controlled: false,
        }
    }

    pub fn add_block(&mut self, block: Block) {
        let d = block.distance_to_grid_center();
        if d.delta_x < self.boundaries.x_min {
            self.boundaries.x_min = d.delta_x;
        }
        if d.delta_x > self.boundaries.x_max {
            self.boundaries.x_max = d.delta_x;
        }
        if d.delta_y < self.boundaries.y_min {
            self.boundaries.y_min = d.delta_y;
        }
        if d.delta_y > self.boundaries.y_max {
            self.boundaries.y_max = d.delta_y;
        }
        if d.delta_z < self.boundaries.z_min {
            self.boundaries.z_min = d.delta_z;
        }
        if d.delta_z > self.boundaries.z_max {
            self.boundaries.z_max = d.delta_z;
        }
        self.blocks.insert(block);
        self.update_hash();
    }

    pub fn record_delta(&mut self, delta: GridDelta) {
        self.pending_deltas.push(delta);
    }

    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<GridDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = GridDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }

    // Helpers de création basés sur PhysicalObjectDelta
    pub fn create_position_delta(&self, p: FloatPosition, ts: u64, seq: u64) -> GridDelta {
        GridDelta {
            grid_id: self.id.0,
            physics: PhysicalObjectDelta::empty(ts, seq).with_position(p),
            blocks_delta: HashMap::new(),
        }
    }
    pub fn create_orientation_delta(&self, o: FloatOrientation, ts: u64, seq: u64) -> GridDelta {
        GridDelta {
            grid_id: self.id.0,
            physics: PhysicalObjectDelta::empty(ts, seq).with_orientation(o),
            blocks_delta: HashMap::new(),
        }
    }
    pub fn create_velocity_delta(&self, v: Velocity, ts: u64, seq: u64) -> GridDelta {
        GridDelta {
            grid_id: self.id.0,
            physics: PhysicalObjectDelta::empty(ts, seq).with_velocity(v),
            blocks_delta: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridDelta {
    pub grid_id: u32,
    pub physics: PhysicalObjectDelta,
    pub blocks_delta: HashMap<u32, BlockDelta>,
}
impl GridDelta {
    pub fn empty(grid_id: u32, ts: u64, seq: u64) -> Self {
        Self {
            grid_id,
            physics: PhysicalObjectDelta::empty(ts, seq),
            blocks_delta: HashMap::new(),
        }
    }

    pub fn apply_to(&self, g: &mut Grid) {
        self.physics.apply_to(&mut g.physical_object);
        for (bid, bdelta) in &self.blocks_delta {
            if let Some(b) = g.blocks.get_mut(*bid) {
                bdelta.apply_to(b);
            }
        }
        g.update_hash();
    }

    pub fn merge(mut v: Vec<GridDelta>) -> Option<GridDelta> {
        if v.is_empty() {
            return None;
        }
        v.sort_by_key(|d| d.physics.sequence);
        let mut m = v.remove(0);
        for d in v {
            if let Some(p) = PhysicalObjectDelta::merge(vec![m.physics.clone(), d.physics.clone()])
            {
                m.physics = p;
            }
            m.blocks_delta.extend(d.blocks_delta);
        }
        Some(m)
    }

    pub fn is_empty(&self) -> bool {
        self.physics.is_empty() && self.blocks_delta.is_empty()
    }

    pub fn estimated_size(&self) -> usize {
        let mut s = std::mem::size_of::<Self>();
        s += self
            .blocks_delta
            .values()
            .map(|b| b.estimated_size())
            .sum::<usize>();
        s
    }

    pub fn add_block_delta(&mut self, d: BlockDelta) {
        self.blocks_delta.insert(d.in_grid_id, d);
    }
}
