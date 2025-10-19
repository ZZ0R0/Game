# game_utils — Utilitaires transverses

## Rôle
Fournir des briques réutilisables **indépendantes** des objets du jeu. Ces utilitaires servent à plusieurs crates sans dépendre des types métier.

## Contenu attendu
- **Arena** : gestion de stockage par handles stables. Emplacement existant : `crates/game_core/crates/game_utils/src/arena.rs`.
- **Threading par jobs** : file MPMC et pool de workers pour paralléliser les boucles “pour chaque grille/entité/bloc” avec pattern map→commit.
- **Maths et géométrie** : types et helpers mathématiques génériques si nécessaires.
- **Collections et pools** : `SmallVec`, slabs, pools d’allocations temporaires.
- **Chronométrage et profiling** : mesures par job, rapports agrégés.

## N’y met pas
- Pas d’IDs ni de types liés sémantiquement aux objets.
- Pas de logique de gameplay.
- Pas de protocoles réseau.

## Interfaces clés exposées
- `arena` : types de handles et API d’insertion/suppression/itération.
- `threading` : API simple pour `for_each` parallélisé et `commit` final.
- Helpers utilitaires pur génériques.

## Dossiers
```
crates/game_core/crates/game_utils/
  ├─ src/
  │   ├─ arena.rs         # handles stables et stockage dense
  │   ├─ threading.rs     # exécuteur de jobs, MPMC, profiling
  │   ├─ math.rs          # vecteurs, AABB, utilitaires
  │   └─ pools.rs         # buffers/pools réutilisables
  └─ Cargo.toml
```

## Intégrations
- Importé par `game_objects`, `game_server`, `game_client` selon besoin.
- Aucune dépendance inverse vers `game_objects`.