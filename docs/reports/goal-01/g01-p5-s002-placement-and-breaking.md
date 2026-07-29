# G01-P5-S002 — Placement and Breaking

## Result

Complete. The three `SourceSpecified` slices primarily owned by `BLK-002` now map to production
gameplay code and committed behavioral tests:

- `BLK-PLACEMENT-001` / `BLK-PLACE-001`;
- `BLK-BREAKING-001` / `BLK-BREAK-001`;
- `BLK-BREAK-CONTENT-001` / `BLK-BREAK-HOOK-001`.

## Evidence

Production owners:

- `ferrite-gameplay::block::placement` — non-atomic write planning, exact flags, special block-item
  dispatch, target/scaffolding boundaries, and door hinge selection;
- `ferrite-gameplay::block::breaking` — Java-float mining progress, active/delayed tracker, and
  ordered generic destroy commit;
- `ferrite-gameplay::block::break_hook` — exact 110-ID/23-category concrete hook dispatch, hook
  positions, experience providers, and deterministic RNG consumption.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/blocks/blk_002.rs`.

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
17 passed; 0 failed
110 special block IDs; 23 concrete hook categories
3 SourceSpecified slices verified
```

## Ownership notes

This batch implements the block-owned algorithms and semantic effect order. Existing Phase 4
serverbound admission/acknowledgement and Region mutation remain their production owners; Phase 5
integration composes these richer transactions with boundary delivery, persistence, and client
projection in `G01-P5-B1`. Item/loot, entity, player-statistic, effect, and rendering leaves retain
their separate generated owners. No downstream behavior was guessed or falsely absorbed into the
block partition.
