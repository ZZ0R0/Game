struct RelPosition {
    x: i32,
    y: i32,
    z: i32,
}

struct RelOrientation {
    pitch: i32,
    yaw: i32,
    roll: i32,
}

struct RelObject {
    position: RelPosition,
    orientation: RelOrientation,
    mass: f32,
}

struct Block {
    pub name: String,
    pub integrity: f32,
    pub rel_object: RelObject,
}