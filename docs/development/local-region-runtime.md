# Local Region Runtime

`G01-P2-B3` connects the executor-neutral tick contract to an in-process multi-Region runner. The
local runner uses the same semantic commands, boundary records, activation generations, transfer
records, and Region state that later Lattice adapters must carry. It is not a separate single-player
or test-only simulation model.

## Consistency island

A `LocalRegionRunner` owns an explicitly registered, bounded set of active Regions. That set is one
local consistency island, not an implicit world-wide or cluster-wide barrier. A caller may operate
independent islands separately when they have no immediate shared-order requirement.

Within an island, Regions are always visited by `SimulationRegionKey`, regardless of insertion order.
All Regions traverse one phase before any advances to the next phase. Commands are delivered at
`Ingress`; boundary batches are delivered at their tagged future phase. A batch emitted after its
target phase has run is rejected instead of being delayed accidentally.

The runner admits only the exact successor tick for every member. Commit is preflighted across the
whole island and publishes immutable per-Region journals in Region-key order. Future queued work may
remain, but required work through the committing tick cannot.

## Immediate boundary effects

An `ImmediateBoundaryEffect` identifies:

- tick and phase;
- source and target Region;
- source and target activation generation;
- source sequence;
- semantic kind and bounded payload.

After normal logic has executed for every Region in a phase, same-phase effects are sorted by target,
source, and source sequence and applied through the Region logic interface before the phase barrier
can complete. This preserves immediate cross-boundary ordering without making mailbox arrival or
thread completion observable.

The queue rejects incompatible ownership domains, self-targets, stale generations, committed ticks,
duplicates, oversized payloads, and overflow before mutation. Duplicate fences remain until commit.

## Entity and player transfer

`EntityTransfer` uses the same dual-generation fence and adds the stable entity ID, an explicit
entity/player role, persistent kind, and bounded complete semantic state. Transfers emitted before
reconciliation are sorted by target, source, and source sequence, then applied at
`ReconcileBoundary`.

The target is preflighted before mutation. It receives the same `StableEntityId` and an immutable
`TransferredEntityState` materialization record; only after target creation succeeds is the source
ECS entity removed. If source removal fails, the target copy is removed and the tick fails.
Gameplay-owned codecs in later subsystem batches must treat the transfer payload as the complete
portable state and materialize their components from it; Bevy entity handles and archetype layout
never cross the boundary.

Both endpoint generations are checked at admission and again at application. A replacement
activation therefore cannot accept work admitted for either an old sender or an old receiver.

## Logic boundary and failure

`RegionLogic` receives the active Region state, current tick/phase, and canonically ordered admitted
inputs. It emits work through a bounded `RegionPhaseOutput`; commit cannot emit new work. Immediate
effects use a separate callback so the runner can establish the same-phase merge barrier explicitly.

If logic, routing, a queue, a barrier, or a transfer fails during a tick, no new committed tick is
published. The runner becomes poisoned and refuses further admission or execution. `G01-P2-B4`
provides the snapshot/journal recovery path that replaces this failed activation with a validated
new generation.
