# Threading system — principes et où l’appliquer

## Objectif
Exécuter en **parallèle** les boucles « pour chaque grid / entité / block » sans écritures concurrentes.

## Pattern
- **map → commit** : chaque job travaille en lecture sur l’état courant et écrit dans un buffer local. Un court **commit** séquentiel fusionne les résultats.
- Granularité cible: jobs de 1–4 ms.
- Éviter l’I/O bloquant dans les jobs.

## Où l’utiliser (code actuel)
- Mises à jour par **grille** et par **bloc**.
- Recalculs d’indexation spatiale par shard.
- Préparation des paquets réseau par **joueur** (construction des deltas filtrés).

## Mesure
- Comptes rendus par nom de job: exécutions, moyenne, p95, max.