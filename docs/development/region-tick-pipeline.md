# Region Tick Pipeline

`G01-P2-B2` fixes the deterministic execution boundary shared by local and distributed Region
runtimes. It defines logical order and bounded admission; it does not depend on an executor, wall
clock, transport, or Lattice type.

## Logical time and phases

`GameTick` is a checked `u64` logical clock. A `RegionTickPipeline` may begin only the exact successor
of its last committed tick and may have only one active tick. Every tick traverses these stable
phases:

1. `Begin`
2. `Ingress`
3. `NormalizeCommands`
4. `PlayerIntent`
5. `ScheduledBlocks`
6. `RandomBlocks`
7. `ImmediateNeighbors`
8. `BlockEntities`
9. `Fluids`
10. `Redstone`
11. `EntityAi`
12. `EntityPhysics`
13. `EntityResolution`
14. `DeferredChanges`
15. `ResultingNeighbors`
16. `EcsStructuralChanges`
17. `EmitBoundary`
18. `ReconcileBoundary`
19. `Replication`
20. `Commit`

The ordinal tags and successor relation are locked by tests. Phase advancement is monotonic and
cannot skip an unsatisfied barrier. Structural changes must be applied before leaving
`EcsStructuralChanges`; boundary output must be sealed before leaving `EmitBoundary`; required input
must be reconciled before leaving `ReconcileBoundary`. `Commit` is reachable only through the fixed
sequence and must use the explicit commit operation.

Queue overflow uses retain-and-backpressure semantics. It never drops, replaces, or reorders accepted
work.

## Command admission

Commands carry a target Region, logical tick, stable source, source sequence, semantic kind, and
bounded payload. The inbox rejects wrong targets, committed or skipped ticks, ticks beyond its
configured horizon, duplicate order keys, oversized payloads, and capacity overflow before mutation.

Accepted commands are ordered by:

1. tick;
2. source identity;
3. source sequence.

Admission order therefore cannot affect execution order. Duplicate keys remain fenced after a tick is
drained and are pruned only when that tick is committed.

## Journals and commit

Each active tick owns a bounded append-only semantic journal. Entries record the current phase,
monotonic per-tick sequence, domain, semantic kind, and bounded payload. A successful commit consumes
the active journal and publishes an immutable `CommittedTickJournal`; failed phase or capacity checks
do not partially commit.

The journal is the semantic input to later persistence and replay batches. It deliberately contains
no Bevy handles, task identities, packets, or transport values.

## Boundary batches

A boundary batch identifies its tick, phase, source and target Regions, source activation generation,
source sequence, and a bounded set of semantic events. Events are sorted by explicit event order and
duplicate event orders are rejected. A Region cannot send a boundary batch to itself.

The receiving inbox validates the target, expected source generation, committed-tick fence,
duplicate key, and capacity before admission. Accepted batches are drained canonically by:

1. tick;
2. phase;
3. source Region key;
4. source sequence.

The generation tag prevents messages from a retired Region activation from mutating its replacement.
Transport acknowledgement, retry, and durable outbox behavior remain owned by the recovery and
Lattice adapter batches.

## Canonical state projection

`ferrite-replay` hashes an immutable semantic projection rather than storage layout. Block records use
persistent resource identities and positions; entity records use stable entity IDs, persistent kinds,
and bounded semantic state; extension records use stable domain/key pairs. Construction validates
Region ownership, bounds, sorting, and uniqueness.

The Region key, mapping size, committed tick, schema tag, and ordered projection participate in the
Region hash. The world hash sorts Region hash records and also includes the world ID, committed tick,
and content-manifest hash. Locked vectors cover both levels and make topology-independent comparison
available to the local runner and later multi-node tests.

`G01-P2-B3` will connect these primitives to cross-Region transfer and a deterministic local runner.
`G01-P2-B4` will make committed journals and projections durable and recoverable.
