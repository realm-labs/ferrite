# ADR-0026: Location-independent durable Region storage

## Status

Accepted

## Context

Simulation ownership is mobile: a Region that unloads, hands off, or loses its worker may next be
activated on a different server. A recovery point stored only on the former worker's filesystem is
therefore not a production durable recovery point. Per-node persistent volumes reduce some restart
risk, but they couple Region placement to a particular node and do not support arbitrary
rescheduling, node loss, or elastic placement.

Lattice owns placement, claims, and activation generations. It does not own Ferrite world bytes,
and direct actor-state transfer cannot be the only recovery path because the source process may be
gone.

## Decision

Distributed production uses a logically dedicated, location-independent durable storage layer.
Compute nodes access it through a backend-neutral Ferrite storage contract; they do not treat their
local filesystem as the sole authority.

The contract has two responsibilities:

- an immutable data plane stores canonical Region snapshots, journal segments, world metadata, and
  checkpoint manifests by stable identity and digest;
- a strongly consistent metadata plane stores the current Region head, writer fence, persistence
  revision, and published world/checkpoint heads using linearizable compare-and-swap.

A Region commit first makes all immutable payloads durable, then advances its head only when the
expected predecessor and activation-generation writer fence still match. Only that successful head
advance returns the durable receipt that can clear dirty state or authorize unload. The storage
layer rejects stale writers even if an obsolete worker can still reach the backend.

Cross-Region or cross-dimension commits publish an immutable manifest containing the exact Region
commit identities and advance a checkpoint head only after every referenced payload is durable.
Recovery reads a published head and its manifest; it never discovers authority by listing files or
choosing the newest timestamp.

`RegionFileStore` remains a valid local-development, test, inspection, and migration adapter. A
node-local digest cache may accelerate reads, but it is disposable and cannot acknowledge
durability. Direct source-to-target streaming is likewise an optional handoff optimization; the
target must be able to recover from the shared durable commit when the source is unavailable.

This ADR fixes the semantic boundary, not a vendor. Goal 07 may implement it as a Ferrite storage
service or as adapters over managed durable blob and strongly consistent metadata systems, provided
the same fencing, atomic publication, integrity, availability, backup, and fault requirements pass.

## Reference development profiles

Ferrite fixes two local profiles so ordinary development and CI do not depend on a production-cloud
decision:

| Profile | Payload data | Commit metadata | Purpose |
|---|---|---|---|
| local single process | `RegionFileStore` | local append-and-repoint index | fast unit, integration, inspection, and migration tests |
| local distributed | MinIO | etcd | multi-process handoff, source-node loss, stale-writer, checkpoint, and storage-fault conformance |

The local distributed profile stores only immutable snapshot, journal, and manifest objects in
MinIO. Region heads, checkpoint heads, persistence revisions, and writer fences remain in a
dedicated etcd namespace; world payloads never enter etcd. Both services use ephemeral,
workspace-owned data in routine CI unless a retained diagnostic run is explicitly requested.

MinIO is the reference development and conformance object store, not the accepted production
backend. `G07-P0-B1` must select and document the formal deployment backend, consistency evidence,
credentials, backup, capacity, and support policy before production claims begin.

## Consequences

- Region placement is independent of the node that last executed or saved it.
- Storage availability and latency become explicit readiness, unload, handoff, backpressure, and
  capacity inputs.
- The data plane may scale independently through immutable objects, while the smaller metadata
  plane pays the cost of strong consistency.
- Garbage collection must trace retained published heads and manifests; object-store listing order
  or eventual consistency cannot decide liveness.
- Disaster recovery, backup, encryption, credentials, quotas, and tenant/world isolation become
  production storage responsibilities rather than node-volume conventions.

## Alternatives Considered

- One persistent volume per worker: rejected because it binds recovery to the former placement and
  fails arbitrary rescheduling or permanent node loss.
- Stream state only during Lattice handoff: rejected because the source may crash before or during
  transfer.
- Put mutable Region files directly in a shared filesystem without a metadata protocol: rejected
  because filesystem visibility alone does not provide writer fencing or atomic checkpoint
  publication.
- Store all world state inside the placement/control plane: rejected because large gameplay blobs,
  retention, backup, and throughput have different scaling and failure requirements from claims.

## Migration or Reversal Plan

Goal 07 introduces the backend-neutral store interface and importer. Existing validated
`RegionFileStore` recovery points are uploaded as immutable objects, verified by canonical digest,
assembled into a checkpoint manifest, and published through a compare-and-swap head only while the
world is stopped or under an explicit migration fence. Local files are retained until the published
checkpoint has been restored on a different node and backup policy permits their removal.
