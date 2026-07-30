# G01-P5-S016 — Redstone Explosion

## Result

Complete. The `SourceSpecified` `RED-EXPLOSION-001` slice now maps to modular production semantics
and committed behavioral tests. `EXP-RED-004` remains a conformance trace and owns no unresolved
implementation behavior.

## Evidence

Production owners:

- `ferrite-gameplay::redstone::explosion::ray`;
- `ferrite-gameplay::redstone::explosion::entity`;
- `ferrite-gameplay::redstone::explosion::block`;
- `ferrite-gameplay::redstone::explosion::fire`;
- `ferrite-gameplay::redstone::explosion::transaction`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/redstone/red_006.rs`.

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
256 passed; 0 failed
1 SourceSpecified slice verified
```

## Ownership notes

This batch fixes 1,352-ray sampling and float/double arithmetic, world-bound and resistance hooks,
entity query/exposure/damage/knockback routing, interaction and gamerule gates, exact Fisher–Yates
draws, insertion-ordered drop collectors with cap 16, resulting-state fire admission, and complete
phase/result order.

Region storage, entity and block callback dispatch, concrete collision clipping, damage systems,
loot tables, named RNG persistence, generation-fenced cross-Region coordination, sounds/particles,
criteria, and packet projection remain with their generated owners. The next active gameplay batch
is `G01-P5-S017`.
