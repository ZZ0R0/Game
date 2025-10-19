# IDs & Handles — conventions

## IDs (définis dans `game_objects`)
- `WorldId`, `GridId`, `EntityId`, `PlayerId`, `BlockId` : entiers typés.
- Stables à travers les deltas et le temps.
- Portés par les objets et utilisés au niveau protocole.

## Handles (dans `game_utils::arena`)
- Références locales `{index, generation}` pour accès mémoire rapide.
- Convertibles depuis/vers `Id` via des tables de correspondance au besoin.
- Jamais exposés sur le fil réseau.

## Règles
- Ne pas mélanger `Id` et `Handle` dans les APIs.
- Les conversions vivent du côté des systèmes qui en ont besoin (rendu, simulation).