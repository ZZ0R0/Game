# game_objects — Objets du jeu et logique

## Rôle
Définir toutes les structures orientées objet du jeu et leurs comportements : entités, grilles, blocs, joueurs, inventaires, etc. Ce crate implémente la logique fondamentale des objets, conforme aux responsabilités annoncées pour `game_core` dans le README racine.

## Contenu attendu
- **Identifiants d’objets** : `PlayerId`, `EntityId`, `GridId`, `BlockId` (u64 typés) et conversions si nécessaire. Les IDs résident ici car ils sont sémantiquement liés aux objets.
- **Types POO** : `Player`, `Entity`, `Grid`, `Block`, composants et agrégats.
- **Journal de changements** : enregistrement append-only des mutations d’objets par tick, source des deltas. Orienté “delta-driven”, sans dirty flags.
- **Indexation spatiale orientée objets** : vues pour AOI/PlayerWorld (requêtes rayon, cône, visibilité).
- **Knowledge (accès à l’info)** : graphe minimal de faits/arêtes pour déterminer ce qu’un joueur connaît d’un objet à l’instant t.
- **PlayerWorld** : vue par joueur = AOI ∩ Knowledge. Ne contient que les objets connus du joueur, pour que le serveur n’envoie que les deltas pertinents.

## N’y met pas
- Pas de rendu graphique.
- Pas de code réseau, ni sérialisation fil de fer.
- Pas d’outils génériques non liés aux objets (ils vont dans `game_utils`).

## Interfaces clés exposées
- Types d’objets et leurs méthodes de mutation qui écrivent dans le **journal de changements**.
- APis de requêtes pour AOI et PlayerWorld.
- Fonctions utilitaires d’évaluation `can_know(player, entity, field)` basées sur le graphe de knowledge.

## Dossiers
```
crates/game_core/crates/game_objects/
  ├─ src/
  │   ├─ ids.rs             # IDs liés aux objets (u64 typés) et helpers
  │   ├─ entity.rs          # entités mobiles de haut niveau
  │   ├─ grid.rs            # grilles, collections de blocs
  │   ├─ block.rs           # blocs et sous-systèmes (hp, énergie, etc.)
  │   ├─ change_journal.rs  # append-only log des mutations (source des deltas)
  │   ├─ spatial_index.rs   # indexations orientées objets (AOI)
  │   ├─ knowledge.rs       # graphe d’accès à l’information
  │   └─ player_world.rs    # vue par joueur = AOI ∩ Knowledge
  └─ Cargo.toml
```

## Intégrations
- Consomme `game_utils` pour les utilitaires (Arena, threading, maths).
- Alimente `game_protocol` via structures brutes destinées au packaging de deltas dans le serveur.