use glam::{Quat, Vec3};

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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            grid_position.x + self.x as f32 * 2.5, // 2.5m per block
            grid_position.y + self.y as f32 * 2.5,
            grid_position.z + self.z as f32 * 2.5,
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntDistance {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IntDistance {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn between(a: &IntPosition, b: &IntPosition) -> Self {
        Self {
            x: (b.x - a.x).abs(),
            y: (b.y - a.y).abs(),
            z: (b.z - a.z).abs(),
        }
    }
}

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

