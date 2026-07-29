# ADR-0008: Journaled Persistence and Committed-Tick Recovery

## Status

Accepted

## Context

Region ownership may move or fail while chunks, entities, scheduled work, random streams, and
boundary protocols are changing. Lattice placement handoff does not transfer Ferrite's in-memory
gameplay state.

## Decision

Ferrite owns a versioned persistence format with append-and-repoint Region data and a write-ahead
journal. A transaction appends and fsyncs its intent, appends and fsyncs data, updates and fsyncs
the index, then marks the journal transaction committed. Fsync batching is configurable but cannot
change commit ordering.

The recovery and handoff unit is `RegionCommitSnapshot`, captured only after required boundary work
for a tick has reconciled. It contains:

- Region identity and activation generation;
- last committed logical tick and persistence revision;
- immutable chunk/entity records;
- scheduled work and named random-stream state;
- applied boundary sequences and other idempotency state.

A crash recovers the latest durable committed Region tick plus a validated journal tail. Work after
that recovery point is not claimed committed. The new activation must install a strictly newer
generation before admission; stale generations cannot commit. A graceful handoff fails closed if
the snapshot/journal tail cannot be made durable.

## Consequences

- Ferrite can state an exact recovery point rather than promising preservation of volatile memory.
- Snapshot serialization/compression may run asynchronously against immutable revisions.
- Dirty flags clear only when captured Region and child revisions are still current.
- Storage cost includes journal and retained generations needed for recovery.

## Alternatives Considered

- Serialize live ECS state: rejected because archetype/runtime layout is not a stable schema.
- Rely on Lattice actor movement: rejected because it does not own Ferrite state transfer.
- Best-effort periodic saves without a journal: rejected because crash outcome and authority would
  be ambiguous.

## Migration or Reversal Plan

Record schemas use explicit versions and pure migrations. A different recovery point or storage
engine requires fault tests for crash, torn write, corruption, handoff, and stale-owner fencing.
