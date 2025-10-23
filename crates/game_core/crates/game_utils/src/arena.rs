// arena.rs — utilitaires d’arène génériques (IDs = u32, sans logique World)
use slotmap::{DefaultKey, SlotMap};
use std::collections::HashMap;
use std::hash::Hash;

/// Clé interne de l’arène
type SmKey = DefaultKey;

/// Un type stocké doit exposer un identifiant (ex: u32, u64...)
pub trait HasId<Id> {
    fn id_ref(&self) -> &Id;
    fn id_mut(&mut self) -> &mut Id {
        unreachable!("HasId::id_mut non implémenté pour ce type; requis par Arena::set")
    }
}

#[derive(Debug)]
pub struct Arena<T, Id>
where
    Id: Eq + Hash + Copy,
    T: HasId<Id>,
{
    slab: SlotMap<SmKey, T>,
    index: HashMap<Id, SmKey>,
}

impl<T, Id> Arena<T, Id>
where
    Id: Eq + Hash + Copy,
    T: HasId<Id>,
{
    pub fn new() -> Self {
        Self {
            slab: SlotMap::with_key(),
            index: HashMap::new(),
        }
    }

    /// Insère `value` déjà porteur d’un Id unique. Retourne cet Id.
    pub fn insert(&mut self, value: T) -> Id {
        let id = *value.id_ref();
        let key = self.slab.insert(value);
        let old = self.index.insert(id, key);
        debug_assert!(old.is_none(), "Id déjà présent dans l'arène");
        id
    }

    /// Place `value` sous l'Id `id`. Remplace et retourne l'ancien si présent.
    pub fn set(&mut self, id: Id, mut value: T) -> Option<T> {
        // Forcer la cohérence Id porté par la valeur
        if *value.id_ref() != id {
            *value.id_mut() = id;
        }

        if let Some(&k) = self.index.get(&id) {
            if let Some(slot) = self.slab.get_mut(k) {
                // Remplacement en place
                let old = std::mem::replace(slot, value);
                return Some(old);
            }
            // Clé orpheline: retomber sur une insertion propre
        }

        let k = self.slab.insert(value);
        let prev = self.index.insert(id, k);
        debug_assert!(prev.is_none(), "index incohérent: Id présent sans entrée valide dans slab");
        None
    }

    /// Accès immuable via Id
    pub fn get(&self, id: Id) -> Option<&T> {
        self.index.get(&id).and_then(|&k| self.slab.get(k))
    }

    /// Accès mutable via Id
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
            Some(k) => self.slab.remove(k),
            None => None,
        }
    }

    /// Itération immuable sur (Id, &T)
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.slab.iter().map(|(_k, v)| (*v.id_ref(), v))
    }

    /// Itération mutable sur (Id, &mut T)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)> {
        self.slab.iter_mut().map(|(_k, v)| (*v.id_ref(), v))
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

impl<T, Id> Clone for Arena<T, Id>
where
    Id: Eq + Hash + Copy,
    T: HasId<Id> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            slab: self.slab.clone(),
            index: self.index.clone(),
        }
    }
}
