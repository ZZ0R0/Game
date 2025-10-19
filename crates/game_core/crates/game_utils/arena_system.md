# Arena System — Guide court

## Objectif
Stocker des objets indexés par **handles** stables, avec accès O(1), itération rapide, et suppression sans déplacer les autres éléments.

## Concepts
- **Handle**: `{index, generation}`. La génération évite l’erreur ABA après suppression.
- **Arena<T>**: tableau de slots; chaque slot stocke `generation` et un `T` optionnel.
- **Free list**: indices libres réutilisés lors des insertions.

## Opérations
- `insert(T) -> Handle`
- `get(handle) -> Option<&T>` et `get_mut(handle) -> Option<&mut T>`
- `remove(handle) -> Option<T>` qui **incrémente** la génération du slot
- `contains(handle) -> bool`
- `iter()` et `iter_mut()` pour parcours dense

## Usage type
```rust
let mut ships: Arena<Ship> = Arena::with_capacity(10_000);

let h = ships.insert(Ship::new("Scout"));

if let Some(s) = ships.get_mut(h) {
    s.velocity = Vec3::new(0.0, 0.0, 10.0);
}

// Handle devenu invalide après remove
let old = h;
let _ = ships.remove(h);
assert!(!ships.contains(old)); // génération différente
```

## Avantages
- Handles stables pour le réseau et le rendu.
- Supprimer n’invalide que le handle ciblé.
- Itération compacte si l’Arena est densifiée.

## Bonnes pratiques
- Encapsuler le type `Handle` par domaine (`GridHandle`, `BlockHandle`).
- Ne jamais exposer l’index brut.
- Incrémenter la génération sur chaque `remove`.
- Option: ajouter une **version** monotone pour faciliter les deltas.