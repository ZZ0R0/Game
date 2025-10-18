use glam::{Vec3, Quat};

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
        Quat::from_euler(glam::EulerRot::XYZ, self.pitch as f32, self.yaw as f32, self.roll as f32)
    }
}

#[derive(Debug, Clone)]
pub struct Volume {
    pub points: Vec<(f32, f32, f32)>,
    pub position: (f32, f32, f32),
}

impl Volume {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            position: (0.0, 0.0, 0.0),
        }
    }

    pub fn unit_cube() -> Self {
        Self {
            points: vec![
                (-0.5, -0.5, -0.5), (0.5, -0.5, -0.5),
                (0.5, 0.5, -0.5), (-0.5, 0.5, -0.5),
                (-0.5, -0.5, 0.5), (0.5, -0.5, 0.5),
                (0.5, 0.5, 0.5), (-0.5, 0.5, 0.5),
            ],
            position: (0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlacedObject {
    pub position: FloatPosition,
    pub orientation: FloatOrientation,
    pub volume: Volume,
}

impl PlacedObject {
    pub fn new(position: FloatPosition, orientation: FloatOrientation, volume: Volume) -> Self {
        Self { position, orientation, volume }
    }

    pub fn default() -> Self {
        Self {
            position: FloatPosition::zero(),
            orientation: FloatOrientation::identity(),
            volume: Volume::unit_cube(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalObject {
    pub placed: PlacedObject,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub mass: f32,
}

impl PhysicalObject {
    pub fn new(placed: PlacedObject, velocity: Velocity, acceleration: Acceleration, mass: f32) -> Self {
        Self { placed, velocity, acceleration, mass }
    }

    pub fn default() -> Self {
        Self {
            placed: PlacedObject::default(),
            velocity: Velocity::zero(),
            acceleration: Acceleration::zero(),
            mass: 1.0,
        }
    }
}