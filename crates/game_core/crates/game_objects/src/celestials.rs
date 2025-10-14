use crate::objects::PhysicalObject;

#[derive(Debug, Clone)]
pub struct CelestialId(pub u32);

#[derive(Debug, Clone)]
pub enum CelestialType {
    Planet,
    Moon,
    Asteroid,
    Star,
}

#[derive(Debug, Clone)]
pub struct CelestialBody {
    pub id: CelestialId,
    pub name: String,
    pub celestial_type: CelestialType,
    pub physical: PhysicalObject,
    pub radius: f32,
    pub gravity_strength: f32,
    pub atmosphere: bool,
}

impl CelestialBody {
    pub fn new(
        id: u32,
        name: String,
        celestial_type: CelestialType,
        physical: PhysicalObject,
        radius: f32,
        gravity_strength: f32,
        atmosphere: bool,
    ) -> Self {
        Self {
            id: CelestialId(id),
            name,
            celestial_type,
            physical,
            radius,
            gravity_strength,
            atmosphere,
        }
    }

    pub fn earth_like_planet(id: u32) -> Self {
        Self::new(
            id,
            "Earth-like Planet".to_string(),
            CelestialType::Planet,
            PhysicalObject::default(),
            60000.0, // 60km radius
            9.81,    // Earth gravity
            true,
        )
    }

    pub fn small_moon(id: u32) -> Self {
        Self::new(
            id,
            "Small Moon".to_string(),
            CelestialType::Moon,
            PhysicalObject::default(),
            10000.0, // 10km radius
            1.62,    // Moon-like gravity
            false,
        )
    }

    pub fn asteroid(id: u32) -> Self {
        Self::new(
            id,
            "Asteroid".to_string(),
            CelestialType::Asteroid,
            PhysicalObject::default(),
            500.0, // 500m radius
            0.1,   // Very low gravity
            false,
        )
    }
}