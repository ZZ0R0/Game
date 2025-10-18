// entities.rs — thin wrapper, no mapping calls; physics lives in objects.rs
use crate::celestials::Celestial;
use crate::grids::Grid;
use crate::humanoids::Humanoid;
use crate::physics::{Acceleration, FloatOrientation, FloatPosition, PhysicalObject, Velocity};
use crate::utils::arena::HasId;

#[derive(Clone)]
pub enum Entity {
    Humanoid(Humanoid),
    Grid(Grid),
    Celestial(Celestial),
}

impl HasId<u32> for Entity {
    #[inline]
    fn id_ref(&self) -> &u32 {
        match self {
            Entity::Humanoid(h) => &h.id.0,
            Entity::Grid(g) => &g.id.0,
            Entity::Celestial(c) => &c.id.0,
        }
    }
}

impl Entity {
    #[inline]
    pub fn physical(&self) -> &PhysicalObject {
        match self {
            Entity::Humanoid(h) => &h.physical_object,
            Entity::Grid(g) => &g.physical_object,
            Entity::Celestial(c) => &c.physical_object,
        }
    }
    #[inline]
    pub fn physical_mut(&mut self) -> &mut PhysicalObject {
        match self {
            Entity::Humanoid(h) => &mut h.physical_object,
            Entity::Grid(g) => &mut g.physical_object,
            Entity::Celestial(c) => &mut c.physical_object,
        }
    }

    // Convenience accessors
    #[inline]
    pub fn get_position(&self) -> &FloatPosition {
        &self.physical().placed_object.position
    }
    #[inline]
    pub fn get_orientation(&self) -> &FloatOrientation {
        &self.physical().placed_object.orientation
    }
    #[inline]
    pub fn get_velocity(&self) -> &Velocity {
        &self.physical().velocity
    }
    #[inline]
    pub fn get_acceleration(&self) -> &Acceleration {
        &self.physical().acceleration
    }
    #[inline]
    pub fn set_position(&mut self, p: FloatPosition) {
        self.physical_mut().placed_object.position = p;
    }
    #[inline]
    pub fn set_orientation(&mut self, o: FloatOrientation) {
        self.physical_mut().placed_object.orientation = o;
    }
    #[inline]
    pub fn set_velocity(&mut self, v: Velocity) {
        self.physical_mut().velocity = v;
    }
    #[inline]
    pub fn set_acceleration(&mut self, a: Acceleration) {
        self.physical_mut().acceleration = a;
    }

    #[inline]
    pub fn player_controlled(&self) -> bool {
        match self {
            Entity::Celestial(_) => false,
            _ => self.player_controlled(),
        }
    }
}
