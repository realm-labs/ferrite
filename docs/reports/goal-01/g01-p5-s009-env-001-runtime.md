# G01-P5-S009 — ENV-001 Fluid and Geyser Runtime

## Result

Complete. Both `SourceSpecified` environment slices primarily owned by `ENV-001` now map to modular
production semantics and committed behavioral tests.

## Evidence

Production owners:

- `ferrite-gameplay::environment::fluid`;
- `ferrite-gameplay::environment::geyser`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/environment/env_001.rs`.

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
151 passed; 0 failed
2 SourceSpecified slices verified
```

## Ownership notes

This batch fixes environment-owned fluid/geyser state identities, constants, ordered scans,
transition plans, scheduling, RNG boundaries, transient clocks and persistence inputs. Region
state, ECS entities, tick queues, collision queries, content snapshots and projection remain their
existing authorities. Generic effect merging, entity synchronization, bucket inventory/stat
consequences and client particle-engine policy remain with their generated shared owners.
