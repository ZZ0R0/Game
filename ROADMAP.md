# ROADMAP

Cette feuille de route intègre le **Delta existant** et introduit un **Knowledge** qui filtre les champs **sans** journal ni dirty flags.

## 0) Pré‑requis
- Conserver `Arena` dans `game_utils` et **IDs** dans `game_objects`.
- `game_core/src` ne contient que des **ré‑exports**.
- Valider que le Delta hiérarchique actuel couvre bien `added/modified/removed`.

## 1) Clarifier les frontières
- Mettre à jour les README de `game_core`, `game_utils`, `game_objects`, `arena_system`, `threading_system`, `ids_handles`, `DELTA_SYSTEM` pour refléter ce document.

## 2) AOI minimal
- Index spatial simple pour “rayon joueur”. Output: set d’entités **chargées physiquement**.
- Emplacement: `game_objects`.

## 3) Knowledge v1 (filtre de champs)
- Définir les **canaux** par entité: `Physique`, `Logique` (et sous‑canaux par bloc si nécessaire).
- Implémenter le **résolveur de capacités**: vision/LOS pour `Physique`, antennes/ownership/terminal pour `Logique`.
- Output: `PlayerView(player, t) -> {(entité, canal) autorisés}`.
- Emplacement: `game_objects`.

## 4) PlayerView ↔ Delta
- Dans le serveur, avant envoi, **filtrer** le delta du tick par `PlayerView`.
- Si entité nouvellement accessible → envoyer **base initiale** des canaux autorisés.
- Si perte d’accès → émettre **tombstones** ciblées.
- Emplacement: `game_server` (filtrage) + `game_protocol` (formes de messages).

## 5) Threading immédiat
- Paralléliser les boucles suivantes avec le pattern **map → commit**:
  - updates par **grille** et par **bloc**,
  - AOI par shard,
  - construction des deltas **par joueur**.
- Emplacement: `game_utils::threading` + appels depuis `game_server`.

## 6) Protocole
- Messages: `WorldDelta` filtré par connexion, `BaseInitiale`, `Tombstone`, `Ack`.
- Emplacement: `game_protocol`.

## 7) Côté client
- Appliquer `removed` avant `added/modified`.
- Maintenir la **liste locale** des entités/canaux connus.
- En cas de désync, demander **rechargement complet autorisé**.

## 8) Tests & critères
- AOI: exact sur cas limites (distance, entrée/sortie de rayon).
- Knowledge: accès logique correct par antennes/ownership/terminal.
- PlayerView ↔ Delta: pas de fuite d’information; base initiale correcte; tombstones à la perte d’accès.
- Threading: gain visible à haute cardinalité, pas de data race.
- Reconnect tardif: reçoit l’état courant autorisé puis les deltas suivants.

## 9) Itérations futures
- Sous‑canaux plus fins côté logique (inventaire par bloc).
- Optimisations sérialisation (packing binaire).
- Priorités réseau par type de champ.