# G01-P8-S001 — WGEN-001 Worldgen Pipeline

## Result

Complete. The source-known portion of the `SourceInconclusive`
`WGEN-PIPELINE-EQUIVALENCE-001` slice now has production owners for status scheduling, biome
sources, the full normal Overworld preset, flat and noise generators, density/noise evaluation,
aquifers and ore veins, runtime interpolation caches, upgrade blending, structure beardification,
surface rules and extensions, carvers, configured features, tree families, base-column queries, and
content-bundle projection.

The separate same-seed/statistical observations remain deferred under `EXP-WGEN-001`,
`EXP-WGEN-005`, and `EXP-WGEN-006`.

## Evidence

Production owner:

- `ferrite-world::generation`, split into 29 top-level responsibility modules and subordinate
  feature/tree modules.

Committed test owner:

- `crates/ferrite-world/tests/slices.rs`;
- `crates/ferrite-world/tests/slices/wgen_001*.rs`.

Design contract:

- [Minecraft 26.2 worldgen pipeline runtime](../../development/worldgen-pipeline-runtime.md).

Validated commands:

```text
cargo test -p ferrite-world --all-features
cargo clippy -p ferrite-world --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
12 ferrite-world unit tests passed; 0 failed
212 WGEN-001 slice tests passed; 0 failed
7,594/7,594 Overworld climate points matched the local locked report
681/681 WGEN-001 content records were available through the runtime catalog
7/7 noise settings and 7/7 world presets decoded through behavior-bearing projections
1 SourceInconclusive source-known surface verified
EXP-WGEN-001, EXP-WGEN-005, and EXP-WGEN-006 remain DeferredExperiment
```

## Coverage notes

The tests fix dependency and mutation admission; all four biome sources; climate tie behavior and
the `22/7568/4/7594` Overworld partition; flat parsing/fill; noise settings; pure and seeded density
families; improved, Perlin, normal, legacy blended, End-island and simplex noise; aquifer pressure
and status; ore-vein draw gates; runtime cache lifecycles; noise fill and base-column traversal;
upgrade height/density/biome blending; beardifier kernels; surface rules, clay bands and extensions;
carver dispatch/path/material behavior; configured-feature selectors and every source-owned
feature/tree family.

Ferrite intentionally makes no claim that its world for a given seed is block-for-block identical
to the official server. The implemented contract is deterministic source-known behavior and
auditable data projection; calibration thresholds and population equivalence require the named
deferred experiments.
