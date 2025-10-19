# game_protocol — Protocole réseau et deltas

## Rôle
Définir les **messages** échangés entre client et serveur, y compris les deltas minimaux. Le README racine indique que ce crate gère le protocole, la reconnexion, plusieurs clients et la sélection des données envoyées avec deltas.

## Contenu attendu
- **Messages** : handshake, auth simple si prévu, `WorldDelta` (added/modified/removed), `ClientAck` (seq reçu), snapshots compacts.
- **Schémas de sérialisation** : format binaire stable, versionné.
- **Erreurs et codes d’état** : timeouts, désynchronisation, resync via snapshot.

## N’y met pas
- Pas de logique de jeu, pas de rendu.
- Pas d’algorithme AOI/Knowledge ici. Uniquement la **forme** des données sur le fil.

## Interfaces clés exposées
- Types de messages et encodeurs/décodeurs.
- Constantes de version de protocole.
- Utilitaires de framing/fragmentation si nécessaires.

## Dossiers
```
crates/game_protocol/
  ├─ src/
  │   ├─ messages.rs      # structures de messages (delta, ack, snapshot)
  │   ├─ encode.rs        # sérialisation
  │   └─ version.rs       # versionnement du protocole
  └─ Cargo.toml
```

## Intégrations
- Consommé par `game_server` pour construire et envoyer les messages.
- Consommé par `game_client` pour décoder et appliquer.