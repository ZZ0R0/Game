use glam::{Quat, Vec3};




/* -------------------- Positions -------------------- */

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn undefined() -> Self {
        Self::new(f32::NAN, f32::NAN, f32::NAN)
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn undefined() -> Self {
        Self::new(f32::NAN, f32::NAN, f32::NAN)
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn undefined() -> Self {
        Self::new(f32::NAN, f32::NAN, f32::NAN)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn undefined() -> Self {
        Self::new(f32::NAN, f32::NAN, f32::NAN)
    }
}

/* -------------------- Orientation -------------------- */

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn undefined() -> Self {
        Self::new(f32::NAN, f32::NAN, f32::NAN)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/* -------------------- Placement -------------------- */

#[derive(Debug, Clone)]
pub struct PlacedObject {
    pub position: FloatPosition,
    pub orientation: FloatOrientation,
}
impl PlacedObject {
    pub fn new(position: FloatPosition, orientation: FloatOrientation) -> Self {
        Self {
            position,
            orientation,
        }
    }
    pub fn default() -> Self {
        Self {
            position: FloatPosition::zero(),
            orientation: FloatOrientation::identity(),
        }
    }
    pub fn undefined() -> Self {
        Self {
            position: FloatPosition::undefined(),
            orientation: FloatOrientation::undefined(),
        }
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
    pub fn undefined() -> Self {
        Self {
            x_min: i32::MIN,
            x_max: i32::MAX,
            y_min: i32::MIN,
            y_max: i32::MAX,
            z_min: i32::MIN,
            z_max: i32::MAX,
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
    pub fn undefined() -> Self {
        Self { radius: f32::NAN }
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
    pub placed_object: PlacedObject,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub mass: f32,
    pub boundaries: Boundaries,
}
impl PhysicalObject {
    pub fn new(
        placed_object: PlacedObject,
        velocity: Velocity,
        acceleration: Acceleration,
        mass: f32,
        boundaries: Boundaries,
    ) -> Self {
        Self {
            placed_object,
            velocity,
            acceleration,
            mass,
            boundaries,
        }
    }
    pub fn undefined() -> Self {
        Self {
            placed_object: PlacedObject::undefined(),
            velocity: Velocity::undefined(),
            acceleration: Acceleration::undefined(),
            mass: f32::NAN,
            boundaries: Boundaries::Rect(RectBounds::undefined()),
        }
    }
}

/* -------------------- Spatial updater contract -------------------- */

/// Mapping layer adaptor. Implemented by the spatial index.
/// Keeps objects.rs independent of the concrete mapping type.
pub trait SpatialUpdater {
    fn update_on_move(&mut self, id: u32, old_pos: IntPosition, new_pos: IntPosition);
}

/* -------------------- Physical delta (shared) -------------------- */

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PhysicalObjectDelta {
    pub position: Option<FloatPosition>,
    pub orientation: Option<FloatOrientation>,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
    pub mass: Option<f32>,
    pub timestamp: u64,
    pub sequence: u64,
}
impl PhysicalObjectDelta {
    #[inline]
    pub fn empty(ts: u64, seq: u64) -> Self {
        Self {
            timestamp: ts,
            sequence: seq,
            ..Default::default()
        }
    }
    #[inline]
    pub fn with_position(mut self, p: FloatPosition) -> Self {
        self.position = Some(p);
        self
    }
    #[inline]
    pub fn with_orientation(mut self, o: FloatOrientation) -> Self {
        self.orientation = Some(o);
        self
    }
    #[inline]
    pub fn with_velocity(mut self, v: Velocity) -> Self {
        self.velocity = Some(v);
        self
    }
    #[inline]
    pub fn with_acceleration(mut self, a: Acceleration) -> Self {
        self.acceleration = Some(a);
        self
    }
    #[inline]
    pub fn with_mass(mut self, m: f32) -> Self {
        self.mass = Some(m);
        self
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.position.is_none()
            && self.orientation.is_none()
            && self.velocity.is_none()
            && self.acceleration.is_none()
            && self.mass.is_none()
    }

    pub fn merge(mut deltas: Vec<PhysicalObjectDelta>) -> Option<PhysicalObjectDelta> {
        if deltas.is_empty() {
            return None;
        }
        deltas.sort_by_key(|d| d.sequence);
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
            m.timestamp = d.timestamp;
            m.sequence = d.sequence;
        }
        Some(m)
    }

    /// Apply delta and notify mapping in one call. Updates only if cell/chunk/region may change.
    pub fn apply_and_update_mapping<U: SpatialUpdater>(
        &self,
        entity_id: u32,
        physical_object: &mut PhysicalObject,
        mapping: &mut U,
    ) {
        // Int position before
        let old_int = physical_object.placed_object.position.to_int_position();
        // Apply mutation
        if !self.is_empty() {
            if let Some(p) = self.position.clone() {
                physical_object.placed_object.position = p;
            }
            if let Some(o) = self.orientation.clone() {
                physical_object.placed_object.orientation = o;
            }
            if let Some(v) = self.velocity.clone() {
                physical_object.velocity = v;
            }
            if let Some(a) = self.acceleration.clone() {
                physical_object.acceleration = a;
            }
            if let Some(m) = self.mass {
                physical_object.mass = m;
            }
        }
        // Int position after
        let new_int = physical_object.placed_object.position.to_int_position();
        if old_int != new_int {
            mapping.update_on_move(entity_id, old_int, new_int);
        }
    }

    /// Keep the old simple apply when you don't care about mapping.
    pub fn apply_to(&self, physical_object: &mut PhysicalObject) {
        if let Some(p) = self.position.clone() {
            physical_object.placed_object.position = p;
        }
        if let Some(o) = self.orientation.clone() {
            physical_object.placed_object.orientation = o;
        }
        if let Some(v) = self.velocity.clone() {
            physical_object.velocity = v;
        }
        if let Some(a) = self.acceleration.clone() {
            physical_object.acceleration = a;
        }
        if let Some(m) = self.mass {
            physical_object.mass = m;
        }
    }
}
