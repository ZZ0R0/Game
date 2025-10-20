use crate::blocks::Block;
use crate::celestials::Celestial;
use crate::grids::Grid;
use crate::humanoids::Humanoid;
use crate::utils::arenas::HasId;
use crate::utils::ids::EntityId;

#[derive(Clone)]
pub enum Entity {
    Humanoid(Humanoid),
    Celestial(Celestial),
    Grid(Grid),
    Block(Block),
}

impl HasId<EntityId> for Entity {
    #[inline]
    fn id_ref(&self) -> &EntityId {
        match self {
            Entity::Humanoid(h) => &h.id,
            Entity::Celestial(c) => &c.id,
            Entity::Grid(g) => &g.id,
            Entity::Block(b) => &b.id,
        }
    }
    #[inline]
    fn id_mut(&mut self) -> &mut EntityId {
        match self {
            Entity::Humanoid(h) => &mut h.id,
            Entity::Celestial(c) => &mut c.id,
            Entity::Grid(g) => &mut g.id,
            Entity::Block(b) => &mut b.id,
        }
    }
}

impl Entity {
    #[inline]
    pub fn is_physical(&self) -> bool {
        matches!(
            self,
            Entity::Humanoid(_) | Entity::Celestial(_) | Entity::Grid(_)
        )
    }
    #[inline]
    pub fn is_logical(&self) -> bool {
        matches!(
            self,
            Entity::Humanoid(_) | Entity::Grid(_) | Entity::Block(_)
        )
    }
}
