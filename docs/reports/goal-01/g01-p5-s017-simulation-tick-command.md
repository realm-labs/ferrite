# G01-P5-S017 — Simulation Tick and Command Limits

## Result

Complete. The two `SourceSpecified` slices `SIM-TICK-PIPELINE-001` and
`SIM-COMMAND-LIMIT-001` now map to modular production semantics and committed behavioral tests.
`EXP-SIM-001`, `EXP-SIM-004`, `EXP-SIM-005`, and `EXP-SIM-006` remain conformance traces and own no
unresolved implementation behavior.

## Evidence

Production owners:

- `ferrite-simulation::server_tick::pacing`;
- `ferrite-simulation::server_tick::rate`;
- `ferrite-simulation::server_tick::pause`;
- `ferrite-simulation::server_tick::phases`;
- `ferrite-simulation::command_limit::context`;
- `ferrite-simulation::command_limit::redirect`;
- `ferrite-simulation::command_limit::chain`.

Committed test owner:

- `crates/ferrite-simulation/tests/slices/sim_001.rs`.

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
45 passed; 0 failed
2 SourceSpecified slices verified
```

## Ownership notes

This batch fixes loop deadlines and overload dropping, rate/freeze/step/sprint state, dedicated and
integrated pause, serial server/level phase order, level time/activity/entity gates, autosave and
tick-time arithmetic, outer/nested command snapshots, front-spliced action queues, all automatic
cost sites, strict fork cardinality/error routing, defensive overflow, and independent
command-block chain traversal.

Concrete server callbacks, level and entity storage, packets, profiler/log sinks, command parsing
and Brigadier execution, Region scheduling, generation fencing, persistence, and projection remain
with their generated owners. `G01-P5-S018` next implements the source-known scheduled-tick surface
while retaining its one deferred cross-chunk restore observation.
