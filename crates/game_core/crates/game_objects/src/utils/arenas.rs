// arenas.rs — exposition haut-niveau (pas de remove_* récursifs ici)

pub use game_utils::arena::{ArcRw, Arena, HasId};

use crate::entities::Entity;
use crate::utils::ids::EntityId;

use std::cell::RefCell;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, RwLock,
};

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

// Compteur d'EntityId (atomic)
#[derive(Default)]
pub struct IdCounters {
    pub entity: AtomicU32,
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
            $field:ident : $ty:ty, $id:path,
            alloc_id_fn = $alloc_id_fn:ident,
            set_fn = $set_fn:ident,
            insert_fn = $insert_fn:ident,
            get_fn    = $get_fn:ident,
            get_mut_fn= $get_mut_fn:ident,
            remove_fn = $remove_fn:ident,
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
                #[inline]
                pub fn $alloc_id_fn(&self) -> $id {
                    let v = self.counters.entity.fetch_add(1, Ordering::Relaxed);
                    $id(v)
                }

                #[inline]
                pub fn $set_fn(&mut self, k: $id, v: $ty) -> Option<$ty> {
                    self.$field.set(k, v)
                }

                #[inline]
                pub fn $insert_fn(&mut self, v: $ty) -> $id {
                    let id = self.$alloc_id_fn();
                    let _ = self.$set_fn(id, v);
                    id
                }

                #[inline]
                pub fn $get_fn(&self, id: $id) -> Option<$ty>
                where
                    $ty: Clone,
                {
                    self.$field.get_cloned(id)
                }

                #[inline]
                pub fn $get_mut_fn(&self, id: $id) -> Option<$ty>
                where
                    $ty: Clone,
                {
                    self.$field.get_cloned(id)
                }

                #[inline]
                pub fn $remove_fn(&mut self, id: $id) -> Option<$ty> {
                    self.$field.remove(id)
                }
            )+

            // --- helpers de tagging ---
            #[inline] pub fn tag_entity(&mut self, id: EntityId)    { $crate::utils::arenas::push_unique(&mut self.lists.entity_ids, id); }
            #[inline] pub fn tag_physical(&mut self, id: EntityId)  { $crate::utils::arenas::push_unique(&mut self.lists.physical_entity_ids, id); }
            #[inline] pub fn tag_logical(&mut self, id: EntityId)   { $crate::utils::arenas::push_unique(&mut self.lists.logical_entity_ids, id); }
            #[inline] pub fn tag_humanoid(&mut self, id: EntityId)  { $crate::utils::arenas::push_unique(&mut self.lists.humanoid_ids, id); }
            #[inline] pub fn tag_celestial(&mut self, id: EntityId) { $crate::utils::arenas::push_unique(&mut self.lists.celestial_ids, id); }
            #[inline] pub fn tag_block(&mut self, id: EntityId)     { $crate::utils::arenas::push_unique(&mut self.lists.block_ids, id); }
        }
    }
}

// ==== arènes concrètes ========================================================
define_arenas! {
    entities: Arc<std::sync::RwLock<Entity>>, EntityId,
        alloc_id_fn= alloc_entity_id,
        set_fn = set_entity,
        insert_fn = insert_entity,
        get_fn    = get_entity,
        get_mut_fn= get_entity_mut,
        remove_fn = remove_entity,
}

// ==== TLS handle ==============================================================
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
