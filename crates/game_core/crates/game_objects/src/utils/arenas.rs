// arenas.rs — listes d'IDs + arènes + TLS
pub use game_utils::arena::{Arena, HasId};

use crate::entities::Entity;
use crate::utils::ids::EntityId;

use std::cell::RefCell;
use std::sync::{Arc, RwLock};

// --- vues multiples: toutes en EntityId --------------------------------------
#[derive(Default)]
pub struct IdLists {
    pub entity_ids: Vec<EntityId>,
    pub physical_entity_ids: Vec<EntityId>,
    pub logical_entity_ids: Vec<EntityId>,
    pub humanoid_ids: Vec<EntityId>,
    pub celestial_ids: Vec<EntityId>,
    pub block_ids: Vec<EntityId>,
}

// Compteur simple d'EntityId (par monde)
#[derive(Default)]
pub struct IdCounters {
    pub entity: u32,
}

#[inline]
fn push_unique<T: PartialEq>(v: &mut Vec<T>, x: T) {
    if !v.contains(&x) {
        v.push(x);
    }
}

#[macro_export]
macro_rules! define_arenas {
    (
        $(
            $field:ident : $ty:ty, $id:ty,
            insert_fn = $insert_fn:ident,
            get_fn    = $get_fn:ident,
            get_mut_fn= $get_mut_fn:ident,
            remove_fn = $remove_fn:ident
        );+ $(;)?
    ) => {
        pub struct Arenas {
            $( pub $field: $crate::utils::arenas::Arena<$ty, $id>, )+
            pub lists: $crate::utils::arenas::IdLists,
            pub counters: $crate::utils::arenas::IdCounters,
        }

        impl Arenas {
            #[inline]
            pub fn new() -> Self {
                Self {
                    $( $field: $crate::utils::arenas::Arena::new(), )+
                    lists: Default::default(),
                    counters: Default::default(),
                }
            }

            $(
                #[inline] pub fn $insert_fn(&mut self, v: $ty) -> $id { self.$field.insert(v) }
                #[inline] pub fn $get_fn(&self, id: $id) -> Option<&$ty> { self.$field.get(id) }
                #[inline] pub fn $get_mut_fn(&mut self, id: $id) -> Option<&mut $ty> { self.$field.get_mut(id) }
                #[inline] pub fn $remove_fn(&mut self, id: $id) -> Option<$ty> { self.$field.remove(id) }
            )+

            // --- alloc d'EntityId (si tu veux auto-générer avant insertion) --
            #[inline] pub fn alloc_entity_id(&mut self) -> EntityId {
                let v = self.counters.entity; self.counters.entity += 1; EntityId(v)
            }

            // --- helpers de tagging (poussent l'EntityId dans les listes) -----
            #[inline] pub fn tag_entity(&mut self, id: EntityId)              { $crate::utils::arenas::push_unique(&mut self.lists.entity_ids, id); }
            #[inline] pub fn tag_physical(&mut self, id: EntityId)            { $crate::utils::arenas::push_unique(&mut self.lists.physical_entity_ids, id); }
            #[inline] pub fn tag_logical(&mut self, id: EntityId)             { $crate::utils::arenas::push_unique(&mut self.lists.logical_entity_ids, id); }
            #[inline] pub fn tag_humanoid(&mut self, id: EntityId)            { $crate::utils::arenas::push_unique(&mut self.lists.humanoid_ids, id); }
            #[inline] pub fn tag_celestial(&mut self, id: EntityId)           { $crate::utils::arenas::push_unique(&mut self.lists.celestial_ids, id); }
            #[inline] pub fn tag_block(&mut self, id: EntityId)               { $crate::utils::arenas::push_unique(&mut self.lists.block_ids, id); }
        }
    }
}

// ==== arènes concrètes ========================================================
define_arenas! {
    entities: Entity, EntityId,
        insert_fn = insert_entity,
        get_fn    = entity,
        get_mut_fn= entity_mut,
        remove_fn = remove_entity;
}

// ==== TLS handle (inchangé) ===================================================
pub type SharedArenas = Arc<RwLock<Arenas>>;

thread_local! { static ARENAS_STACK: RefCell<Vec<SharedArenas>> = RefCell::new(Vec::new()); }

pub struct ArenasScope;
impl Drop for ArenasScope {
    fn drop(&mut self) {
        ARENAS_STACK.with(|s| {
            let _ = s.borrow_mut().pop();
        });
    }
}

#[inline]
pub fn enter_scope(handle: SharedArenas) -> ArenasScope {
    ARENAS_STACK.with(|s| s.borrow_mut().push(handle));
    ArenasScope
}

#[inline]
pub fn with_current<R>(f: impl FnOnce(&RwLock<Arenas>) -> R) -> R {
    ARENAS_STACK.with(|s| {
        let h = s.borrow().last().cloned().expect("No current Arenas scope");
        f(h.as_ref())
    })
}
#[inline]
pub fn with_current_write<R>(f: impl FnOnce(&mut Arenas) -> R) -> R {
    ARENAS_STACK.with(|s| {
        let h = s.borrow().last().cloned().expect("No current Arenas scope");
        let mut g = h.write().unwrap();
        f(&mut *g)
    })
}
#[inline]
pub fn with_current_read<R>(f: impl FnOnce(&Arenas) -> R) -> R {
    ARENAS_STACK.with(|s| {
        let h = s.borrow().last().cloned().expect("No current Arenas scope");
        let g = h.read().unwrap();
        f(&*g)
    })
}
#[inline]
pub fn current_handle() -> Option<SharedArenas> {
    ARENAS_STACK.with(|s| s.borrow().last().cloned())
}
