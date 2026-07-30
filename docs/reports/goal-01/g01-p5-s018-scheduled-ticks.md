# G01-P5-S018 — Scheduled Tick Runtime

## Result

Complete. The source-known portion of the `SourceInconclusive`
`SIM-SCHEDULED-TICKS-001` slice now maps to modular production records, per-chunk containers,
level-wide collection, persistence operations, and committed behavioral tests. The sole
source-inconclusive restored equal-head ordering remains owned by `EXP-SIM-002`.

## Evidence

Production owners:

- `ferrite-simulation::scheduled_tick::record`;
- `ferrite-simulation::scheduled_tick::container`;
- `ferrite-simulation::scheduled_tick::level`.

Committed test owner:

- `crates/ferrite-simulation/tests/slices/sim_003.rs`.

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
59 passed; 0 failed
1 SourceInconclusive source-known surface verified
EXP-SIM-002 remains DeferredExperiment
```

## Ownership notes

This batch fixes signed creation and sub-order wrapping, seven priority values, identity/position
deduplication, unloaded-request refusal, per-chunk trigger-first ordering, cross-chunk
priority/sub-order merging, pre-callback FIFO collection, query timing, callback rescheduling,
independent block/fluid caps, activity and current-type gates, saved delay narrowing and reload
rebasing, inclusive clear, copy history and sub-order translation, and retained backlog.

Ferrite deliberately orders comparator-equal restored chunk heads by chunk coordinate for
topology-independent replay. That fallback is not asserted as vanilla behavior. The deferred
observation stays open under `EXP-SIM-002` with the policy “do not claim a vanilla cross-chunk
restored-tick tie-break.”

Region generation fencing, cross-Region semantic delivery, durable handoff transactions, client
projection, and subsystem continuity remain with `G01-P5-B1`. `G01-P5-S019` next implements the
source-specified random-activity surface.
