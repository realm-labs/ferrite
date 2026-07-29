# Block Interaction and Convergence

`G01-P4-B3` establishes the Region-owned transaction path for the minimal playable world's block
interactions. Java wire state, protocol-neutral targeting, authoritative voxel mutation, committed
replication, and per-connection prediction acknowledgement remain separate responsibilities.

## Ownership path

The implemented path is:

`Play packet -> connection-local prediction registration -> normalized Region command -> Region
voxel mutation -> committed journal -> direct correction and aggregated replication`

The Java 26.2 adapter owns packed positions, strict hands and block-hit directions, modulo
player-action directions, packet IDs, raw block-state projection, and prediction sequence state.
`ferrite-gameplay::block` owns the strict eye-to-unit-AABB reach predicate, reconstructed hit
validation, adjacent targeting, and the bootstrap break-session decision. The server runtime owns
normalization and Region routing. Only the target Region reads or writes its `RegionVoxelState`.

The connection load gate drops destroy, use-on, and use-in-air before sequence registration.
Use-on and use-in-air register immediately after that gate. Destroy start, abort, and stop route
their authoritative command first and register afterward. Registration rejects negative values,
keeps the maximum value seen during the current listener interval, emits one
`block_changed_ack` at the start of the next listener tick, and resets to `-1`. It deliberately
does not retain a greatest-ever ACK floor.

## Wire surface

The serverbound adapter now supports:

| Identity | Important boundary |
|---|---|
| `pick_item_from_block` | packed position and include-data flag |
| `player_action` | strict action, modulo unsigned-byte direction, sequence |
| `swing` | strict main/off hand |
| `use_item_on` | strict hand/direction, three raw floats, both hit booleans, sequence |
| `use_item` | strict hand, sequence, yaw, pitch |

The clientbound adapter supports cumulative `block_changed_ack`, strict single
`block_update`, and `section_blocks_update`. Block positions preserve signed 26/12/26-bit fields;
section positions preserve signed 22/20/22-bit fields. Section changes use the Java wire layout
`x << 8 | z << 4 | y`. Single updates accept only locked global state IDs `0..=32,365`.

## Region command and commit

Destroy and use-on requests become bounded `ferrite:player/block_interaction` commands. The command
contains the normalized player identity, eye position, effective interaction range, target and hit
data, and the selected bootstrap placement state. It does not persist a packet ID, packed
coordinate, or client prediction sequence.

The current bootstrap break path records the state observed at start. A matching stop removes that
same state; a mismatched or empty target is rejected and corrected. Abort clears the transient
session. Use-on selects a replaceable hit position or its face-adjacent position, validates strict
reach and hit components, and mutates only a loaded voxel owned by the executing Region. A pending
teleport permits sequence registration but suppresses mutation and returns authoritative
corrections.

Every processed Region command writes a `ferrite:player/block_result` mutation record. Successful
voxel changes additionally write a `ferrite:world/block_update` replication record. Neither record
is visible to a connection before the tick commits.

## Correction and aggregation

Committed command results select only the originating stable player. Rejected break requests can
carry a single authoritative correction. Every use-on path that reaches the common correction tail
carries the hit position followed by the adjacent position. These direct updates are projected
before ordinary committed replication.

Replication entries from all committed Regions are reduced by block position with last committed
state winning. They are then grouped by section in deterministic section/position order. One
change in a section becomes `block_update`; two or more become one `section_blocks_update`.
Internal `BlockStateId` values pass through the same installed `JavaTerrainRegistryMap` as full
chunks; they are never cast to protocol raw IDs. Missing mappings fail closed. The connection
integration exposes the projected packets and provides a method that enqueues them on the Play
connection.

## Deliberate coverage boundary

This fixed batch supplies the reusable C2 transaction and convergence path; it does not mark
`BLK-PLACE-001`, `BLK-BREAK-001`, or `PLY-BREAK-001` fully verified. Generated gameplay batches
still own inventory/component dispatch, hardness and progress timing, game-mode and permission
branches, content-specific candidate states, multi-block placement, loot, events, crack
publication, cross-Region multi-position placement, and the deferred client render experiment.
The bootstrap session's selected placement state defaults to state ID `1` and is replaceable by
the later authoritative inventory/item owner.

Pick-block inventory effects, auxiliary player actions, swing publication, and held-item use
effects are decoded and normalized at the C2 boundary but remain with their mapped generated
families. No generated family counter advances in this fixed integration batch.
