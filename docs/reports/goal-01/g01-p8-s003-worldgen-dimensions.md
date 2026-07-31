# G01-P8-S003 — WGEN-004 dimensions

## Result

Complete. `WGEN-DIMENSION-001` has a production owner and deterministic behavioral evidence in
`ferrite-world`.

The batch implements the four locked dimension records, all 48 typed environment declarations,
dimension/biome/timeline/weather resolution, camera interpolation, named global clocks, all four
locked timelines, build/light/weather/identity/coordinate gates, monster-light admission, initial
spawn planning, and position-local bed/anchor decisions.

## Evidence

Production owner:

- `ferrite-world::generation::dimension`;
- responsibility modules `environment`, `timeline`, `clock`, and `spawn`.

Committed test owner:

- `crates/ferrite-world/tests/slices/wgen_004.rs`.

Design contract:

- [Minecraft 26.2 dimension runtime](../../development/worldgen-dimension-runtime.md).

Validated commands:

```text
cargo test -p ferrite-world --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
git diff --check
```

Focused result before the repository gates:

```text
19 WGEN-004 slice tests passed; 0 failed
4/4 dimension records and 48/48 attribute declarations locked
4/4 timeline records, 6/6 day markers, and 23/23 total tracks locked
constant and uniform monster-light draw paths verified
1,024-candidate spawn cap and border/radius permutation branches verified
```

## Boundary disposition

The runtime deliberately preserves dimension key, type, generator/stem, default clock, and sampled
attribute map as independent inputs. Tests include the literal-End/custom-key weather cross-product,
fixed-time clocks that continue advancing, Nether universal timelines without a default clock,
inclusive build endpoints above Nether logical height, and positional gameplay gates.

S003 supplies unrounded coordinate scale and dimension-key identity to later batches. Nether portal
search/creation remains `G01-P8-S004`; world-border state and clamping remain `G01-P8-S005`.
