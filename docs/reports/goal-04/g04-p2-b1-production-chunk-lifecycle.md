# G04-P2-B1 — Production chunk lifecycle

## Outcome

The formal Minecraft gateway now feeds every installed session's bounded player-view and
player-simulation tickets into one production lifecycle driver. Ticket replacement is atomic and
bounded. Each chunk demand is routed through the configured Region mapping to the existing
`WorldServiceRegionRuntime`; no second chunk representation or lifecycle store was introduced.

The driver admits bounded generation requests and publishes only results that match Region
activation generation, request ID, input revision, sequential target status, and content manifest.
P2-B1 advances lifecycle status without inventing terrain. P2-B2 owns the deterministic terrain and
biome transformations that replace the current pass-through generation worker.

## Commit and unload boundary

Current continuity intentionally rejects generation in flight. Until P2-B3 adds a versioned
continuation format, the worker request and its fenced result therefore complete outside Region
authority but before the composite continuity commit. A stale result fails closed and cannot make a
chunk visible or durable.

The strongest active ticket derives dormant, accessible, block-ticking, or entity-ticking state.
When all tickets disappear, active chunks demote and receive a monotonic unload token only after
generation has finished. The chunk remains authoritative and resident while that token is pending.

Formal persistence now returns the exact recovery point and `CommitReceipt` for every committed
Region. The composite runtime verifies the point's Region, activation generation, Region size,
content manifest, canonical state hash, and complete current world-service record set before
applying the receipt. Revision, tick, and digest must then match. Only identity-matched pending
unloads are removed; a new ticket cancels the token and makes an older receipt harmless.

## Bounds and failure policy

- total tickets are bounded by the configured maximum sessions and per-session chunk contract;
- generation in flight, accepted results, lifecycle actions, and event draining have explicit
  per-tick limits;
- formal world event capacity covers the bounded worst-case promotion and save/unload fanout;
- ticket overload preserves the prior admitted set, while unknown ownership, stale generation,
  mismatched content, invalid sequence, or mismatched save receipt fails closed.

## Verification

- Formal lifecycle unit tests prove atomic ticket overload, request/result fencing before commit,
  stale-generation rejection, ticket-loss ordering, and unload only after a matching file-store
  receipt.
- `cargo test -p ferrite-server-runtime --all-features` passes the lifecycle tests plus formal
  network entry, autosave, exact-tick restart, corruption, Region service, and replay regressions.
- `cargo clippy -p ferrite-server-runtime --all-targets --all-features -- -D warnings` passes.
- Universal Rust, source-policy, production-manifest, and diff gates run before commit.
