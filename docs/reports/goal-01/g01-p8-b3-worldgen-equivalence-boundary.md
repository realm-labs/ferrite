# G01-P8-B3 — Worldgen equivalence boundary

## Result

Complete. Phase 8 now records the source-inconclusive statistical-equivalence observation as a
separate executable contract without treating it as missing source-specified implementation work.
All 27 source-specified world slices remain verified; the one source-inconclusive world slice has a
verified deterministic Ferrite implementation and an explicitly deferred observation.

## Evidence

Executable owner:

- `crates/ferrite-testkit::phase8::equivalence`;
- `apps/behavior-runner/tests/experiments/wgen_pipeline_equivalence_001.rs`.

Policy contract:

- [World-generation equivalence boundary](../../development/worldgen-equivalence-boundary.md);
- [Phase 8 conformance](../../development/phase8-conformance.md).

Validated commands:

```text
cargo test -p behavior-runner --test experiments --all-features
cargo test -p ferrite-world --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
cargo ferrite task check
git diff --check
```

Focused result before repository gates:

```text
3 exact WGEN experiment definitions retained as planned
8,200 total planned repetitions retained (8 + 4,096 + 4,096)
27 SourceSpecified world slices Verified; 1 SourceInconclusive implementation Verified
identical Ferrite seed replay matched; distinct Ferrite seed diverged
same-seed vanilla identity=false; committed statistical thresholds=false
```

## Boundary disposition

`WGEN-PIPELINE-EQUIVALENCE-001` remains `DeferredExperiment` for its observation only. The retained
policy forbids a block-for-block same-seed claim, and replacement requires committed named
statistical thresholds. No planned experiment was reported as executed, no result was invented,
and no source-specified behavior was deferred. With that distinction recorded, Phase 8 is complete
and Phase 9 may begin.
