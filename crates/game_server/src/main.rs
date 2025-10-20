use game_core::arenas::with_current;
use game_core::ids::PhysicalEntityId;
use game_core::physics::PhysicalEntity;
use game_core::world::World;

fn main() {
    let world = World::new(42, "Main".into());

    // Active le monde courant pour ce thread.
    let _guard = world.scope();

    // Partout dans ce thread: accès sans passer `&mut Arenas`
    with_current(|lock| {
        let mut a = lock.write().unwrap();
        let id = PhysicalEntityId(1);
        let _ = a.insert_physical_entity(PhysicalEntity::new(
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            vec![],
        ));
        if let Some(pe) = a.physical_entity_mut(id) {
            pe.compute_and_apply_pending_deltas();
        }
    });

    // Appels imbriqués/systèmes profonds:
    run_system(); // utilisera with_current(...)
}

fn run_system() {
    game_core::arenas::with_current(|lock| {
        let a = lock.read().unwrap();
        // lecture…
        let _ = a.physical_entity(game_core::ids::PhysicalEntityId(1));
    });
}
