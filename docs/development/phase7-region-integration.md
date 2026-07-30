# Phase 7 Region Integration

`G01-P7-B1` integrates the audited entity and mob runtimes with Region authority, lifecycle
continuity, cross-Region transfer, tracking, and bounded observer fan-out.

## Ownership boundary

`ferrite-server-runtime::phase7` separates four responsibilities:

- `model` defines bounded canonical entity payloads, persistent lifecycle state, command and
  transfer inputs, and semantic observer projections;
- `continuity` owns the versioned entity and applied-transfer record formats;
- `transfer` defines target-side replay keys and source commit receipts;
- `runtime` owns one Region generation, the stable entity set, observer queues, transfer receipts,
  and every state transition described below.

Gameplay algorithms remain in `ferrite-gameplay`, packet codecs remain in `ferrite-protocol`, and
the general transport envelope remains in `ferrite-region-runtime`. Adapters lower their results to
these project-owned semantic values; only the authoritative `Phase7RegionRuntime` may commit them.

## Command and lifecycle fencing

Every entity command carries the exact Region key, activation generation, stable entity ID,
expected revision, and contiguous command sequence. The runtime rejects a wrong Region, stale
generation, missing entity, sequence gap, or revision mismatch before changing state or publishing
an event. A sequence at or below the last committed sequence is idempotently reported as already
applied.

An entity is `Active`, `Inactive`, or `OutboundPending`. Activation emits a spawn projection;
deactivation and despawn emit removal projections; active mutations emit updates. Inactive entities
remain authoritative and durable without being tracked. Pending entities remain durable at the
source until transfer acknowledgement, but are no longer visible to source observers.

Entity payloads retain the canonical project-owned bytes and a BLAKE3 digest. Each payload is
limited to 1 MiB. Entity, observer, per-observer projection, and applied-transfer receipt counts all
have explicit nonzero limits.

## Tracking and atomic fan-out

Observers are keyed by stable entity ID and have independent FIFO projection queues. Joining an
observer snapshots every active entity in stable entity-ID order. A join that cannot fit the entire
snapshot is rejected without installing the observer.

Before any tracked mutation commits, the runtime reserves one monotonically ordered projection for
every observer. If any queue is full or the sequence range cannot be represented, no entity state,
revision, command sequence, lifecycle, or observer queue changes. This makes a logical publication
atomic across the Region rather than allowing different clients to observe different committed
states.

## Two-phase cross-Region transfer

A transfer follows an explicit prepare, accept, and commit protocol:

1. The source validates both endpoint generations, the target chunk owner, entity revision, command
   sequence, and lifecycle. It stores an `OutboundPending` record and emits a source removal.
2. The target validates its current generation, envelope role, capacity, entity kind, decoded active
   state, and target chunk ownership. It atomically installs the entity, records a durable replay
   key, and emits a target spawn.
3. The target returns an `EntityTransferReceipt`. Only a byte-for-byte matching receipt lets the
   source delete its pending entity.

Retrying the same pending request rebuilds the same transfer. Re-delivery at the target returns
`AlreadyApplied` from the durable replay key and does not publish another spawn. A rejection leaves
the source pending and retryable. An explicit abort restores the source entity to `Active` and
publishes a new spawn; it does not roll back the already consumed command sequence or revision.
Applied target receipts are retained until a checkpoint policy explicitly prunes a completed tick
range.

## Save and restore

Snapshots emit entity records in stable entity-ID order followed by ordered applied-transfer
receipts. The `ferrite:phase7/entity_v1` record preserves kind, source chunk, revision, last command
sequence, canonical payload, and the complete active, inactive, or outbound-pending lifecycle.
Pending records include the target Region and generation, transfer tick and sequence, candidate
chunk, candidate revision, and candidate payload.

Restore installs the caller-supplied activation generation, validates every record and capacity,
rejects duplicate IDs or receipts, and verifies both source and pending-target chunk ownership.
Malformed identities, zero generations, invalid lifecycle tags, oversized payloads, truncation, and
trailing bytes fail closed. Restored pending transfers can therefore resume without making a
second gameplay decision, while restored target receipts continue to suppress duplicate delivery.

## Validation

`crates/ferrite-server-runtime/tests/phase7_region_integration.rs` verifies:

- Region, generation, revision, and sequence fencing before mutation;
- activation, deactivation, despawn, and exact projection reasons;
- atomic all-observer backpressure and stable bounded observer joins;
- two-phase transfer ordering, stale-generation rejection, retry, abort, commit, and idempotence;
- active, inactive, pending, and applied-receipt save/reload continuity;
- stable snapshot ordering and rejection of corruption, wrong ownership, and oversized payloads.
