# Durable World-Service Integration (Historical Goal 01 Phase 8)

The active `ferrite-server-runtime::world_service` module joins the audited generation and dimension
models to Region ownership, durable recovery, handoff, lifecycle ordering, and offline inspection.
The runtime owns coordination only: chunk storage remains in `ferrite-world`, commit selection
remains in `ferrite-persistence`, and the inspector consumes those stable formats without depending
on the server runtime or foundation internals.

## Chunk identity and generation publication

Every operation is scoped by a `SimulationRegionKey` and `ActivationGeneration`. A generation
request also captures its request identity, source chunk revision, next adjacent `ChunkStatus`, and
content-manifest digest. Completion publishes only when all of those values still match and the
generated chunk retains its position and a non-regressed revision. An authoritative edit may occur
while work is in flight; the resulting revision mismatch makes the completion stale instead of
overwriting newer state.

Status publication advances exactly one audited stage. Activity independently advances through
`Dormant`, `Accessible`, `BlockTicking`, and `EntityTicking`. Promotion to block ticking emits the
persisted-tick unpack event first. Promotions, demotions, saves, unloads, and cancellation use a
bounded, monotonically sequenced event queue. Capacity is preflighted before mutation so
backpressure cannot leave a partial state transition.

Demand loads an empty owned column or cancels the exact pending-unload token. Unload preparation
captures the current revision. Saving creates a `RegionRecoveryPoint`, but a chunk is not torn down
until `apply_save_receipt` receives the matching receipt from the actual `RegionFileStore` commit.
Receipt identity, revision, committed tick, and digest are checked before `Saved` and `Unloaded`
events are emitted.

## Durable representation and recovery

The `FWC1` chunk codec stores exact block-state and biome runtime IDs, dense sections, block
entities, and chunk/section revisions under explicit size, section, and block-entity limits. Those
numeric IDs are not a global persistence namespace: they are valid only under the exact
content-manifest digest stored in the enclosing Region snapshot. Restore refuses a different
manifest before admitting any column, making reconstruction deterministic for the locked content
set.

The `P8C1` record adds status, activity, and pending-unload continuity. Snapshot records and the
contiguous journal tail are materialized by canonical record identity before their state hash is
verified. Restore then checks Region identity, a strictly newer activation generation, mapping,
Region side, layout, ownership, uniqueness, and capacity. Non-chunk records survive as auxiliary
continuity for other Region-owned systems. The same path accepts a validated recovered handoff.

## Level and process lifecycle

`WorldLifecycleRuntime` constructs the Overworld first and remaining dimensions in source order.
Each dimension's `(0, 0)` control Region and activation generation exclusively own level-global
state. The `P8L1` record durably stores the audited world-border save projection and `no_save` flag;
multi-record restore validates the whole input before changing any level.

Preparation waits until all level work reaches zero, then reactivates tickets and marks levels
ready in dimension order. Shutdown has two ordered stages. The first closes network admission,
saves players and levels, removes players, clears `no_save`, and deactivates closing tickets. The
second requires all work drained, flushes levels, records every per-level close result while
continuing after individual failure, then closes saved data, resources, and the storage lock. Event
capacity is preflighted for each stage.

## Inspection boundary

`world-inspector` accepts a store directory plus world, dimension, Region coordinates, and mapping
version. `RegionFileStore::load_named` validates and reconstructs that identity, after which the
tool materializes the recovery point, decodes both legacy `ferrite:phase8/chunk_v1` and current
`ferrite:world-service/chunk_v1` records, reports `continuity_generation`, recomputes the canonical
state hash, and emits JSON. Mixed or unsupported world-service identity generations fail closed.
Keeping the CLI on `ferrite-persistence` and `ferrite-world` preserves the repository dependency
direction while still exposing the on-disk contract.

This filename is retained because completed Goal 01 ledgers link to it. Active module, type,
diagnostic, inspector, and test-target names are responsibility-owned. Writers use versioned
`ferrite:world-service/*_v1` identities. Store migration appends and commits the current generation
at the next persistence revision, so an interrupted preparation leaves the legacy commit selected.
