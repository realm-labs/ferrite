# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-PREDICT-001` — Client prediction is provisional and server correction is authoritative

**Parent:** `CLI-001`, `CLI-002`, `CLI-003`, `CLI-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — this aggregate mixes frame timing, input, prediction, and movement
correction; exact event priorities, thresholds, and convergence branches remain unexpanded.

**Applies when:**

The client locally predicts movement, interaction, block state, inventory, or use animation before
receiving the server result.

**Authoritative state:**

Server world/player/inventory state, client predicted state, sequence/state identifiers, pending
teleport/correction and acknowledgement state.

**Transition and ordering:**

Client samples input, performs allowed local prediction and sends request with relevant ordering
token; server processes against its current state; on acceptance normal authoritative updates
converge the client; on rejection or divergence server sends state/position/menu correction; client
applies correction and acknowledges where required before later requests can be trusted.

**Branches and aborts:**

Prediction disabled for operation; accepted exactly; accepted with different result; rejected
permission/reach/collision/state; stale sequence/menu state; correction arrives after later
predictions; teleport acknowledgement outstanding.

**Constants and randomness:**

Network delay/reordering is not gameplay RNG. Position and inventory comparisons use
operation-specific numeric/state identifiers. Do not use render interpolation as authoritative
simulation.

**Side effects:**

Temporary local block/pose/slot/swing, request packet, authoritative block/entity/slot/position
updates, rollback/resync, acknowledgement and possibly replay of later local input. Sounds/particles
may be locally predicted only for explicitly client-originated paths.

**Gates:**

Operation prediction support, connection state, sequence/state ID, pending correction, game mode,
server validation and client screen/state.

**Boundary cases and quirks:**

A visible local success can be rolled back. Corrections must not duplicate item consumption or
effects. Ordering tokens prevent an old rejection from overwriting newer authoritative state.

**Evidence:**

`Confirmed` authority model; reordering matrix `Implementation blocker`; `OFF-CLIENT-001`,
`OFF-SERVER-001`; `EXP-CLI-001`.

**Test vectors:**

Inject latency/reorder for placement, breaking, movement and menu click; server mutates target
before request; two predictions before first correction; assert final state and one-time side
effects.
