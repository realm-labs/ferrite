# G01-P5-S008 — SIM-005 Block Runtime

## Result

Complete. Both `SourceSpecified` block slices primarily owned by `SIM-005` now map to modular
production semantics and committed behavioral tests. This closes all 125 block slices.

## Evidence

Production owners:

- `ferrite-gameplay::block::bell`;
- `ferrite-gameplay::block::enchanting_table`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/blocks/sim_005.rs`.

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
122 passed; 0 failed
2 SourceSpecified slices verified
125/125 block slices verified
```

## Ownership notes

This batch fixes Bell and Enchanting Table state identities, constants, transitions, ordered
effects, transient clocks, RNG boundaries and rendering inputs. Region state, ECS entities,
scheduled queues, content snapshots, persistence and projection remain their existing authorities.
Downstream enchanting offers and commits remain with `ITM-ENCHANT-001`; generic interaction,
block-event, effect, packet and particle policies remain with their generated shared owners.
