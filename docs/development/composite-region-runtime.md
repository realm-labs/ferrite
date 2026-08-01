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
boundaries. `G03-P2-B4` adapts the composite coordinator to the formal Region runner and gateway.

## Simulation and player-service installation

`SimulationPlayerRegionRuntime` owns one coordinator, one `SimulationRegionRuntime`, and one
`PlayerServiceRegionRuntime` with the same Region key, activation generation, and committed tick.
Typed service commands cover player join, player/item mutation, menu lifecycle, and simulation
schedule admission. Admission also creates the canonical coordinator command, so replay identity
includes the complete semantic payload rather than only a dispatch tag.

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
