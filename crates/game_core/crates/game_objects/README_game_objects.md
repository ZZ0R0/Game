# game_objects — objets et logique

## Rôle
Définir les entités du jeu (grids, blocks, entities, players) et **toute la logique associée**.

## Contenu
- **IDs** des objets (u64 typés) et helpers.
- **Système de Delta** hiérarchique existant (added/modified/removed).
- **AOI** (requêtes spatiales) utilisées pour le chargement physique.
- **Knowledge**: **filtre de champs** par joueur (voir ci‑dessous).

## Knowledge: deux canaux par entité
- **Physique**: pose/forme/état visuel. Accès typiquement via vision/AOI.
- **Logique**: inventaires, terminaux, propriété, stats internes. Accès via antennes, ownership, session terminal avec TTL.
Le knowledge **ne modifie pas** l’AOI; il **filtre** les données envoyées.

## PlayerView
Vue par joueur listant, à l’instant T, les paires `(entité, canal)` autorisées. Sert au serveur pour **filtrer** le Delta avant envoi.

## Hors‑périmètre
- Pas de rendu, pas de protocole réseau ici.