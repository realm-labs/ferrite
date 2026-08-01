# G04-P1-B3 — Formal world persistence and recovery

## Outcome

The formal gateway now consumes each Region's post-commit composite continuity, retains one bounded
latest capture, autosaves at the configured interval, and performs a final capture and flush before
releasing Region authorities. The captures include simulation queues and random state, player and
entity service records, world chunks, overworld level state, and the `world_v1` metadata record.

`CompositeProductionRegionRuntime::restore` reconstructs those service owners at a newer activation
generation. The metadata-only revision created by P1-B2 remains a valid bootstrap boundary; any
partial service record set fails because it cannot decode the mandatory simulation runtime record.
World-service restoration filters cross-service records back to their owning runtimes instead of
recapturing them as duplicate auxiliary data.

## Checkpoint and fault policy

Region stores commit in stable order with the overworld control Region last. Its latest committed
tick publishes the complete world checkpoint. `RegionFileStore::load_at_or_before` selects the
latest complete per-Region prefix at that tick while still decoding and validating all later frames.
This distinguishes a valid successor written before a crash from the published world state.

Startup requires every Region that existed at a nonzero checkpoint to have that exact prefix. A
valid unpublished successor may be at most one autosave interval ahead. The runtime restores the
published prefix and advances through bounded no-input ticks to the highest durable successor so a
new branch never regresses a store's tick. Missing, mismatched, corrupt, over-far-ahead, or
wrong-manifest state fails before readiness.

The current save worker is synchronous and therefore has at most one pending write; this is within
every valid configured pending-save bound. Every save is a full snapshot, which is stricter than the
configured maximum checkpoint cadence. P2-B1 will connect dirty chunk tickets, asynchronous save
admission, receipts, and unload decisions to this sink.

## Shutdown and inspection

Drain closes admission, transitions the world lifecycle, refreshes metadata and level auxiliary
records, forces one final composite capture if necessary, reports pending durable work through
`NodeLifecycle`, flushes all Region stores, publishes the control checkpoint, closes level state,
and only then sets active Region authority to zero.

`world-inspector` now recognizes `ferrite:world-service/world_v1` as current continuity alongside
level and chunk records. It continues to report canonical hash, activation generation, committed
tick, persistence revision, content manifest, auxiliary count, and decoded chunks without starting
the server.

## Verification

- `cargo test -p ferrite-persistence --all-features`: bounded checkpoint selection plus existing
  torn-tail, uncommitted-index, checksum, revision, and tick fault tests pass.
- `cargo test -p ferrite-server-runtime --all-features`: formal two-tick autosave, clean shutdown,
  exact-tick restart, corrupt-store rejection, unpublished-prefix recovery, and all existing Region
  service tests pass.
- `cargo test -p world-inspector --all-features`: current metadata and legacy/current generation
  classification pass.
- Universal Rust, source-policy, production-manifest, and diff gates run before commit.
