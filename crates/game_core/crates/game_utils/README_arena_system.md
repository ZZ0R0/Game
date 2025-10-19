# Arena system — principes et usages

## Objectif
Fournir des **handles stables** pour référencer des éléments vivants sans invalider les références lors des insertions/suppressions.

## Usages recommandés
- Collections denses de **grids**, **entities**, **blocks**.
- Caches côté client (ex: instances GPU) indexés par handle.
- Tables `Id → Handle` quand l’état réseau exige des IDs stables (IDs définis dans `game_objects`).

## Règles
- Ne jamais exposer d’index brut. Toujours passer par le handle.
- Réutilisation d’un slot = **nouvelle génération** du handle.
- Itération fréquente → préférer Arena à `HashMap` volumineux.
- Les **IDs des objets** ne résident pas ici; ils sont définis dans `game_objects`.

## Bénéfices
- Accès O(1) et itération compacte.
- Stabilité pour le rendu, la physique et le réseau.