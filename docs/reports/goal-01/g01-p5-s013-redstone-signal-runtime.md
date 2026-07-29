# G01-P5-S013 — Redstone Signal Runtime

## Result

Complete. The three `SourceSpecified` slices owned primarily by `RED-001` now map to modular
production semantics and committed behavioral tests:

- `RED-SIGNAL-UPDATE-001`;
- `RED-COMPARATOR-RUNTIME-001`;
- `RED-DAYLIGHT-DETECTOR-RUNTIME-001`.

The associated experiments are conformance traces and own no unresolved implementation behavior.

## Evidence

Production owners:

- `ferrite-gameplay::redstone::signal`;
- `ferrite-gameplay::redstone::comparator`;
- `ferrite-gameplay::redstone::daylight_detector`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/redstone/red_001.rs`.

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
211 passed; 0 failed
3 SourceSpecified slices verified
```

## Ownership notes

This batch fixes directional signal aggregation, ordinary and direct conductor behavior, default
and experimental dust selection, default dust routing and guarded writes, connection shapes and
lifecycle ordering, complete comparator input/cache/schedule/refresh/use semantics, and complete
daylight ticker/formula/use semantics.

Region state, scheduling, boundary delivery, deterministic RNG streams, state mutation, neighbor
dispatch, event delivery, and sound projection remain with their generated owners. Delay
components, piston behavior, and explosion behavior remain in `G01-P5-S014` through
`G01-P5-S016`.
