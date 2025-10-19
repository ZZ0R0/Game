# README_client_apply_system.md

## Purpose
Apply filtered `WorldDelta` packets, maintain the client-side known set of `(entity, channel)`, and request full reload on desync.

## Location
`crates/game_client/README_client_apply_system.md`

## Files to implement
- `crates/game_client/src/net_apply.rs`
- Touch points:
  - local mirrors of entities for rendering
  - `ConnShadow` notion mirrored on client to know what is loaded

## Public API (target)
```rust
pub fn apply_packet(state: &mut ClientState, pkt: NetPacket);
```

## Steps
1. Process `RemovedTombstone` first.
2. Apply `AddedFull` for first-time `(entity, channel)`.
3. Apply `ModifiedPatch` for known pairs.
4. Update local mirrors used by renderer/physics.
5. If a consistency check fails, request **full authorized reload**.

## Tests
- Idempotency: re-applying same packet is harmless.
- Reorder tolerance if your transport can reorder.