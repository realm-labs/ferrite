# Required Play Serverbound Entry Protocol

The locked `PROTO-PLAY-SERVERBOUND-ENTRY-001` family contains one packet:
`minecraft:accept_teleportation`, Play serverbound ID `0`. Its entire payload is one signed VarInt
challenge copied from clientbound `minecraft:player_position`.

Ferrite resolves ID `0` through the generated state-and-direction-local packet catalog. Other
recognized Play serverbound packets are refused until their owning family is installed, absent IDs
fail as unknown, and truncation or trailing bytes are terminal codec errors. Signed VarInt
endpoints and accepted nonminimal encodings are preserved by the common wire layer.

## Connection-local synchronization

`TeleportSynchronizer` owns only connection synchronization data:

- the current signed challenge;
- an optional authoritative position awaiting confirmation;
- the listener tick at which that position was sent;
- the three last-good coordinates;
- whether acknowledgement must complete dimension-change bookkeeping.

Issuing a correction increments the challenge. `Integer.MAX_VALUE` wraps to zero; every other
value uses ordinary Java-compatible signed increment. The operation replaces the pending position,
records the current listener tick, and returns the challenge and position needed to construct the
clientbound position packet.

Acknowledgement has three disjoint outcomes:

1. A nonmatching challenge is stale and is ignored without changing pending state.
2. A matching challenge with a pending position accepts that exact stored position, updates all
   last-good coordinates, completes any pending dimension change, and clears the pending position.
3. A matching challenge without a pending position requests disconnect with
   `multiplayer.disconnect.invalid_player_movement`.

Consequently, the first exact response succeeds, its exact duplicate faults, and arbitrary stale
or future values remain no-ops. Challenge zero received before any correction is the third case.

## Movement order and resend

Movement admission is explicitly suppressed while a position remains pending. A movement echo can
never acknowledge a teleport implicitly. Once the matching ID-0 packet clears the pending
position, the following ID-31 movement echo is admitted to the separately owned C2 movement
validator.

At exactly 20 elapsed listener ticks no resend occurs. At more than 20 ticks, the synchronizer
issues a fresh challenge for the same authoritative position, replaces the current challenge, and
refreshes the send tick. Listener-tick subtraction uses Java-compatible signed wrapping, so the
boundary remains correct across `Integer.MAX_VALUE`.

All challenge IDs, pending wire positions, listener ticks, and acknowledgement outcomes remain
connection-local. They are not ECS components, replay commands, persisted world data, transaction
IDs, or Region routing identities.

## Conformance evidence

`crates/ferrite-protocol/tests/c1/play_serverbound_entry.rs` locks:

- the official threshold-256 challenge-one frame `03000001`;
- signed VarInt endpoints, nonminimal encoding, truncation, trailing bytes, and fail-closed family
  dispatch;
- stale, future, matching, duplicate, and matching-without-pending outcomes;
- movement suppression before acknowledgement and validation afterward;
- last-good position and dimension-change completion on the first valid acknowledgement;
- the exact tick-20/tick-21 resend boundary, signed listener-tick wrap, stale old challenges, and
  fresh current-challenge acceptance;
- challenge increment and `Integer.MAX_VALUE`-to-zero wrap.
