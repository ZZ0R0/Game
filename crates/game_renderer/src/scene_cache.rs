use wgpu::BindGroup;
use ahash::AHashMap;
use std::hash::Hash;

/// Identifiant unique pour un objet rendu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId {
    pub id: u32,
    pub version: u64,
}

/// Cache pour les bind groups et ressources GPU
pub struct SceneCache {
    /// Cache des bind groups par ObjectId
    bind_groups: AHashMap<ObjectId, BindGroup>,
    /// Version actuelle de chaque objet
    object_versions: AHashMap<u32, u64>,
}

impl SceneCache {
    pub fn new() -> Self {
        Self {
            bind_groups: AHashMap::new(),
            object_versions: AHashMap::new(),
        }
    }

    /// Vérifie si un objet a été modifié
    pub fn is_dirty(&self, object_id: u32, version: u64) -> bool {
        match self.object_versions.get(&object_id) {
            Some(&cached_version) => cached_version != version,
            None => true, // Nouvel objet
        }
    }

    /// Met en cache un bind group pour un objet
    pub fn cache_bind_group(&mut self, object_id: u32, version: u64, bind_group: BindGroup) {
        let id = ObjectId { id: object_id, version };
        
        // Nettoie l'ancienne version si elle existe
        if let Some(&old_version) = self.object_versions.get(&object_id) {
            let old_id = ObjectId { id: object_id, version: old_version };
            self.bind_groups.remove(&old_id);
        }

        self.bind_groups.insert(id, bind_group);
        self.object_versions.insert(object_id, version);
    }

    /// Récupère un bind group depuis le cache
    pub fn get_bind_group(&self, object_id: u32, version: u64) -> Option<&BindGroup> {
        let id = ObjectId { id: object_id, version };
        self.bind_groups.get(&id)
    }

    /// Nettoie les entrées obsolètes du cache
    pub fn cleanup_old_entries(&mut self, active_objects: &[(u32, u64)]) {
        let active_ids: AHashMap<u32, u64> = active_objects.iter().copied().collect();
        
        // Supprime les objets qui ne sont plus actifs
        self.bind_groups.retain(|object_id, _| {
            active_ids.get(&object_id.id)
                .map_or(false, |&version| version == object_id.version)
        });

        // Met à jour les versions d'objets
        self.object_versions = active_ids;
    }

    /// Statistiques du cache
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            bind_groups_count: self.bind_groups.len(),
            objects_count: self.object_versions.len(),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub bind_groups_count: usize,
    pub objects_count: usize,
}
