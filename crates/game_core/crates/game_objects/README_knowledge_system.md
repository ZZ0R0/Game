# README_knowledge_system.md

## Purpose
Define **what fields a player is allowed to receive** at time T. Knowledge is a **field filter** independent from AOI. Two channels per entity:
- **Physical**: pose/shape/visual state (LOS/AOI-driven).
- **Logical**: inventories, terminals, ownership, internal stats (antennas/ownership/terminal session).

## Location
Place this file at:
`crates/game_core/crates/game_objects/README_knowledge_system.md`

## Files to implement
- `crates/game_core/crates/game_objects/src/knowledge.rs`
- Optionally: `src/knowledge_rf.rs` if you split RF/antenna routing
- Touch points:
  - `src/factions.rs` (ownership/permissions)
  - `src/blocks/*` (terminals, inventories)
  - `src/players.rs` (active terminal session with TTL)

## Public API (target)
```rust
pub enum Channel { Physical, Logical, LogicalBlock(BlockId) }

pub struct Knowledge;

impl Knowledge {
    pub fn can_access_entity(&self, p: PlayerId, e: EntityId, chan: Channel) -> bool;
    pub fn can_access_field(&self, p: PlayerId, e: EntityId, field: FieldId) -> bool;
    pub fn begin_terminal_session(&mut self, p: PlayerId, e: EntityId, ttl_ticks: u32);
    pub fn end_terminal_session(&mut self, p: PlayerId, e: EntityId);
    pub fn tick_expire(&mut self); // drop TTL-based capabilities
}
```

## Rules
- Physical channel requires AOI + LOS.
- Logical channel requires at least one **capability** source:
  - Antenna path from player to entity (RF relay) with security pass.
  - Ownership/faction rule.
  - Active terminal session (cap ephemeral).

## Steps
1. **Model channels** and map fields to channels. Document per entity and per block.
2. **Capabilities engine**: for each player, compute capability facts for `(entity, channel)` from LOS, RF, ownership, terminal.
3. **TTL management**: sessions and RF links may expire. Run `tick_expire()` each frame.
4. **API**: expose `can_access_*` used by the PlayerView and by the server filtering stage.

## Tests
- Access variations when relay toggles on/off.
- Mixed-faction blocks inside one grid.
- Terminal session grant/revocation timing.