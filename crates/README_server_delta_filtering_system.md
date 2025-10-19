# README_server_delta_filtering_system.md

## Purpose
Filter the **existing world Delta** per connection using PlayerView, without hashes or journals. Send base state when an entity/channel becomes visible, send diffs otherwise, and send tombstones when access is lost.

## Location
Place this file at:
`crates/game_server/README_server_delta_filtering_system.md`

## Files to implement
- `crates/game_server/src/delta_filter.rs`
- Touch points:
  - `game_protocol` message types
  - `game_objects::player_view` and `knowledge`
  - server tick in `crates/game_server/src/lib.rs` or `sim.rs`

## Public API (target)
```rust
pub struct ConnShadow; // per-connection known (entity,channel) minimal state

pub fn build_conn_delta(
    world_delta: &WorldDelta,    // produced by your existing delta system
    view: &PlayerView,
    shadow: &mut ConnShadow,     // tracks what client already knows
) -> NetPacket;                  // protocol message
```

## Steps
1. Compute PlayerView for connection.
2. For each entry in `world_delta`, **keep only** fields whose `(entity, channel)` is in `view`.
3. For newly visible entries not in `shadow`, send **base initial** subset for allowed channels, then start applying diffs.
4. If a previously known `(entity, channel)` is no longer allowed, emit **tombstone** and drop from `shadow`.
5. Output a compact `NetPacket` defined in `game_protocol`.

## Tests
- Late join: receives base initial then only diffs.
- Permission loss: sends tombstones and client hides fields.