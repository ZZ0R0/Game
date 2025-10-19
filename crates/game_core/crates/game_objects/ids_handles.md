# Identifiants et Handles — Guide court

## Objectif
Uniformiser l’adressage des entités dans le moteur et sur le réseau.

## Types
- `WorldId`, `GridId`, `BlockId`: entiers compacts typés.
- `Handle<T>`: `{index, generation}` pour accès local en mémoire.
- `Uuid` facultatif pour ressources persistantes hors‑mémoire.

## Règles
- Ids stables à travers les deltas et les snapshots.
- Interdiction de réutiliser un `Handle` sans génération différente.
- Conversion explicite entre `Id` et `Handle` via tables de correspondance.

## Exemples
```rust
let gid: GridId = grid.id();
let h: GridHandle = world.lookup_handle(gid).unwrap();
```