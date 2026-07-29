# Required Play Serverbound Block Protocol

Ferrite implements all five packets in `PROTO-PLAY-SERVERBOUND-BLOCK-001` for Minecraft Java
26.2:

| ID | Identity | Input responsibility |
|---:|---|---|
| 36 | `minecraft:pick_item_from_block` | clone/select the targeted loaded block result |
| 41 | `minecraft:player_action` | destroy or dispatch one auxiliary player action |
| 63 | `minecraft:swing` | start the selected hand's server swing |
| 66 | `minecraft:use_item_on` | run predicted block-hit interaction |
| 67 | `minecraft:use_item` | run predicted held-item use with supplied look intent |

Packed coordinates, strict hand/action ordinals, modulo player-action directions, strict block-hit
directions, raw float bits, prediction sequences, and packet identities remain inside the Java
26.2 adapter. Accepted requests cross into gameplay and Region ownership only through normalized
commands.

## Wire and mapping boundary

Block positions retain signed 26/12/26-bit packing. Hands accept only 0 and 1. All eight action
ordinals are strict. The player-action direction is the deliberate exception: every unsigned byte
maps by modulo six. Block-hit direction is a strict VarInt in `0..=5`.

Block-hit offsets preserve every IEEE-754 bit pattern during decoding. Handler validation
reconstructs the hit and requires each component's distance from block center to be strictly below
`1.0000001`; NaN, infinity, and equality fail there rather than in the codec. Booleans use the
ordinary nonzero wire rule.

Pick, destroy, and use-on share strict eye-to-unit-AABB reach with padding 1.0. The protocol
adapter does not duplicate authoritative item, inventory, block, or permission state. Those values
are supplied by the gameplay owners after packet normalization.

## Dispatch and sequence ordering

The reusable block dispatcher makes handler/sequence order explicit:

- destroy start, abort, and stop are dropped while client loading is closed; when admitted, the
  authoritative handler runs before sequence registration;
- use-on and use-in-air are dropped by the same gate, then register before any handler work;
- pick, swing, and auxiliary player actions do not consult that loaded gate;
- auxiliary player-action sequences are ignored, including negative values;
- a negative destroy sequence faults only after the destroy handler returned, while a negative
  use sequence faults before its handler.

The connection-local accumulator accepts nonnegative values, keeps the maximum within one listener
interval, emits one clientbound ID-4 ACK at the next tick boundary, resets to -1, and may emit a
smaller value in a later interval.

## Gameplay and convergence integration

The Phase-4 Region path already supplies strict reach and reconstructed-hit predicates,
Region-owned break/use-on commands, pending-teleport suppression, committed mutation, the ordered
pair of immediate use-on corrections, and deterministic block-update aggregation. Pick inventory
effects, item-specific use behavior, auxiliary actions, swing publication, permissions, progress,
loot, and content-specific consequences remain owned by their generated gameplay batches; this
protocol batch does not advance those slice counters.

## Evidence

- `crates/ferrite-protocol/tests/c2/play_serverbound_block.rs` owns all five exact goldens,
  ordinal/position/float boundaries, malformed input, gate behavior, and registration order.
- `crates/ferrite-gameplay/src/block/targeting.rs` owns strict reach and reconstructed-hit tests.
- `crates/ferrite-server-runtime/tests/block_interaction.rs` owns normalized Region routing,
  committed mutation, corrections, and convergence.
