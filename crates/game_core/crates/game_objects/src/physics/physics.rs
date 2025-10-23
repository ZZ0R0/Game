use crate::utils::ids::MapId;
use crate::utils::mapping::pos_to_map_id;
use crate::physics::metrics::{FloatPosition, Velocity, Acceleration, FloatOrientation};
use crate::physics::boundaries::Boundaries;


/* -------------------- Physical object -------------------- */

#[derive(Debug, Clone)]
pub struct PhysicalObject {
    pub timestamp: Option<u64>,
    pub position: Option<FloatPosition>,
    pub orientation: Option<FloatOrientation>,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
    pub mass: Option<f32>,
    pub boundaries: Option<Boundaries>,
    pub map_id: Option<MapId>,
    pub pending_deltas: Vec<PhysicalObjectDelta>,
}

impl PhysicalObject {
    // Constructeur pur: ne fait que construire.
    pub fn new(
        timestamp: Option<u64>,
        position: Option<FloatPosition>,
        orientation: Option<FloatOrientation>,
        velocity: Option<Velocity>,
        acceleration: Option<Acceleration>,
        mass: Option<f32>,
        boundaries: Option<Boundaries>,
        map_id: Option<MapId>,
        pending_deltas: Vec<PhysicalObjectDelta>,
    ) -> Self {
        Self {
            timestamp,
            position,
            orientation,
            velocity,
            acceleration,
            mass,
            boundaries,
            map_id,
            pending_deltas,
        }
    }

    pub fn record_delta(&mut self, delta: PhysicalObjectDelta) {
        self.pending_deltas.push(delta);
    }

    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<PhysicalObjectDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = PhysicalObjectDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }

    pub fn update_map_id(&mut self) {
        if let Some(ref pos) = self.position {
            let ip = pos.to_int_position();
            let m: &mut MapId = self.map_id.get_or_insert(MapId::undefined());
            pos_to_map_id(&ip, m);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalObjectDelta {
    pub timestamp: Option<u64>,
    pub position: Option<FloatPosition>,
    pub orientation: Option<FloatOrientation>,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
    pub mass: Option<f32>,
    pub boundaries: Option<Boundaries>,
    pub mapp_id: Option<MapId>,
}

impl PhysicalObjectDelta {
    pub fn merge(mut deltas: Vec<PhysicalObjectDelta>) -> Option<PhysicalObjectDelta> {
        if deltas.is_empty() {
            return None;
        }
        deltas.sort_by_key(|d| d.timestamp);
        let mut m = deltas.remove(0);
        for d in deltas {
            if d.position.is_some() {
                m.position = d.position;
            }
            if d.orientation.is_some() {
                m.orientation = d.orientation;
            }
            if d.velocity.is_some() {
                m.velocity = d.velocity;
            }
            if d.acceleration.is_some() {
                m.acceleration = d.acceleration;
            }
            if d.mass.is_some() {
                m.mass = d.mass;
            }
            if d.boundaries.is_some() {
                m.boundaries = d.boundaries;
            }
            if d.mapp_id.is_some() {
                m.mapp_id = d.mapp_id;
            }
            if d.timestamp.is_some() {
                m.timestamp = d.timestamp;
            }
        }
        Some(m)
    }

    pub fn apply_to(&self, e: &mut PhysicalObject) {
        if let Some(ts) = self.timestamp {
            e.timestamp = Some(ts);
        }
        if let Some(p) = self.position.clone() {
            e.position = Some(p);
        }
        if let Some(o) = self.orientation.clone() {
            e.orientation = Some(o);
        }
        if let Some(v) = self.velocity.clone() {
            e.velocity = Some(v);
        }
        if let Some(a) = self.acceleration.clone() {
            e.acceleration = Some(a);
        }
        if let Some(m) = self.mass {
            e.mass = Some(m);
        }
        if let Some(b) = self.boundaries.clone() {
            e.boundaries = Some(b);
        }
        if let Some(id) = self.mapp_id {
            e.map_id = Some(id);
        }
    }
}
