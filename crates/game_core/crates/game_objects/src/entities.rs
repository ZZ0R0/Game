use crate::celestials::Celestial;
use crate::grids::Grid;
use crate::players::Player;

use crate::objects::{Acceleration, FloatOrientation, FloatPosition, Velocity};




enum Entity {
    Player(Player),
    Grid(Grid),
    Celestial(Celestial),
}

impl Entity {
    pub fn get_position(&self) -> &FloatPosition {
        match self {
            Entity::Player(p) => p.get_position(),
            Entity::Grid(g) => g.get_position(),
            Entity::Celestial(c) => c.get_position(),
        }
    }

    pub fn get_orientation(&self) -> &FloatOrientation {
        match self {
            Entity::Player(p) => p.get_orientation(),
            Entity::Grid(g) => g.get_orientation(),
            Entity::Celestial(c) => c.get_orientation(),
        }
    }

    pub fn get_velocity(&self) -> &Velocity {
        match self {
            Entity::Player(p) => p.get_velocity(),
            Entity::Grid(g) => g.get_velocity(),
            Entity::Celestial(c) => c.get_velocity(),
        }
    }

    pub fn get_acceleration(&self) -> &Acceleration {
        match self {
            Entity::Player(p) => p.get_acceleration(),
            Entity::Grid(g) => g.get_acceleration(),
            Entity::Celestial(c) => c.get_acceleration(),
        }
    }

    pub fn get_mass(&self) -> f32 {
        match self {
            Entity::Player(p) => p.physical_object.mass,
            Entity::Grid(g) => g.physical_object.mass,
            Entity::Celestial(c) => c.physical_object.mass,
        }
    }
}