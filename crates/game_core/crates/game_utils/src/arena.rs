// arena.rs — utilitaires d’arène génériques (IDs = u32, sans logique World)
// Stockage clé=Id avec accès O(1), et "fine-grained locking" via handles Arc<RwLock<T>>.
// Pour compatibilité avec les arènes locales (ex: LogicalObject.components):
//  - insert(&mut self, value: T) -> Id   (requiert T: HasId<Id>)
//  - get(&self, id: Id) -> Option<&T>

use slotmap::{DefaultKey, SlotMap};
use std::collections::HashMap;
use std::hash::Hash;

pub type ArcRw<T> = std::sync::Arc<std::sync::RwLock<T>>;

/// Clé interne de l’arène
type SmKey = DefaultKey;

/// Optionnel: types porteurs d’un Id local
pub trait HasId<Id> {
    fn id_ref(&self) -> &Id;
    fn id_mut(&mut self) -> &mut Id {
        unreachable!("HasId::id_mut non implémenté")
    }
}

#[derive(Debug)]
pub struct Arena<T, Id>
where
    Id: Eq + Hash + Copy,
{
    slab: SlotMap<SmKey, T>,
    // index: Id -> SlotKey
    index: HashMap<Id, SmKey>,
    // rindex: SlotKey -> Id
    rindex: HashMap<SmKey, Id>,
}

impl<T, Id> Arena<T, Id>
where
    Id: Eq + Hash + Copy,
{
    pub fn new() -> Self {
        Self {
            slab: SlotMap::with_key(),
            index: HashMap::new(),
            rindex: HashMap::new(),
        }
    }

    /// Place `value` sous l'Id `id`. Remplace et retourne l'ancien si présent.
    pub fn set(&mut self, id: Id, value: T) -> Option<T> {
        if let Some(&k) = self.index.get(&id) {
            if let Some(slot) = self.slab.get_mut(k) {
                let old = std::mem::replace(slot, value);
                return Some(old);
            }
        }
        let k = self.slab.insert(value);
        let prev = self.index.insert(id, k);
        let prev_r = self.rindex.insert(k, id);
        debug_assert!(
            prev.is_none() && prev_r.is_none(),
            "index incohérent lors d'un set()"
        );
        None
    }

    /// Accès immuable via Id
    pub fn get_ref(&self, id: Id) -> Option<&T> {
        self.index.get(&id).and_then(|&k| self.slab.get(k))
    }

    /// Accès immuable en clonant la valeur (utile si T=Arc<..>)
    pub fn get_cloned(&self, id: Id) -> Option<T>
    where
        T: Clone,
    {
        self.get_ref(id).cloned()
    }

    /// Accès mutable via Id (éviter si T est déjà sous RwLock)
    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        if let Some(&k) = self.index.get(&id) {
            self.slab.get_mut(k)
        } else {
            None
        }
    }

    /// Retire et retourne l’élément par Id
    pub fn remove(&mut self, id: Id) -> Option<T> {
        match self.index.remove(&id) {
            Some(k) => {
                self.rindex.remove(&k);
                self.slab.remove(k)
            }
            None => None,
        }
    }

    /// Itération immuable sur (Id, &T)
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.slab
            .iter()
            .map(move |(k, v)| (*self.rindex.get(&k).expect("rindex absent"), v))
    }

    /// Itération mutable sur (Id, &mut T)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)> {
        let rindex = &self.rindex;
        self.slab
            .iter_mut()
            .map(move |(k, v)| (*rindex.get(&k).expect("rindex absent"), v))
    }

    pub fn len(&self) -> usize {
        self.slab.len()
    }
    pub fn is_empty(&self) -> bool {
        self.slab.is_empty()
    }

    pub fn clone_inner_slab(&self) -> SlotMap<SmKey, T>
    where
        T: Clone,
    {
        self.slab.clone()
    }
}

/// Compat: insert() et get() pour les arènes locales qui portent l’Id dans T.
impl<T, Id> Arena<T, Id>
where
    Id: Eq + Hash + Copy,
    T: HasId<Id>,
{
    /// Insère `value` déjà porteur de son Id. Panique si l'Id existe déjà.
    pub fn insert(&mut self, value: T) -> Id {
        let id = *value.id_ref();
        if self.index.contains_key(&id) {
            panic!("insert(): Id déjà présent dans l'arène");
        }
        let k = self.slab.insert(value);
        let prev = self.index.insert(id, k);
        let prev_r = self.rindex.insert(k, id);
        debug_assert!(prev.is_none() && prev_r.is_none());
        id
    }

    /// Alias compat pour l’ancien API
    #[inline]
    pub fn get(&self, id: Id) -> Option<&T> {
        self.get_ref(id)
    }
}

impl<T, Id> Clone for Arena<T, Id>
where
    Id: Eq + Hash + Copy,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            slab: self.slab.clone(),
            index: self.index.clone(),
            rindex: self.rindex.clone(),
        }
    }
}
