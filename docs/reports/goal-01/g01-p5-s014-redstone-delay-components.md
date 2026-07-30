# G01-P5-S014 — Redstone Delay Components

## Result

Complete. The `SourceSpecified` `RED-DELAY-COMPONENTS-001` slice now maps to modular production
semantics and committed behavioral tests. `EXP-RED-002` remains a conformance trace and owns no
unresolved implementation behavior.

## Evidence

Production owners:

- `ferrite-gameplay::redstone::delay::diode`;
- `ferrite-gameplay::redstone::delay::repeater`;
- `ferrite-gameplay::redstone::delay::observer`;
- `ferrite-gameplay::redstone::delay::torch`;
- `ferrite-gameplay::redstone::delay::orientation`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/redstone/red_003.rs`.

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
224 passed; 0 failed
1 SourceSpecified slice verified
```

## Ownership notes

This batch fixes the shared diode input/schedule/due-tick transaction, repeater delay/lock/use
behavior, observer two-tick pulse and replacement/removal edges, floor/wall torch input and signal
faces, level-owned torch burnout history, and experimental neighbor-orientation draws.

Region state, authoritative scheduler integration, deterministic RNG streams, boundary delivery,
mutation acceptance, sounds, particles, and network projection remain with their generated owners.
Piston and explosion behavior remain in `G01-P5-S015` and `G01-P5-S016`.
