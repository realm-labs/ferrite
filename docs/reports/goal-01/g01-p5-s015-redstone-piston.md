# G01-P5-S015 — Redstone Piston

## Result

Complete. The `SourceSpecified` `RED-PISTON-001` slice now maps to modular production semantics and
committed behavioral tests. `EXP-RED-003` remains a conformance trace and owns no unresolved
implementation behavior.

## Evidence

Production owners:

- `ferrite-gameplay::redstone::piston::power`;
- `ferrite-gameplay::redstone::piston::resolver`;
- `ferrite-gameplay::redstone::piston::execution`;
- `ferrite-gameplay::redstone::piston::moving`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/redstone/red_004.rs`.

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
239 passed; 0 failed
1 SourceSpecified slice verified
```

## Ownership notes

This batch fixes piston quasi-connectivity sampling, event 0/1/2 admission and revalidation,
pushability and sticky structure resolution, reverse mutation/notification orders, sticky fast
retraction, moving-state lifecycle, entity collision displacement, slime velocity, honey carrying,
base ejection, and moving-block hooks.

Region mutation, block-event scheduling, chunk/boundary preflight, concrete collision geometry,
entity storage, deterministic RNG streams, sounds/particles, and packet projection remain with
their generated owners. Explosion behavior remains in `G01-P5-S016`.
