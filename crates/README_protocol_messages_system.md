# README_protocol_messages_system.md

## Purpose
Define wire messages for **filtered deltas**, base initial, tombstones, and ACKs.

## Location
`crates/game_core/crates/game_protocol/README_protocol_messages_system.md`

## Files to implement
- `crates/game_core/crates/game_protocol/src/messages.rs`
- Optional helpers in `encode.rs`

## Message shapes (target)
- `NetPacket::WorldDelta { entries: Vec<DeltaEntry> }`
- `DeltaEntry { entity_id, channel_id, payload_kind, payload }`
  - `payload_kind = AddedFull | ModifiedPatch | RemovedTombstone`
- `ClientAck { upto_tick: u64 }` if needed

## Steps
1. Add channel ids (Physical, Logical, LogicalBlock(BlockId)).
2. Add entry kinds for base initial, patch, tombstone.
3. Keep binary layout stable and versioned in `version.rs`.
4. Ensure decode path in client and encode path in server.

## Tests
- Round-trip encode/decode golden tests.
- Backward-compat with empty logical channel.