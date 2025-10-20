use crate::utils::ids::MapId;
use crate::utils::mapping::pos_to_map_id;

use glam::{Quat, Vec3};

/* -------------------- Positions -------------------- */

#[derive(Debug, Clone)]
pub struct FloatPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl FloatPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
    pub fn to_int_position(&self) -> IntPosition {
        IntPosition::new(self.x as i32, self.y as i32, self.z as i32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl IntPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
    pub fn to_world_position(&self, grid_position: &FloatPosition) -> FloatPosition {
        FloatPosition::new(
            grid_position.x + self.x as f32 * 2.5,
            grid_position.y + self.y as f32 * 2.5,
            grid_position.z + self.z as f32 * 2.5,
        )
    }
}

/* -------------------- Metrics -------------------- */

#[derive(Debug, Clone)]
pub struct IntPositionDelta {
    pub delta_x: i32,
    pub delta_y: i32,
    pub delta_z: i32,
}
impl IntPositionDelta {
    pub fn new(delta_x: i32, delta_y: i32, delta_z: i32) -> Self {
        Self {
            delta_x,
            delta_y,
            delta_z,
        }
    }
    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }
    pub fn to_vec3(&self) -> Vec3 {
        Self::to_float_delta(&self).to_vec3()
    }
    pub fn between(a: &IntPosition, b: &IntPosition) -> Self {
        Self {
            delta_x: (b.x - a.x),
            delta_y: (b.y - a.y),
            delta_z: (b.z - a.z),
        }
    }
    pub fn to_float_delta(&self) -> FloatPositionDelta {
        FloatPositionDelta::new(
            self.delta_x as f32,
            self.delta_y as f32,
            self.delta_z as f32,
        )
    }

    pub fn eulerian_distance(&self) -> f32 {
        ((self.delta_x * self.delta_x + self.delta_y * self.delta_y + self.delta_z * self.delta_z)
            as f32)
            .sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct FloatPositionDelta {
    pub delta_x: f32,
    pub delta_y: f32,
    pub delta_z: f32,
}
impl FloatPositionDelta {
    pub fn new(delta_x: f32, delta_y: f32, delta_z: f32) -> Self {
        Self {
            delta_x,
            delta_y,
            delta_z,
        }
    }
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.delta_x, self.delta_y, self.delta_z)
    }
    pub fn between(a: &FloatPosition, b: &FloatPosition) -> Self {
        Self {
            delta_x: (b.x - a.x),
            delta_y: (b.y - a.y),
            delta_z: (b.z - a.z),
        }
    }
    pub fn to_int_delta(&self) -> IntPositionDelta {
        IntPositionDelta::new(
            self.delta_x as i32,
            self.delta_y as i32,
            self.delta_z as i32,
        )
    }
    pub fn eulerian_distance(&self) -> f32 {
        (self.delta_x * self.delta_x + self.delta_y * self.delta_y + self.delta_z * self.delta_z)
            .sqrt()
    }
}

/* -------------------- Kinematics -------------------- */

#[derive(Debug, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Velocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Acceleration {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/* -------------------- Orientation -------------------- */

#[derive(Debug, Clone)]
pub struct FloatOrientation {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}
impl FloatOrientation {
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { pitch, yaw, roll }
    }
    pub fn identity() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    pub fn to_quat(&self) -> Quat {
        Quat::from_euler(glam::EulerRot::XYZ, self.pitch, self.yaw, self.roll)
    }
}

#[derive(Debug, Clone)]
pub struct FloatOrientationDelta {
    pub delta_pitch: f32,
    pub delta_yaw: f32,
    pub delta_roll: f32,
}
impl FloatOrientationDelta {
    pub fn new(delta_pitch: f32, delta_yaw: f32, delta_roll: f32) -> Self {
        Self {
            delta_pitch,
            delta_yaw,
            delta_roll,
        }
    }
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.delta_pitch, self.delta_yaw, self.delta_roll)
    }
    pub fn between(a: &FloatPosition, b: &FloatPosition) -> Self {
        Self {
            delta_pitch: (b.x - a.x).abs(),
            delta_yaw: (b.y - a.y).abs(),
            delta_roll: (b.z - a.z).abs(),
        }
    }
    pub fn to_int_delta(&self) -> IntPositionDelta {
        IntPositionDelta::new(
            self.delta_pitch as i32,
            self.delta_yaw as i32,
            self.delta_roll as i32,
        )
    }
}

#[derive(Debug, Clone)]
pub struct IntOrientation {
    pub pitch: i32,
    pub yaw: i32,
    pub roll: i32,
}
impl IntOrientation {
    pub fn new(pitch: i32, yaw: i32, roll: i32) -> Self {
        Self { pitch, yaw, roll }
    }
    pub fn identity() -> Self {
        Self::new(0, 0, 0)
    }
    pub fn to_quat(&self) -> Quat {
        Quat::from_euler(
            glam::EulerRot::XYZ,
            self.pitch as f32,
            self.yaw as f32,
            self.roll as f32,
        )
    }
}

#[derive(Debug, Clone)]
pub struct IntOrientationDelta {
    pub delta_pitch: f32,
    pub delta_yaw: f32,
    pub delta_roll: f32,
}
impl IntOrientationDelta {
    pub fn new(delta_pitch: f32, delta_yaw: f32, delta_roll: f32) -> Self {
        Self {
            delta_pitch,
            delta_yaw,
            delta_roll,
        }
    }
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.delta_pitch, self.delta_yaw, self.delta_roll)
    }
    pub fn between(a: &FloatPosition, b: &FloatPosition) -> Self {
        Self {
            delta_pitch: (b.x - a.x).abs(),
            delta_yaw: (b.y - a.y).abs(),
            delta_roll: (b.z - a.z).abs(),
        }
    }
    pub fn to_int_delta(&self) -> IntPositionDelta {
        IntPositionDelta::new(
            self.delta_pitch as i32,
            self.delta_yaw as i32,
            self.delta_roll as i32,
        )
    }
}

/* -------------------- Bounds -------------------- */

#[derive(Debug, Clone, Copy)]
pub struct RectBounds {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
    pub z_min: i32,
    pub z_max: i32,
}
impl RectBounds {
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

#[derive(Debug, Clone, Copy)]
pub struct CircleBounds {
    pub radius: f32,
}
impl CircleBounds {
    pub fn null() -> Self {
        Self { radius: 0.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Boundaries {
    Rect(RectBounds),
    Circle(CircleBounds),
}
impl Boundaries {
    pub fn kind(&self) -> &'static str {
        match self {
            Boundaries::Rect(_) => "Rectangular",
            Boundaries::Circle(_) => "Circular",
        }
    }
}

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
