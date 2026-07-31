# G01-P8-B1 — Durable-world Region integration

## Result

Complete. Phase 8 generation now has a bounded, generation-fenced Region owner; exact durable
chunk, lifecycle, and level-global records; actual `RegionFileStore` commit receipts before
teardown; recovery and handoff admission; ordered world bootstrap and shutdown; and an independent
offline inspection CLI.

## Evidence

Production owners:

- `ferrite-world::durable` for the bounded `FWC1` chunk representation;
- `ferrite-server-runtime::phase8` for generation, activity, unload, recovery, handoff, level, and
  shutdown coordination;
- `ferrite-persistence::RegionFileStore::load_named` for validated external lookup;
- `world-inspector` for recovery-point JSON inspection.

Committed tests:

- `crates/ferrite-server-runtime/tests/phase8_durable_world.rs`;
- `crates/ferrite-persistence::store` unit tests;
- `apps/world-inspector/tests/usage.rs`.

Design contract:

- [Phase 8 durable-world integration](../../development/phase8-durable-world-integration.md).

Validated commands:

```text
cargo test -p ferrite-world --all-features
cargo test -p ferrite-persistence --all-features
cargo test -p ferrite-server-runtime --all-features
cargo test -p world-inspector --all-features
cargo ferrite task check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
git diff --check
```

Focused result before repository gates:

```text
8 Phase 8 durable-world integration tests passed; 0 failed
exact chunk round trip and malformed continuity rejection passed
four-part asynchronous generation fencing and adjacent publication passed
activity ordering, atomic bounded events, unload cancellation, and receipt-fenced teardown passed
store recovery, journal overlay, handoff, auxiliary continuity, and inspector reports passed
dimension bootstrap, control-Region ownership, level records, and ordered shutdown passed
```

## Boundary disposition

Runtime numeric block-state and biome IDs are deliberately preserved byte-for-byte, but only
inside a recovery point carrying the exact locked content manifest. A manifest mismatch fails
restore; these values are not presented as globally stable persistent identities.

This batch closes the integration boundary but does not claim behavior-family conformance.
Cross-topology deterministic generation, save/load/crash matrices, structural invariants, boundary
behavior, and final surface/join dispositions remain `G01-P8-B2`. Same-seed vanilla identity
remains governed by the existing explicit deferred experiments and `G01-P8-B3`.
