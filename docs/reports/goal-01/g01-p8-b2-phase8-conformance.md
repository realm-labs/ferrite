# G01-P8-B2 — Phase 8 conformance

## Result

Complete. Every source-specified world slice and locked worldgen data family remains verified, and
Ferrite's deterministic dispatch, structural status/activity invariants, Region boundaries,
save/load, crash recovery, three Phase 8 root surfaces, and 12 assigned cross-system joins now have
executable conformance evidence.

## Evidence

Primary suites:

- `crates/ferrite-world/tests/slices.rs`: 399 world behavior/data tests;
- `apps/behavior-runner/tests/phase8_conformance.rs`: aggregate catalog, generation, boundary,
  save/load, and crash suite;
- `apps/behavior-runner/tests/surfaces/{content_dispatch,persistence_reload,world_lifecycle}.rs`;
- the 12 Phase 8 owners in `apps/behavior-runner/tests/joins`.

Design contract:

- [Phase 8 conformance](../../development/phase8-conformance.md);
- [durable-world Region integration](../../development/phase8-durable-world-integration.md).

Validated commands:

```text
cargo test -p behavior-runner --test phase8_conformance --test surfaces --test joins --all-features --no-fail-fast
cargo test -p ferrite-world --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
cargo ferrite task check
git diff --check
```

Focused result before repository gates:

```text
963 worldgen records across 14 runtime families validated
399 source-specific world slice tests retained
64 deterministic four-chunk dispatch cases across all 12 statuses passed
8 Region-boundary, 16 save/load, and 6 crash/fence cases passed
3 Phase 8 root surfaces and 12 Phase 8 cross-system joins passed
```

## Boundary disposition

This batch verifies Ferrite's project-owned deterministic state under scheduling-order changes and
recovery. It does not claim that a vanilla seed and a Ferrite seed produce block-identical terrain.
The source-inconclusive WGEN pipeline observation and its three named experiments remain explicitly
deferred. `G01-P8-B3` records that non-identical-seed equivalence contract and completes Phase 8
without converting statistical uncertainty into an implementation claim.
