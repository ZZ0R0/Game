# game_utils — utilitaires transverses

## Rôle
Regrouper les briques réutilisables **non liées** aux objets du jeu.

## Contenu
- **Arena** : stockage par handles stables. Fichier: `crates/game_core/crates/game_utils/src/arena.rs`.
- **Threading par jobs** : pool + file MPMC pour paralléliser les boucles massives. Fichier: `crates/game_core/crates/game_utils/src/threading.rs`.
- **Math/geom & pools** : helpers génériques, buffers réutilisables.

## Hors‑périmètre
- Pas d’IDs métier, pas de logique de gameplay.
- Pas de protocoles réseau.

## Intégrations
- Importé par `game_objects`, `game_server`, `game_client`.
- Aucune dépendance inverse vers `game_objects`.