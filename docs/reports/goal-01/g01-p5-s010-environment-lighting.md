# G01-P5-S010 — Environment Lighting Runtime

## Result

Complete for the source-specified portion. `ENV-LIGHTING-001` now maps to production semantics and
committed behavioral tests, while its one unresolved end-to-end latency observation remains
`DeferredExperiment` under `EXP-ENV-004`.

## Evidence

Production owner:

- `ferrite-gameplay::environment::lighting`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/environment/env_003.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
167 passed; 0 failed
1 SourceInconclusive source-known implementation verified
EXP-ENV-004 remains DeferredExperiment
```

## Ownership notes

This batch fixes lighting channels, propagation, queue ordering, section lifecycle, visible-map
publication, brightness, emitter evaluation, packet admission and client import ordering. Region
light maps, chunk tasks, networking and rendering remain their existing authorities. No finite
mutation-to-render deadline is asserted; only a profile-scoped committed experiment may replace
that deferred observation.
