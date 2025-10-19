# Threading System — Guide court

## Objectif
Exécuter des jobs courts sur un pool fixe de threads. Mesurer le temps moyen, le max et le nombre d’exécutions par job.

## Modèle
- **Threader**: pool de `N` threads dormants.
- **Job**: nom + closure `FnOnce()` ou `FnMut(&mut Ctx)`.
- **Queue MPMC**: envoi non bloquant depuis n’importe quel thread.
- **Profiling**: horodatage début/fin par job, agrégation atomique.

## API type
```rust
let mut th = Threader::new(num_cpus::get());

let job_id = th.submit(Job::named("mesh_chunk", || {
    mesh_chunk(chunk_id);
}));

th.flush(); // attendre que la file soit vide
```

## Macro pratique
```rust
// Exemple d’idée, adapter à l’implémentation réelle
job!("mesh_chunk", || mesh_chunk(chunk_id));
job!("upload_mesh", || upload_mesh(handle));
```

## Report
- `get_job_report("mesh_chunk") -> {runs, mean_ms, max_ms}`
- `dump_reports_sorted_by_time()` pour diagnostiquer les lenteurs.

## Conseils
- Jobs **courts** (micro‑tâches). Éviter I/O bloquant.
- Regrouper par phase: `Gen → Mesh → Upload`.
- Éviter l’allocation répétée: recycler buffers via pools.
- Limiter le **work stealing** si l’ordre est important.

## Tests rapides
- Saturer avec 10k jobs `sleep(0)` → vérifier la progression.
- Ajouter un job lent → mesurer l’impact sur la latence moyenne.
- Vérifier l’arrêt propre `shutdown()`.
```