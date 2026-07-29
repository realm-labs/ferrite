# G01-P4-B1 Chunk Join Projection Report

## Result

`G01-P4-B1` implements the bounded, deterministic path from an admitted Play session to Java 26.2
terrain packets. The implementation covers join and respawn state, chunk tickets and interest,
minimal terrain snapshots, full sections, biomes, three client heightmaps, block entities, light,
unload rules, batch feedback, and connection-local outbound delivery.

The design contract is recorded in
[Chunk Join and Terrain Projection](../../development/chunk-join-projection.md).

## Source-locked behaviors

The batch preserves the reviewed Java 26.2 boundaries:

- requested view distance clamps to `2..=32` and to the configured server maximum;
- center changes precede view-difference processing;
- unsent pending chunks leave without an unload packet, while sent live chunks receive one;
- batches contain nearest ready full chunks between exact start and finish markers;
- the initial sender target is 9 chunks per tick with one unacknowledged batch;
- feedback treats NaN as `0.01`, otherwise clamps to `0.01..=64`, and opens at most 10 batches;
- full chunks carry the dimension's exact section count and light uses two boundary layers;
- client-used heightmaps map to raw IDs 1, 4, and 5;
- respawn retention bits `0x01` and `0x02` are independent and high bits are ignored semantically.

## Automated evidence

Focused suites cover:

- deterministic minimal snapshots at negative coordinates, heightmap construction, exact light
  counts, and sparse-section materialization;
- ticket thresholds, expiration, atomic source replacement, view clamps, bounds, stable nearest
  ordering, pending/sent transitions, unload filtering, and recenter order;
- batch quota, acknowledgement, NaN and finite feedback, in-flight bounds, and unavailable
  snapshots;
- generation-fenced batch preparation, retry after projection failure, and stale-commit refusal;
- runtime-to-Java registry projection and absent/out-of-range mappings;
- canonical block and biome palette round trips, exact control-packet bytes, full chunk/light
  payloads, malformed and truncated data, cache storage behavior, biome refresh, unload, and
  respawn masks;
- Play connection enqueue after the half-duplex Configuration-to-Play transition.

The acceptance commands are:

```text
cargo test -p ferrite-world
cargo test -p ferrite-server-runtime
cargo test -p ferrite-protocol
cargo ferrite task check
git diff --check
```

`cargo ferrite task check` remains the universal batch gate and includes format, Clippy with
warnings denied, all workspace tests, offline reference verification, implementation-manifest
checks, source policy, dependency direction, and repository policy.

## Scope boundary

This evidence proves the reusable chunk join projection, not the complete Phase 4 playable path.
Movement, collision and Region transfer, block interaction and correction, topology trace
equivalence, and the unmodified-client C2 smoke remain in `G01-P4-B2` through `G01-P4-B5`.
