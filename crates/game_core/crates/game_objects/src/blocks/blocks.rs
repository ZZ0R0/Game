// blocks.rs — Block + remove() interne

use crate::entities::Entity;
use crate::physics::{IntOrientation, IntPosition};
use crate::utils::arenas::Arenas;
use crate::utils::arenas::{with_current_write, Arena, HasId};
use crate::utils::ids::{BlockDefId, EntityId, FactionId};
use std::sync::Arc;

// -------- Types de base

#[derive(Debug, Clone)]
pub struct BlockDef {
    pub id: BlockDefId,
    pub name: String,
    pub footprint: (i32, i32, i32),
    pub mass: f32,
    pub integrity: f32,
    pub block_type: String,
    pub available_components: Vec<String>,
}
impl BlockDef {
    pub fn new(
        id: BlockDefId,
        name: impl Into<String>,
        fp: (i32, i32, i32),
        mass: f32,
        integ: f32,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            footprint: fp,
            mass,
            integrity: integ,
            block_type: kind.into(),
            available_components: Vec::new(),
        }
    }
}

// -------- Block instance + deltas

#[derive(Debug, Clone)]
pub struct Block {
    pub id: EntityId,
    pub grid_id: EntityId,
    pub def: Arc<BlockDef>,
    pub current_integrity: f32,
    pub current_mass: f32,
    pub position: IntPosition,
    pub orientation: IntOrientation,
    pub faction_id: FactionId,
}

// Pour Arena<Block, EntityId>
impl HasId<EntityId> for Block {
    #[inline]
    fn id_ref(&self) -> &EntityId {
        &self.id
    }
    #[inline]
    fn id_mut(&mut self) -> &mut EntityId {
        &mut self.id
    }
}

impl Block {
    pub fn new(
        id: EntityId,
        grid_id: EntityId,
        def: Arc<BlockDef>,
        position: IntPosition,
        orientation: IntOrientation,
        integrity: f32,
        faction_id: FactionId,
    ) -> Self {
        Self {
            id,
            grid_id,
            current_mass: def.mass,
            def,
            position,
            orientation,
            current_integrity: integrity,
            faction_id,
        }
    }

    /// Supprime ce block et le détache de sa grille
    #[inline]
    pub fn remove(id: EntityId) -> bool {
        with_current_write(|a| Self::remove_with_ctx(a, id))
    }

    /// Version interne réutilisable depuis d'autres objets (évite re-lock)
    pub(crate) fn remove_with_ctx(a: &mut Arenas, block_id: EntityId) -> bool {
        let Some(h) = a.get_entity(block_id) else {
            return false;
        };

        // lire la grid cible
        let grid_id_opt = {
            let g = h.read().unwrap();
            if let Entity::Block(ref b) = *g {
                Some(b.grid_id)
            } else {
                None
            }
        };

        // retirer l'id du block de la grid si existante
        if let Some(grid_id) = grid_id_opt {
            if let Some(gh) = a.get_entity(grid_id) {
                let mut gw = gh.write().unwrap();
                if let Entity::Grid(ref mut grid) = *gw {
                    grid.block_ids.retain(|&x| x != block_id);
                }
            }
        }

        // suppression shallow de l'arène + listes globales
        let existed = a.remove_entity(block_id).is_some();
        if existed {
            a.lists.entity_ids.retain(|&x| x != block_id);
            a.lists.physical_entity_ids.retain(|&x| x != block_id);
            a.lists.logical_entity_ids.retain(|&x| x != block_id);
            a.lists.block_ids.retain(|&x| x != block_id);
        }
        existed
    }
}
