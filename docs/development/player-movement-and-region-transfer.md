# Player Movement and Region Transfer

`G01-P4-B2` adds the first authoritative player loop after Play installation. The implementation
keeps Java 26.2 connection state, protocol-neutral movement admission, Region-owned ECS state, and
cross-Region ownership commits in separate modules.

## State ownership

The player path has four owners:

1. `ferrite-protocol::java_26_2` owns packet IDs, field order, framing, keepalive challenges,
   teleport IDs, pending correction state, and disconnect presentation.
2. `ferrite-gameplay::player` owns the version-independent pose, velocity, last-good positions,
   movement counters, client-load gate, known movement, collision decision, and floating state.
3. `ferrite-server-runtime::player` normalizes the four Java movement variants, routes semantic
   state updates, drives per-session chunk interest, and coordinates explicit player transfers.
4. The active `SimulationRegion` owns the stable player entity and its `PlayerSessionState`
   component. Neither Java packet types nor teleport/keepalive challenges enter that ECS state.

Admission now carries the immutable Region mapping and a normalized spawn pose. The bootstrap flat
world places the player at the center of the selected spawn chunk with feet at Y 65. The
`ferrite:session/join` command creates the stable entity and component during Region ingress; it is
not created in the socket task.

## Java 26.2 session boundary

The serverbound adapter decodes these required C2 packets:

| Identity | Body |
|---|---|
| `chunk_batch_received` | desired chunks/tick float |
| `client_tick_end` | empty |
| `keep_alive` | signed long |
| `move_player_pos` | XYZ doubles and flags |
| `move_player_pos_rot` | XYZ doubles, yaw/pitch floats, and flags |
| `move_player_rot` | yaw/pitch floats and flags |
| `move_player_status_only` | flags |
| `player_loaded` | empty |

Position and rotation omission remains structural: an omitted field retains the current
authoritative value. Flag bits 0 and 1 are on-ground and horizontal collision; high bits decode as
ignored and re-encode canonically.

The clientbound adapter adds trusted component-NBT disconnect, keepalive, absolute vehicle
correction, and independent-relative player rotation. Exceptional IEEE-754 values remain wire
data. Disconnect reasons use translatable component NBT and the socket closes only after the
disconnect frame completes.

`ServerConnection` now continues decoding after it enters Play. It emits bounded Play packet
events, handles keepalive echoes and teleport acknowledgements connection-locally, and exposes
explicit correction and disconnect operations. A matching teleport acknowledgement without a
pending correction queues `invalid_player_movement`; a stale ID is ignored. A pending correction
suppresses position handling through the normalized event flag. Its resend age advances with the
connection listener tick and replaces the challenge only after more than 20 ticks.

For non-owner sessions, the first Play tick establishes the keepalive baseline. At 15 seconds the
connection sends the current millisecond clock. Another 15 seconds with that challenge pending
disconnects for timeout. Only an exact signed-long echo clears it and updates latency with
`(old * 3 + round_trip) / 4`. Singleplayer-owner sessions skip both challenge and invalid-echo
failure.

## Movement transaction

Movement validation preserves the reviewed ordering:

1. reject position NaN and non-finite rotation before load or teleport gates;
2. retain position infinities until the X/Z `±30,000,000` and Y `±20,000,000` clamps;
3. wrap supplied rotations and apply won-game, 60-tick load, pending-teleport, passenger, and
   sleeping branches;
4. apply the normal-tick packet multiplier, including the locked `>5 -> 1` behavior, and the
   `100N` or fall-flight `300N` displacement test;
5. consume a collision probe, apply the `0.0625` horizontal residual test, preserve the locked
   always-zero residual-Y defect, and reject newly introduced collision;
6. on success, snap to the exact clamped packet target and use the packet's two movement flags;
7. derive known movement and gravity-scaled floating timeout state.

The collision decision accepts a `CollisionWorld` probe so Region/world code supplies geometry
without entering the protocol adapter. `FlatWorldCollision` provides the Phase 4 bootstrap floor,
and `NoCollision` supports isolated adapter tests. This batch implements the player validator's
collision-admission transaction; it does not mark the complete generic swept-AABB
`PLY-COLLISION-001` leaf verified. Exact shape clipping, stepping, bounce, friction, and registry
properties remain with their generated gameplay batch.

Every connection-side state mutation first prepares the Region command or transfer. Routing failure
restores the previous session state. The session keeps separate working and committed snapshots;
only a command receipt from a successfully committed tick advances the latter. Same-Region
movement uses a sequenced
`ferrite:player/state` command. `player_loaded`, client tick end, pending-teleport rotation, and
passenger rotation also project their state mutations instead of remaining socket-only.

## Explicit Region transfer

Crossing the versioned mapping boundary creates an `EntityTransfer` for the next authoritative
tick. It carries:

- stable player identity and `Player` role;
- source and target Region keys;
- both activation generations;
- source sequence and target tick;
- a bounded, deterministic `FPS1` encoding of the complete protocol-neutral player session state.

The source remains the session owner after admission. At `ReconcileBoundary`, the local runner
checks both generations, journals both sides, creates the target entity, installs the transfer
record, and removes the source entity. `PlayerRegionLogic` then validates and materializes the
typed player component in the target Region.

`LocalTickReport` returns committed transfer receipts only after every Region commits the tick.
The connection-side `PlayerSession` changes its owner key only after a receipt matches tick,
source, target, stable identity, and player role. Until then, additional movement waits behind the
handoff. `JavaPlayerConnection` recenters chunk interest only after that same receipt, preventing a
client-visible ownership switch from preceding authoritative commit. Same-Region recentering is
likewise released only by the matching committed command receipt.

The routing interface exposes command admission, transfer admission, and activation-generation
lookup without depending on local executor types. The local runner implements it now; the
Lattice-backed topology adapter and trace-equivalence proof remain `G01-P4-B4`.

## Composition point

The B2 path is:

`Play frame -> Java packet event -> PlayerSession validator -> Region command or fenced transfer -> committed receipt -> chunk recenter / correction / disconnect`

Block targeting and prediction correction continue in
[Block Interaction and Convergence](block-interaction-and-convergence.md). Distributed trace
equivalence and the unmodified-client C2 acceptance path remain `G01-P4-B4` and `G01-P4-B5`.
