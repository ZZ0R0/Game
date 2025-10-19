# README_playerview_system.md

## Purpose
Build the **per-connection view** of allowed `(entity, channel)` pairs at time T by combining AOI (physical load) and Knowledge (field filter).

## Location
`crates/game_core/crates/game_objects/README_playerview_system.md`

## Files to implement
- `crates/game_core/crates/game_objects/src/player_view.rs`
- Touch points:
  - `src/aoi.rs`
  - `src/knowledge.rs`
  - `src/world.rs`

## Public API (target)
```rust
pub struct PlayerViewEntry { pub entity: EntityId, pub channel: Channel }

pub struct PlayerView {
    pub player: PlayerId,
    pub entries: smallvec::SmallVec<[PlayerViewEntry; 256]>,
}

impl PlayerView {
    pub fn build(world: &World, aoi: &Aoi, kg: &Knowledge, player: PlayerId, center: Vec3, r_m: f32) -> Self;
}
```

## Steps
1. Call AOI sphere query around the player to get **PhysicalSet**.
2. Call Knowledge to extend with **Logical-only** entities reachable via RF/ownership/session.
3. Produce `entries` unique by `(entity, channel)`.
4. Keep PlayerView as a **pure read model**; no mutation of world state.

## Tests
- Player sees an enemy grid physically but has no logical access.
- Player has logical access via antenna to a far-away friendly grid.