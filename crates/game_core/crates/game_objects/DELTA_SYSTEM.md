# Delta System — Guide court

## Objectif
Envoyer et appliquer uniquement les changements entre serveur et client. Réduire la bande passante et le coût CPU par rapport à l’envoi de snapshots complets.

## Vocabulaire
- **Snapshot**: état complet à un instant t.
- **Delta**: différences entre deux états consécutifs.
- **Scopes**: World, Grid, Block. Un delta peut être composé hiérarchiquement (WorldDelta contient des GridDelta, etc.).

## Modèle de données minimal
Chaque scope porte trois collections :
- `added`: éléments nouvellement créés.
- `modified`: éléments existants avec champs modifiés.
- `removed`: identifiants supprimés.

Chaque élément possède:
- `id`: identifiant stable (par type).
- `ver`: version/epoch croissante pour l’ordonnancement.
- `fields`: données (complet pour `added`, patch partiel pour `modified`).

## Dirty flags
- Chaque objet a un `dirty_mask` par sous‑champ ou groupe logique.
- Mutations doivent **marquer** le bit correspondant.
- La construction d’un delta lit les bits et les **réinitialise** après export réussi.

## Construction côté serveur
1. Mutation de l’état → marquage dirty.
2. Au tick réseau, parcourir scopes visibles pour le client.
3. Construire `WorldDelta` avec:
   - Grids visibles: `{added, modified, removed}`.
   - Pour chaque Grid modifiée: `GridDelta` avec sous‑ensembles de Blocks.
4. Sérialiser et envoyer.

### Pseudo‑API
```rust
// exemples d’API possibles, à adapter au code réel
let delta = world.build_delta_for(client_view_frustum);
net.send(client_id, &delta);
world.clear_dirty_until(delta.ver);
```

## Application côté client
1. Valider `ver` monotone par scope.
2. Appliquer `removed` (detach, free) avant `added` et `modified`.
3. Propager aux sous‑systèmes:
   - **Rendu**: mettre à jour les buffers seulement pour objets touchés.
   - **Physique**: recharger colliders des entités modifiées/supprimées.

### Pseudo‑API
```rust
let delta: WorldDelta = net.recv()?;
client_world.apply(delta, |evt| {
    renderer.on_world_delta(evt);
    physics.on_world_delta(evt);
});
```

## Ordonnancement et conflits
- Rejeter un delta si `ver` < `last_applied_ver`.
- En cas de perte réseau, accepter `ver` > `last+1` si les deltas sont **auto‑contenus** (chaque `modified` porte les champs nécessaires). Sinon, redemander un snapshot rapide.

## Conseils d’implémentation
- Ids typés et stables (`WorldId`, `GridId`, `BlockId`).
- Versions par scope pour éviter un global lock.
- Patches compacts: encodez `modified` sous forme `{field_id → value}`.
- Sérialisation compacte (ex: un format binaire stable).

## Exemple minimal (grille et blocs)
```rust
struct GridDelta {
    ver: u64,
    added: Vec<GridFull>,
    modified: Vec<GridPatch>,
    removed: Vec<GridId>,
    blocks: BlocksDelta, // sous-scope
}

struct BlocksDelta {
    added: Vec<BlockFull>,
    modified: Vec<BlockPatch>,
    removed: Vec<BlockId>,
}
```

## Tests rapides
- Mutations unitaires: créer/supprimer/modifier un bloc → delta attendu.
- Idempotence: appliquer deux fois un même delta ne doit rien casser.
- Perte de paquet: sauter `ver=N`, appliquer `ver=N+1` selon les règles.