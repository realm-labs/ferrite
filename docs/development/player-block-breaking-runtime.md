# Player Block-Breaking Runtime

`G01-P6-S016` implements the source-known portion of `PLY-BLOCK-BREAK-001`. The gameplay runtime
owns client input, local progress, predicted mutation, and sequence state. The existing
`BLK-BREAK-001` runtime remains authoritative for server admission, progress, harvesting, loot, and
world mutation.

## Responsibility boundary

`ferrite-gameplay::player::breaking` is separated into four modules:

- `input` decides whether a held attack is suppressed by a successful click, exits without stopping
  for miss/use/piercing gates, continues a non-air hit, or explicitly stops;
- `session` owns the retained target/tool reference, accumulated Java-float progress, sound counter,
  five-call delay, local crack state, and sequenced start/stop/abort plans;
- `mutation` repeats the local adventure, held-item, game-master, and air gates before the
  callback/fluid/write/destroy-hook transaction;
- `prediction` owns wrapping client sequences, teleport fencing, cumulative server acknowledgement,
  retained authoritative state, flags-19 restoration, and collision-sensitive position snap.

The protocol projection remains responsible for wire packet application and its existing
fastutil-8.5.18-compatible multi-position ACK removal order. The gameplay layer does not duplicate
that specialized hash-table layout.

## Input and break-session order

All attack clicks are consumed before held continuation. A click that reports an instant local
break suppresses held work in the same client tick. Positive miss delay, active item use, and a
piercing weapon return without calling stop. Outside those gates, release, screen focus, mouse
capture loss, or a missing block hit stops the active record; a currently-air block hit returns
without stop.

Restricted or out-of-border starts send nothing and preserve an older record. Replacing a target
sends an unsequenced abort for the old position with the new hit face before beginning the new
prediction. Identical target checks ignore count, accept equal item/components on a replacement
stack, and observe component mutation through the retained object identity.

Ordinary start opens prediction, optionally invokes the zero-progress attack callback, performs
local instant destruction at one-tick progress `>=1.0`, or installs an active record and publishes
crack `-1`. The sequenced START packet follows local mutation. Instabuild performs the local attempt
and START in the same order, installs no active record, and sets the five-call delay even if local
destruction reports false.

Continuation sends a changed carried slot first. Positive delay decrements and succeeds before
mode, border, target, or state validation. A matching non-air record accumulates the current
one-tick progress using `f32`, requests hit sound at counters `0,4,8,...`, increments the float
counter, and publishes the Java float-to-int crack stage below one. Threshold and NaN comparison
outcomes complete prediction with STOP, reset progress/ticks, set delay five, and clear the crack.
Explicit stop uses face `DOWN` and leaves the delay, target, retained item, and sound counter intact.

## Prediction and convergence

Prediction pre-increments a wrapping signed sequence, performs local mutation before packet
construction, and always closes the scope afterward. Successful predicted writes preserve the
pre-write state and first captured player position; later prediction at the same position advances
only the sequence. Local destruction writes the fluid state with flags `11`, and runs the original
block destroy hook only after a successful write.

Authoritative updates stage flags-19 server state while an entry is retained. A cumulative ACK
removes entries through its sequence, restores only differing state, and supplies the captured
position only when the ACK is newer than the last teleport; collision then controls the exact snap.
The server stores the maximum nonnegative received sequence and rejects negative values.

For a successful predicted break, the source-proven logical order is ACK restoration followed by
the later authoritative air update. This implementation preserves that order. It does **not** claim
that a render frame occurs between the two handlers.

## Region ownership

Client plans are non-authoritative projections. Server packets become Region commands after
connection admission; the owning Region serializes `BLK-BREAK-001`, block callbacks, inventory
damage, drops, and replication. Prediction identifiers are convergence metadata, not authority or
Region placement keys.

## Deferred observation

`EXP-PLY-003` remains `DeferredExperiment`. It may replace the observation record only with a
committed, profile-scoped frame capture that preserves TCP byte order and identifies whether any
rendered frame exposes the ACK-restored state before the block update. Either observed outcome must
not alter the source-specified logical state machine.

## Validation

`crates/ferrite-gameplay/tests/slices/player/ply_006.rs` covers held-input gates, replacement order,
ordinary/instant/creative start, delay and item identity, Java-float progress and NaN, explicit stop,
local mutation order, cumulative acknowledgement, teleport/collision restoration, and the explicit
no-render-claim boundary.
