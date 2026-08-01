# Composite Region Runtime

## Responsibility

`ferrite-server-runtime::composite` is the production Region coordination boundary. It does not
replace the 20 low-level `TickPhase` barriers owned by `ferrite-simulation`; it groups service work
into the nine production-integration stages that must remain visible from ingress through client
projection:

1. ingress capture and normalization;
2. player and item service work;
3. block, environment, fluid, lighting, and redstone simulation;
4. entity, combat, mob, AI, tracking, and transfer work;
5. world lifecycle, ticket, generation, and dimension work;
6. cross-service and cross-Region reconciliation;
7. continuity preparation;
8. authoritative commit;
9. post-commit projection.

`CompositeStage::ALL` is the stable order. A tick enters and completes each stage exactly once.
Out-of-order entry, overlapping stages, missing continuity preparation, non-sequential ticks, and
projection production after commit all fail explicitly without advancing the coordinator.

## Typed queues and budgets

Commands and projections name a `CompositeOwner`, stable sequence, responsibility-owned
`ResourceId`, and bounded payload. Commands are canonically keyed by tick, owner, and sequence, so
admission order does not affect replay. The runtime separately bounds commands, lifecycle events,
post-commit projections, continuity records, payload bytes, and the future-tick horizon. Queue
overflow retains the active stage and returns the owning capacity in the error.

Events identify the completed stage and carry the replay identity on the commit event. Consumers
may drain events and committed projections independently without changing authoritative state.

## Commit and replay boundary

Continuity preparation accepts current responsibility identities and unreserved auxiliary records.
Legacy continuity identities are read-only and cannot enter a new composite commit. The commit
identity hashes the complete Region identity, activation generation, tick, canonical command
order, fixed stage order, continuity hash, and canonical pending-projection order.

Before the commit stage completes, projections remain private. Commit advances the authoritative
tick, prunes the committed command prefix, and atomically publishes the pending projections to the
post-commit queue. The projection stage may then drain them under transport backpressure.

`G03-P2-B2` and `G03-P2-B3` install the concrete service runtimes behind these stage and queue
boundaries. `G03-P2-B4` installs that coordinator in the formal Region runner and gateway.

## Formal gateway route

`CompositeRegionRouter` is the only Region command and player-transfer route owned by
`MinecraftGateway`. It contains the low-level `LocalRegionRunner` executor and exactly one
`CompositeProductionRegionRuntime` for each active Region. The local executor retains ownership of
the 20 deterministic simulation barriers, required transfer delivery, and the transient
`PlayerSessionState` used to acknowledge movement to a connection. It no longer selects or
receives a production gameplay-logic implementation from the gateway.

At reconciliation, the adapter compares stable player identities in the executor state with the
composite player service. Joins, disconnects, and completed Region transfers become typed
`JoinPlayer` or `LeavePlayer` commands in stable identity order. At the final low-level commit
barrier, every Region must complete all nine composite stages. The route returns both the local
executor report needed by existing session acknowledgements and the composite reports containing
service outcomes, continuity candidates, projections, and commit receipts. Missing a composite
commit for any formal Region fails the whole formal tick.

The historical `PlayerRegionLogic` remains a focused conformance fixture for the low-level runner;
the formal listener does not instantiate or execute it. Post-commit delivery consumes the
composite reports in `G03-P3-B2`; durable storage consumption remains explicitly incomplete in the
production manifest.

## Simulation and player-service installation

`CompositeProductionRegionRuntime` owns one coordinator, one `SimulationRegionRuntime`, and one
`PlayerServiceRegionRuntime` with the same Region key, activation generation, and committed tick.
Typed service commands cover player join and leave, player/item mutation, menu lifecycle, and
simulation schedule admission. Admission also creates the canonical coordinator command, so replay
identity includes the complete semantic payload rather than only a dispatch tag.

The player-service stage preflights composite projection capacity before it mutates any player.
Player projections are removed from the service queue and installed in the coordinator's private
pre-commit queue. The simulation stage admits scheduled block/fluid work to the bounded simulation
queues. The continuity stage captures simulation and player records into one current-generation
set. Only after the coordinator commit succeeds does the simulation clock advance; the projection
stage then exposes the committed projection prefix and the tick report drains all nine lifecycle
events.

Any service execution error poisons that Region runtime, matching the fail-stop behavior of the
local Region runner and preventing partially executed service state from being retried as if it had
rolled back. Capacity failures that can be preflighted, including player projection backpressure,
occur before authoritative mutation.

## Entity, world, and reconciliation installation

The production runtime also owns `EntityServiceRegionRuntime` and `WorldServiceRegionRuntime`
instances created with the same Region identity. The entity stage handles insertion, observer
admission, and mutation. Entity projections are drained for every stable observer and installed in
the same private pre-commit queue as player projections. The world stage owns chunk demand and
revision-fenced voxel mutation.

The reconciliation stage has typed commands for simulation boundary transactions and the complete
two-phase entity-transfer protocol. A boundary transaction mutates the world service's Region voxel
state through the simulation runtime's generation and replay fences, drains every resulting
mechanic effect as a typed outcome, and converts authoritative block changes into protocol-neutral
semantic projections. Entity transfer is routed as prepare, target accept/idempotent receipt, and
source commit commands; the transfer state and receipt are included in canonical replay metadata.

Continuity preparation now joins simulation, player, entity, applied-transfer receipt, world chunk,
and auxiliary records. Commit retains this exact current-generation record set as
`CommittedCompositeContinuity`; the next tick cannot start until the consumer takes it. This avoids
an unbounded persistence handoff queue and makes omission of a committed durable candidate
explicit. The tick report consumes and exposes that record set together with its count and hash.

Command and projection encoders live under `composite::services::codec`, separate from authority
execution, so the production service coordinator remains below the source-size limit and replay
identity encoding has one owner.
