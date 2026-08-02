# Persistence and Region Recovery

`G01-P2-B4` defines Ferrite's durable Region recovery point. Lattice placement state, Bevy layouts,
Rust memory layout, runtime registry IDs, packets, and compressed storage blocks are not persistence
schemas.

## Recovery point

A `RegionCommitSnapshot` records:

- `SimulationRegionKey`, activation generation, committed logical tick, and persistence revision;
- Region side length, content-manifest digest, and canonical state hash;
- bounded stable records for chunks, entities, scheduled work, named random streams, applied
  boundary sequences, and versioned extensions.

Each record uses a persistent resource domain plus bounded canonical key/value bytes. Construction
sorts by kind, domain, and key and rejects duplicate identities. Runtime IDs and Bevy entities must
be lowered into these records by their owning subsystem codecs.

A `RegionRecoveryPoint` combines the base snapshot with zero or more strictly contiguous committed
journal-tail frames. Its committed tick is the last tail tick, or the snapshot tick when the tail is
empty. Magic, schema version, fixed-width fields, minimal length encodings, bounds, semantic
identities, complete consumption, and a locked BLAKE3 digest are validated on read.

## Production storage boundary

The recovery point is topology-independent, but durability is not achieved merely by encoding it.
In distributed production, every eligible Region worker must be able to load the same committed
point after the former owner and its local disk are unavailable. The production `RegionDurableStore`
therefore exposes stable logical identities rather than filesystem paths:

```text
WorldId / DimensionId / SimulationRegionKey / mapping-version
  -> immutable snapshot and journal objects
  -> fenced Region commit head
  -> optional published cross-Region/world checkpoint manifest
```

Immutable objects are addressed and verified by digest. The metadata plane advances a Region head
with a linearizable compare-and-swap over its expected predecessor, persistence revision, and an
opaque monotonically ordered writer fence derived from activation authority. A stale generation is
rejected by storage even if its former process can still reach the backend. Only a successful head
advance yields the durable receipt consumed by `DirtyTracker`, unload, or handoff.

For an atomic cross-Region outcome, every Region payload is first made durable and an immutable
manifest records the exact commit identities. The checkpoint publisher then advances one manifest
head by compare-and-swap. Recovery follows that published head; it does not infer a valid prefix
from timestamps, directory enumeration, or the newest independently visible object.

The storage contract is intentionally backend-neutral. A deployment may use a Ferrite storage
service or managed blob/metadata systems, but it must prove integrity, storage-side fencing,
read-after-commit recovery from a different worker, bounded retries, backup/restore, and corruption
isolation. A local cache is digest-verified and disposable and never returns a durability receipt.

The Goal 07 local multi-process and CI reference backend is MinIO plus etcd: MinIO contains only
immutable recovery payloads and manifests, while a dedicated etcd namespace contains the fenced
Region and checkpoint heads. It exists to make the full backend contract reproducible on a
developer machine; production backend selection and acceptance remain separate.

## Local append-and-repoint adapter

`RegionFileStore` uses three append-only framed logs:

- `region-journal.log` for transaction intents and commit markers;
- `region-data.log` for encoded recovery points;
- `region-index.log` for Region-to-data repoints.

A durable commit performs this order:

1. append the transaction intent and sync the journal;
2. append the recovery point and sync data;
3. append the index repoint and sync the index;
4. append and sync the transaction commit marker.

Every frame carries a declared length and BLAKE3 checksum. The initial persistence revision must be
one; later revisions must advance exactly by one. Committed ticks and activation generations cannot
regress.

Recovery selects only index records whose matching intent and commit marker are both complete and
whose transaction metadata, frame location, checksum, recovery-point digest, Region identity, and
revision agree. A complete checksum or semantic corruption is rejected. An incomplete trailing frame
is ignored during reads and truncated to the last verified frame before the next write. An index
written without its final commit marker cannot advance the recovery point.

This adapter assumes one generation-fenced writer for a Region. The Lattice adapter enforces that
writer lease, and conflicting local instances fail closed when transaction metadata disagrees.
Those rules are sufficient for local development, codec testing, offline inspection, and importing
existing stores. They do not make files on one compute node a distributed production authority.

## Dirty acknowledgement

`DirtyTracker` gives asynchronous capture an exact revision token. Each real mutation advances the
revision. Persistence acknowledgement clears dirty state only when the captured revision still
equals the live revision; a stale capture leaves the object dirty. Subsystem snapshot builders apply
this primitive to Region and child revisions rather than clearing flags merely because background
serialization finished.

## Handoff and activation

`RegionHandoffState` packages a validated recovery point, its digest, and the requested target
activation generation. Preparation rejects a target generation that is not strictly newer than the
snapshot generation. Installation rechecks the Region identity and digest before returning a
`RecoveredRegion`; stale owners therefore cannot resume admission from the handed-off state.

In distributed production, the handoff record carries the published Region commit identity. Direct
source-to-target bytes may reduce latency, but target activation must validate that identity against
the durable store and must also succeed after source loss. The target never depends on mounting or
reattaching the source worker's local filesystem.

The recovered value deliberately remains a stable semantic recovery point. Subsystem codecs
materialize voxel, ECS, scheduled-work, and RNG state only after all validation succeeds.
`G01-P2-B5` connects this boundary to the pinned Lattice placement and lease adapter.
