use objects_data::*;


struct Atmosphere {
    pub oxygen_level: f32,
    pub pressure: f32,
    pub temperature: f32,
    pub toxicity: f32,
}

struct Celestial {
    pub id: Id,
    pub name: String,
    pub physical : PhysicalObject,
    pub size:u32,
    pub seed:u64,
    pub mass: f32,
}

struct Planet {
    pub celestial: Celestial,
    pub atmosphere: Atmosphere,
}

struct Asteroid {
    pub celestial: Celestial,
}