# ADR-0006: Deterministic Region Tick and Boundary Order

## Status

Accepted

## Context

Region execution may be sequential, parallel, or remote. Mailbox arrival, thread completion, hash
iteration, and wall-clock scheduling cannot be allowed to change observable gameplay.

## Decision

Every Region executes the canonical 20-step pipeline defined in `docs/architecture.md`, beginning
with tick admission and ending with committed tick publication. The order is:

```text
begin -> ingress -> normalize -> player intent
-> scheduled blocks -> random blocks -> immediate neighbors
-> block entities -> fluids -> redstone
-> entity AI -> physics/collision -> damage/death/drops/spawns
-> deferred changes -> resulting neighbors -> ECS structural changes
-> boundary batches -> required reconciliation -> replication/effects -> commit
```

Each phase declares mutable state, ordering keys, structural-change points, budgets, overflow
behavior, and whether reconciliation is required. Cross-Region messages are bounded
`BoundaryBatch` values tagged with world tick, phase, source Region, activation generation, and
source sequence. Receivers validate and deduplicate those tags before mutation.

Only mechanics that need immediate shared ordering form a scoped barrier or
`ConsistencyIsland`. There is no cluster-wide or unconditional world-wide tick barrier.

## Consequences

- Local and distributed executions can be compared by committed hashes.
- Overload is explicit: required work waits, backpressures, or fails visibly; it is not silently
  dropped.
- Phase changes require regression vectors and a superseding or amended decision record.
- Some parallelism is intentionally delayed until deterministic merge rules exist.

## Alternatives Considered

- Independent actor interval timers: rejected because drift and delivery timing become gameplay.
- One global cluster barrier: rejected because unrelated worlds and Regions would block each other.
- Arrival-order reconciliation: rejected because network topology becomes observable.

## Migration or Reversal Plan

Version the phase contract and replay header, migrate or reject incompatible snapshots, and prove
old/new behavior with reference rules plus topology-equivalence tests before changing order.
