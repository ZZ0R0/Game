Guideline projet actuel (Dev Engine, Rust) : démarrer par un MVP client‑serveur pour afficher un vaisseau contrôlé par le serveur, sans gameplay. Étapes clés à suivre et à conserver :
1) Contrats de base (unités, axes, IDs),
2) Transport fiable abstrait (framing binaire, version),
3) 4 messages réseau : HELLO/ACCEPT, SNAPSHOT (au join), STATE (updates), BYE,
4) Modèle Ship minimal {ShipId, Transform, MeshId, Tint},
5) Registre d’actifs client avec placeholder si MeshId inconnu,
6) Boucles : serveur tick 20–30 Hz, client frame libre + interpolation ~100 ms,
7) Join/resync par SNAPSHOT complet, IDs stables,
8) Chemin de données “spawn unique” : serveur crée 1 Ship test, client l’affiche et applique STATE périodiques,
9) Observabilité minimale (RTT, tick/s, bytes/s, logs),
10) Tests : latence/perte simulées, asset manquant, version mismatch. Arbo conseillée : net/, proto/, server/, client/, shared/. Étapes futures après MVP : delta state, AOI simple, entrées + prédiction/réconciliation.
