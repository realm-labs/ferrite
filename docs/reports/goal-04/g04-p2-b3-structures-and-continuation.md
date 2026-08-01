# G04-P2-B3 — Structures and generation continuation

## Outcome

`ferrite:overworld_v1` now creates deterministic, sparse `ferrite:waystone_ruin` starts, derives
every intersecting chunk reference without consulting mutable neighbor state, and places the
referenced structure fragment during `FEATURES`. Starts and references are part of the authoritative
`ChunkColumn`, so structure discovery, block placement, persistence, and later client projection do
not use a parallel catalog.

The placement grid has a fixed eight-chunk spacing with a seed-derived offset per cell. A start's
bounded placement crosses its positive X/Z chunk edges. This makes reference state and split
placement observable while keeping candidate discovery and per-stage writes bounded. As with the
rest of Ferrite generation, this is deterministic project-owned behavior within the Goal 01
equivalence boundary; it is not a claim of Mojang same-seed structure identity.

## Versioned durability and restart

The current chunk payload is `FWC2`. It appends bounded, sorted, deduplicated version-1 structure
starts and references to the existing voxel and block-entity state. `FWC1` remains readable and
migrates to an empty structure state; new writes always use `FWC2`. Invalid versions, inverted
bounds, foreign starts, non-intersecting references, excess counts, and oversized payloads fail
closed.

The current lifecycle wrapper is `P8C2`. A pending generation continuation records:

- continuation version;
- monotonic request ID and exact source revision;
- the one sequential target status;
- the locked content-manifest digest.

Composite continuity may now commit that marker. Restore rejects unknown versions, mixed content,
revision drift, non-sequential targets, or simultaneous generation/unload work. A valid marker is
reissued with the same request identity and source column under the new Region activation. The
request sequence resumes above every recovered identifier, and the ordinary result fence still
prevents stale publication. `P8C1` remains readable without inventing a continuation.

## Verification

- Overworld tests prove deterministic start selection, cross-chunk references, and placed blocks.
- Durable chunk tests prove `FWC2` structure round trip and `FWC1` migration.
- World-service continuity tests prove `P8C2` continuation round trip and `P8C1` migration.
- A runtime restart test commits generation in flight, restores under a new activation, reissues the
same request, and proves later request IDs cannot collide.
- Offline inspection reports structure start/reference counts and the complete pending continuation
  identity without interpreting it as completed generation.
- Formal lifecycle tests still drive the complete status pyramid and retain request/result fencing,
  bounded ticket, save-receipt, and unload coverage.
- Universal Rust, source-policy, production-manifest, and diff gates run before commit.
