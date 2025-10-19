# game_core — rôle et frontières

## Rôle
Point d’entrée partagé. Ici, uniquement l’API publique qui **ré-exporte** les sous‑crates internes. Pas de logique métier dans `src/`.

## Contenu
- `src/lib.rs` : ré‑exports des modules utiles depuis `crates/`.
- `crates/` :
  - `game_objects` : objets du jeu et logique associée.
  - `game_utils`   : utilitaires génériques, indépendants des objets.

## Hors‑périmètre
- Pas de rendu, pas de réseau.
- Pas de code client/serveur spécifique.
- Pas de duplication d’IDs, d’arena ou de threading ici.

## Convention d’exposition
- Ré‑exporter **sans renommer** les types stables.
- Conserver des chemins d’accès stables pour le reste du workspace.