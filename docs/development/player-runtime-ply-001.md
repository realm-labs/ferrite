# PLY-001 Player Runtime

`G01-P6-S013` implements the five `SourceSpecified` player slices primarily owned by `PLY-001`.
The runtime keeps client prediction, server movement authority, geometry, and chunk-distance
admission as explicit responsibilities rather than one connection-local state machine.

## Runtime boundary

`ferrite-gameplay::player` separates the partition into the following owners:

- `input` owns seven-button conflict cancellation, Java-float normalization and square remapping,
  previous-sample crouch slowdown, sprint/flight/pose transitions, independent message baselines,
  and server-retained move intent;
- `collision` owns ordered entity/border/block AABB clipping, Y-first and magnitude-selected X/Z
  axes, first-improving step selection, edge retention, piston accumulation, bounded movement
  records, collision flags, restitution, and block-speed scaling;
- `travel` owns ordinary ground/air input acceleration, jump power and cooldown, climbable clamps,
  collision dispatch, levitation/gravity selection, and friction/drag ordering;
- `auto_jump` owns detector entry gates, actual-motion versus raw-input direction, head clearance,
  ordered entity-then-block probes, Jump Boost rise limits, option-cache refresh, and one-pass
  delayed jump consumption;
- `movement` and `state` retain the authoritative movement-packet admission transaction, including
  finite-value checks, coordinate clamps, load/pending/passenger/sleep gates, packet frequency,
  speed and collision rejection, packet-owned final flags, known movement, and anti-floating;
- `convergence` owns client movement-message selection, the 20-tick reminder, server teleport
  issue/resend/acknowledgement, and client ACK/forced-PosRot/prediction-barrier ordering;
- `spectator` owns the live `spectators_generate_chunks` predicate, delayed movement
  reconciliation, shared player-distance sources, ticket/cap inputs, and an independent ready-chunk
  client projection.

Fluid travel, swimming steering, fall-flying dynamics, and ability-flight wrapper behavior remain
with `PLY-MOVE-SPECIAL-001` in `G01-P6-S014`. Target interaction and block breaking remain with
their named later batches. Collision content is supplied as contextual shapes and material
properties; this partition does not duplicate registry-owned block behavior.

## Exact ordering

Generic movement clips Y before horizontal movement and selects Z-before-X only when
`abs(requested.x) < abs(requested.z)`. Step candidates are deduplicated as floats, sorted
ascending, and stop at the first candidate that strictly improves horizontal squared distance.
Movement flags deliberately retain the separate shape and equality epsilons, while vertical
collision remains an exact comparison. Piston work uses only the first nonzero X/Y/Z component and
the per-tick `[-0.51,+0.51]` accumulator.

Ordinary travel performs jump dispatch before acceleration and movement, then applies
levitation/gravity and finally horizontal/vertical drag. Input shaping uses immutable button
samples, opposing-key cancellation, the `0.98f` base scale, item and sneaking multipliers, and the
unit-square remap before storing movement intent.

Movement packet probes never replace server authority with their clipped position: accepted
packets snap to the clamped packet target and copy both packet status bits, while rejected probes
correct to the pre-packet pose. Pending teleport rotation, strict age-20 resend behavior, exact ID
acknowledgement, and client correction message order are separate convergence state.

## Spectator and Region ownership

The authoritative Region owns each player's admission record, last Section position, shared
players-per-chunk sets, movement state, and collision/travel transaction. A spectator is ignored
by distance sources exactly when the live rule is false, but remains in the player map and retains
an independent client chunk view. Mode or rule changes alone do not reconcile admission; the next
accepted move performs entity projection first, then old-source removal, new-source addition,
ignored-bit update, and view refresh.

All inputs are ordered values, contextual geometry, stable player IDs, and explicit live rules.
There is no ambient RNG, wall clock, node identity, or topology-dependent branch. Region transfer
continues to serialize the authoritative movement state through the established Phase 4 boundary;
client prediction caches and pending projection queues remain session-local.

## Validation

`crates/ferrite-gameplay/tests/slices/player/ply_001.rs` verifies all five slices across input
conflicts and cadence, collision/step/edge boundaries, ordinary travel, auto-jump shape ordering,
movement validation and teleport convergence, and spectator distance/projection independence.
