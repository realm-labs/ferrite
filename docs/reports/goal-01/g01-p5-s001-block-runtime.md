# G01-P5-S001 — BLK-001 Block Runtime

## Result

Complete. All 41 `SourceSpecified` slices primarily owned by `BLK-001` map to production code and
committed behavioral tests.

The locked local content bundle lowered 1,196 block definitions and 32,366 canonical states.
This partition owns 40 behavior families containing 178 block IDs and 1,309 states. The additional
`BLK-STATE-SCHEMA-001` slice applies across the complete block registry.

## Evidence

Production owners:

- `ferrite-registry::minecraft_block` — fail-closed report lowering and raw/canonical lookup;
- `ferrite-registry::block_state` — strict direct mutation and lenient component patches;
- `ferrite-gameplay::block::runtime` — family ownership plus block-owned geometry, contact,
  storage, and operator transitions.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/blocks/blk_001.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices
cargo clippy -p ferrite-registry -p ferrite-gameplay -p ferrite-tooling --all-targets --all-features -- -D warnings
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused results before the universal gate:

```text
10 passed; 0 failed
content bundle: 32 registries, 9,078 entries
block catalog: 1,196 definitions, 32,366 canonical states
G01-P5-S001: 40 families, 178 block IDs, 1,309 states, 41 slices
```

## Ownership notes

The leaf rules explicitly delegate generic placement/breaking, item and loot allocation, entity
damage, redstone propagation, world-generation placement, resource rendering, and packet
projection to their shared owners. This batch locks every `BLK-001`-owned input and transition;
later generated partitions and phase closure tests own those joins. No deferred experiment or
guessed vanilla behavior was introduced.
