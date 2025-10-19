# game_core — Rôle et frontières

## Rôle
Point d’entrée partagé. Expose l’API publique minimale des sous-crates internes. Aucun code métier ici, seulement `lib.rs` qui ré-exporte.

Le repo définit que `game_core` contient les structures POO de base et la logique fondamentale du jeu, sans graphisme ni réseau. Les détails d’implémentation vivent dans des sous-crates dédiées. Cela suit les responsabilités décrites au README racine du projet. 

## Contenu attendu
- `src/lib.rs` : ré-exports stables vers les sous-crates.
- `crates/` : sous-crates internes spécialisées, par exemple :
  - `game_objects` : types d’objets du jeu et logique associée.
  - `game_utils`   : utilitaires transverses indépendants des objets.

## N’y met pas
- Pas de rendu graphique.
- Pas de code réseau.
- Pas de logique spécifique client ou serveur.

## Interface publique (exposition)
- Types et traits de haut niveau ré-exportés depuis `game_objects`.
- Outils génériques ré-exportés depuis `game_utils`.
- Versionnement de l’API : conserver des chemins de modules stables.

## Dossiers
```
crates/game_core/
  ├─ src/
  │   └─ lib.rs          # uniquement des `pub use` vers les sous-crates
  └─ crates/
      ├─ game_objects/   # logique des objets
      └─ game_utils/     # utilitaires réutilisables
```