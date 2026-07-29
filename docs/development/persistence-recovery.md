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

## Append-and-repoint store

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

The store API assumes one generation-fenced writer for a Region. The Lattice adapter batch must
enforce that writer lease; independent store instances racing without placement fencing are outside
this local storage boundary and must fail closed if they create conflicting transaction metadata.

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

The recovered value deliberately remains a stable semantic recovery point. Subsystem codecs
materialize voxel, ECS, scheduled-work, and RNG state only after all validation succeeds.
`G01-P2-B5` connects this boundary to the pinned Lattice placement and lease adapter.
