# G01-P5-S003 — Block Update and Runtime

## Result

Complete. Six `SourceSpecified` slices primarily owned by `BLK-003` now map to production code and
committed behavioral tests:

- `BLK-COMMAND-AREA-001`;
- `BLK-SPAWNER-RUNTIME-001`;
- `BLK-TRIAL-SPAWNER-RUNTIME-001`;
- `BLK-UPDATE-PIPELINE-001`;
- `BLK-VAULT-RUNTIME-001`;
- `BLK-VINE-RUNTIME-001`.

## Evidence

Production owners:

- `ferrite-gameplay::block::update` — flags, state-write lifecycle, collector, block events, and
  block-entity ticker lifecycle;
- `ferrite-gameplay::block::command_area` — clone/fill/fillbiome preflight, traversal, flags, count,
  and nonrollback phases;
- `ferrite-gameplay::block::spawner` and `trial_spawner` — ordinary and six-state encounter
  runtimes plus shared spawn-egg gates;
- `ferrite-gameplay::block::vault` — key, rewarded-player, state, persistence, and reverse-ejection
  runtime;
- `ferrite-gameplay::block::vine` — complete state/support/transform/growth decision kernel.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/blocks/blk_003.rs`.

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
27 passed; 0 failed
6 SourceSpecified slices verified
```

## Ownership notes

The batch implements block-owned transition kernels and semantic effect order. Phase 5 integration
later binds them to Region voxel transactions, boundary batches, persistence, and projection.
Generic entity admission, loot, effects, player state, and client rendering remain with their
explicit later owners; their call positions and source-owned inputs are fixed here without
duplicating or guessing downstream algorithms.
