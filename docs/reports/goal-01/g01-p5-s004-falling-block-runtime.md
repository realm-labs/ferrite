# G01-P5-S004 — Falling Block Runtime

## Result

Complete. The one `SourceSpecified` slice primarily owned by `BLK-006`,
`BLK-FALLING-001`, now maps to production code and committed behavioral tests for the normative
`BLK-FALL-001` leaf.

## Evidence

Production owner:

- `ferrite-gameplay::block::falling` — exact 26-block classification, scheduling and start
  transaction, entity motion/landing/timeout/persistence decisions, and anvil, concrete powder,
  brushable, scaffolding, dragon egg, and ambient-particle subtype behavior.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/blocks/blk_006.rs`.

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
35 passed; 0 failed
1 SourceSpecified slice verified
```

## Ownership notes

This batch implements the block-owned deterministic transition kernel and its ordered semantic
effects. The later Phase 5 integration batch connects those effects to Region-owned blocks,
scheduled ticks, the private Region ECS, persistence, cross-Region delivery, and projection.
Generic collision, entity damage application, item spawning, game-event dispatch, and client
rendering retain their generated owners; their call positions, gates, values, and RNG cardinality
are fixed here without duplicating those downstream systems.
