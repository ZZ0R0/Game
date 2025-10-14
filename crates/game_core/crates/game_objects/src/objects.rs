use glam::{Vec3, Quat};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
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
pub struct Orientation {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl Orientation {
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
    pub position: Position,
    pub orientation: Orientation,
    pub volume: Volume,
}

impl PlacedObject {
    pub fn new(position: Position, orientation: Orientation, volume: Volume) -> Self {
        Self { position, orientation, volume }
    }

    pub fn default() -> Self {
        Self {
            position: Position::zero(),
            orientation: Orientation::identity(),
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