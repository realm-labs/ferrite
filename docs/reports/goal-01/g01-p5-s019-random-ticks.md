# G01-P5-S019 — Random Tick Runtime

## Result

Complete. The `SourceSpecified` `SIM-RANDOM-ACTIVITY-001` slice now maps to modular production
ticket, activity-order, position-stream, and chunk-sampling semantics with committed behavioral
tests. `EXP-SIM-003` remains a conformance regression vector and owns no unresolved implementation
behavior.

## Evidence

Production owners:

- `ferrite-simulation::random_tick::ticket`;
- `ferrite-simulation::random_tick::tracker`;
- `ferrite-simulation::random_tick::position`;
- `ferrite-simulation::random_tick::activity`.

Committed test owner:

- `crates/ferrite-simulation/tests/slices/sim_004.rs`.

Validated commands:

```text
cargo test -p ferrite-simulation --all-features
cargo clippy -p ferrite-simulation --all-targets --all-features -- -D warnings
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
72 passed; 0 failed
1 SourceSpecified slice verified
```

## Ownership notes

This batch fixes normal/debug/freeze phase gates, all nine ticket mappings, simulation-only lowest
level, strict-negative timeout expiry, holder-save and unload expiry behavior, the level 31 random
activity boundary, visible/ticking holder gates, fastutil 8.5.18 map history and iterator order,
signed position generation, precipitation draw consumption, bottom-to-top section snapshots,
block-before-fluid dispatch from one captured state, shared callback RNG, signed-short eligibility
counts, and no catch-up or speed clamp.

The compatibility tracker is a graph-update sink; `G01-P5-B1` owns binding ticket listeners and
distance propagation to Region activation, holder and persistence state. It also owns the ordered
per-level consistency island required by the shared RNG, durable runtime-stream handoff,
cross-Region callback transactions, and client projection. All Phase 5 slice batches are now
implemented, so integration is the next batch.
