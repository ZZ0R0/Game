use crate::volume::Volume::Volume;

struct Position {
    x: f32,
    y: f32,
    z: f32,
}

struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

struct Acceleration {
    x: f32,
    y: f32,
    z: f32,
}

struct Orientation {
    pitch: f32,
    yaw: f32,
    roll: f32,
}

struct PlacedObject {
    position: Position,
    orientation: Orientation,
    volume: Volume,
}

struct PhysicalObject {
    placed: PlacedObject,
    velocity: Velocity,
    acceleration: Acceleration,
    mass: f32,
}